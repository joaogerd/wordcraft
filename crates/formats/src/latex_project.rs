//! Filesystem-aware LaTeX project resolution for scientific import.
//! This is deliberately not a full TeX macro interpreter.
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

const MAX_DEPTH: usize = 24;
const MAX_BYTES: usize = 32 * 1024 * 1024;
const LITERAL_ENVIRONMENTS: &[&str] = &["verbatim", "verbatim*", "Verbatim", "lstlisting", "minted", "alltt", "comment"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub file: PathBuf,
    pub message: String,
}

#[derive(Debug)]
pub struct ProjectSource {
    pub source: String,
    pub diagnostics: Vec<Diagnostic>,
    pub files_read: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("failed to read project file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("file is outside project root: {0}")]
    OutsideRoot(PathBuf),
    #[error("non-UTF-8 source file: {0}")]
    NonUtf8(PathBuf),
    #[error("project expansion exceeded {0} bytes")]
    TooLarge(usize),
    #[error("include nesting exceeded {0} levels")]
    TooDeep(usize),
    #[error("include cycle: {0}")]
    Cycle(PathBuf),
}

struct Context {
    root: PathBuf,
    stack: Vec<PathBuf>,
    files_read: Vec<PathBuf>,
    diagnostics: Vec<Diagnostic>,
    remaining: usize,
}

/// Resolve braced \input and \include from a canonical project root.
/// Included paths are relative to the including file, not TEXINPUTS.
/// Unrecognized commands remain unchanged.
pub fn load(root: &Path, main: &Path) -> Result<ProjectSource, ProjectError> {
    let root = fs::canonicalize(root).map_err(|source| ProjectError::Io { path: root.into(), source })?;
    let mut ctx = Context { root: root.clone(), stack: Vec::new(), files_read: Vec::new(), diagnostics: Vec::new(), remaining: MAX_BYTES };
    let source = ctx.expand(&root.join(main), 0)?;
    Ok(ProjectSource { source, diagnostics: ctx.diagnostics, files_read: ctx.files_read })
}

impl Context {
    fn expand(&mut self, input: &Path, depth: usize) -> Result<String, ProjectError> {
        if depth >= MAX_DEPTH {
            return Err(ProjectError::TooDeep(MAX_DEPTH));
        }
        let normalized = normalize(input);
        if !normalized.starts_with(&self.root) {
            return Err(ProjectError::OutsideRoot(normalized));
        }
        let file = fs::canonicalize(&normalized).map_err(|source| ProjectError::Io { path: normalized, source })?;
        if !file.starts_with(&self.root) {
            return Err(ProjectError::OutsideRoot(file));
        }
        if self.stack.contains(&file) {
            return Err(ProjectError::Cycle(file));
        }
        let mut reader = File::open(&file).map_err(|source| ProjectError::Io { path: file.clone(), source })?;
        let limit = self.remaining.saturating_add(1);
        let mut bytes = Vec::with_capacity(self.remaining.min(64 * 1024));
        reader.by_ref().take(limit as u64).read_to_end(&mut bytes).map_err(|source| ProjectError::Io { path: file.clone(), source })?;
        if bytes.len() > self.remaining {
            return Err(ProjectError::TooLarge(MAX_BYTES));
        }
        self.remaining -= bytes.len();
        let input = String::from_utf8(bytes).map_err(|_| ProjectError::NonUtf8(file.clone()))?;
        self.stack.push(file.clone());
        self.files_read.push(file.clone());
        let result = self.scan(&file, &input, depth);
        self.stack.pop();
        result
    }

    fn scan(&mut self, file: &Path, source: &str, depth: usize) -> Result<String, ProjectError> {
        let bytes = source.as_bytes();
        let mut out = String::with_capacity(source.len());
        let mut cursor = 0;
        let mut i = 0;
        let mut in_comment = false;
        while i < bytes.len() {
            if bytes[i] == b'\n' {
                in_comment = false;
                i += 1;
                continue;
            }
            if in_comment {
                i += 1;
                continue;
            }
            if bytes[i] == b'%' && !is_escaped(bytes, i) {
                in_comment = true;
                i += 1;
                continue;
            }
            if bytes[i] != b'\\' || is_escaped(bytes, i) {
                i += 1;
                continue;
            }
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
                end += 1;
            }
            let cmd = &source[start..end];
            if cmd == "verb" {
                let mut delimiter = end;
                if bytes.get(delimiter) == Some(&b'*') {
                    delimiter += 1;
                }
                if let Some(&mark) = bytes.get(delimiter)
                    && mark != b'\n'
                    && mark != b'\r'
                {
                    i = bytes[delimiter + 1..].iter().position(|&byte| byte == mark).map_or(bytes.len(), |offset| delimiter + offset + 2);
                    continue;
                }
            }
            if cmd == "begin"
                && let Some((environment, close)) = braced_argument(source, end)
                && LITERAL_ENVIRONMENTS.contains(&environment)
            {
                let terminator = format!("\\end{{{environment}}}");
                i = source[close + 1..].find(&terminator).map_or(bytes.len(), |offset| close + 1 + offset + terminator.len());
                continue;
            }
            if cmd != "input" && cmd != "include" {
                i = end.max(i + 1);
                continue;
            }
            let arg = skip_tex_trivia(bytes, end);
            if arg >= bytes.len() || bytes[arg] != b'{' {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "UNSUPPORTED_INCLUDE_SYNTAX",
                    file: file.into(),
                    message: format!("\\{cmd} without braced filename was not expanded"),
                });
                i = end;
                continue;
            }
            let arg_start = arg + 1;
            let Some(close) = (arg_start..bytes.len()).find(|&k| bytes[k] == b'}' || bytes[k] == b'{') else {
                self.malformed_include(file, cmd);
                i = end;
                continue;
            };
            if bytes[close] == b'{' {
                self.malformed_include(file, cmd);
                i = end;
                continue;
            }
            let name = source[arg_start..close].trim();
            if name.is_empty() || name.contains('\\') || name.contains('\0') {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "DYNAMIC_INCLUDE",
                    file: file.into(),
                    message: format!("unresolved dynamic include: {name}"),
                });
                i = close + 1;
                continue;
            }
            let base = file.parent().unwrap_or(&self.root);
            let candidate = base.join(name);
            let candidate = if candidate.extension().is_some() { candidate } else { candidate.with_extension("tex") };
            out.push_str(&source[cursor..i]);
            let contents = self.expand(&candidate, depth + 1)?;
            out.push_str(&contents);
            cursor = close + 1;
            i = cursor;
        }
        out.push_str(&source[cursor..]);
        Ok(out)
    }

    fn malformed_include(&mut self, file: &Path, command: &str) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "MALFORMED_INCLUDE",
            file: file.into(),
            message: format!("\\{command} has an unclosed or nested filename argument"),
        });
    }
}

fn braced_argument(source: &str, command_end: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut open = command_end;
    while matches!(bytes.get(open), Some(b' ' | b'\t')) {
        open += 1;
    }
    if bytes.get(open) != Some(&b'{') {
        return None;
    }
    let start = open + 1;
    let close = bytes[start..].iter().position(|&byte| byte == b'}')? + start;
    Some((&source[start..close], close))
}

fn skip_tex_trivia(bytes: &[u8], mut position: usize) -> usize {
    loop {
        while bytes.get(position).is_some_and(u8::is_ascii_whitespace) {
            position += 1;
        }
        if bytes.get(position) != Some(&b'%') {
            return position;
        }
        position += 1;
        while !matches!(bytes.get(position), None | Some(b'\n' | b'\r')) {
            position += 1;
        }
    }
}

fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn is_escaped(bytes: &[u8], pos: usize) -> bool {
    let mut n = 0;
    let mut i = pos;
    while i > 0 && bytes[i - 1] == b'\\' {
        n += 1;
        i -= 1;
    }
    n % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Tmp(PathBuf);
    impl Tmp {
        fn new() -> Self {
            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let p = std::env::temp_dir().join(format!("wc-latex-project-{}-{stamp}", std::process::id()));
            fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn put(&self, name: &str, data: &str) {
            let f = self.0.join(name);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, data).unwrap();
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn expands_nested_input_and_include() {
        let t = Tmp::new();
        t.put("main.tex", "Start \\input{parts/a} End");
        t.put("parts/a.tex", "A \\include{b}");
        t.put("parts/b.tex", "B");
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert!(r.source.contains("A"));
        assert!(r.source.contains("B"));
        assert_eq!(r.files_read.len(), 3);
    }
    #[test]
    fn ignores_commented_and_escaped_commands() {
        let t = Tmp::new();
        t.put("main.tex", "% \\input{missing}\n\\% literal \\input{real}");
        t.put("real.tex", "OK");
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert!(r.source.contains("OK"));
    }
    #[test]
    fn keeps_verbatim_environments_literal() {
        let t = Tmp::new();
        let source = "Before\\n\\begin{verbatim}\\n\\input{missing} % literal\\n\\end{verbatim}\\nAfter";
        t.put("main.tex", source);
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert_eq!(r.source, source);
        assert_eq!(r.files_read.len(), 1);
    }
    #[test]
    fn keeps_inline_verb_commands_literal() {
        let t = Tmp::new();
        let source = "Before \\verb|\\input{missing}| and \\verb*+\\include{missing}+ after";
        t.put("main.tex", source);
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert_eq!(r.source, source);
        assert_eq!(r.files_read.len(), 1);
    }
    #[test]
    fn reports_unclosed_include_argument_without_mutating_it() {
        let t = Tmp::new();
        let source = "Scientific $E=mc^2$ and \\input{unfinished";
        t.put("main.tex", source);
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert_eq!(r.source, source);
        assert!(r.diagnostics.iter().any(|d| d.code == "MALFORMED_INCLUDE"));
    }
    #[test]
    fn expansion_only_replaces_the_include_command() {
        let t = Tmp::new();
        t.put("main.tex", "Alpha\\input{part}Omega");
        t.put("part.tex", "$\\alpha + \\beta$\\n\\SI{9.81}{m.s^{-2}}");
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert_eq!(r.source, "Alpha$\\alpha + \\beta$\\n\\SI{9.81}{m.s^{-2}}Omega");
    }
    #[test]
    fn resolves_include_after_tex_whitespace_and_comment() {
        let t = Tmp::new();
        t.put("main.tex", "A\\input % source note\n  {part}B");
        t.put("part.tex", "X");
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert_eq!(r.source, "AXB");
    }
    #[test]
    fn rejects_parent_directory_escape() {
        let t = Tmp::new();
        t.put("main.tex", "\\input{../outside}");
        let e = load(&t.0, Path::new("main.tex")).unwrap_err();
        assert!(matches!(e, ProjectError::OutsideRoot(_)));
    }
    #[test]
    fn detects_include_cycle() {
        let t = Tmp::new();
        t.put("main.tex", "\\input{a}");
        t.put("a.tex", "\\input{main}");
        assert!(matches!(load(&t.0, Path::new("main.tex")), Err(ProjectError::Cycle(_))));
    }
    #[test]
    fn rejects_excessive_include_depth() {
        let t = Tmp::new();
        for depth in 0..=MAX_DEPTH {
            t.put(&format!("{depth}.tex"), &format!("\\input{{{}}}", depth + 1));
        }
        assert!(matches!(load(&t.0, Path::new("0.tex")), Err(ProjectError::TooDeep(MAX_DEPTH))));
    }
    #[test]
    fn rejects_non_utf8_source() {
        let t = Tmp::new();
        fs::write(t.0.join("main.tex"), [0xff]).unwrap();
        assert!(matches!(load(&t.0, Path::new("main.tex")), Err(ProjectError::NonUtf8(_))));
    }
    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let project = Tmp::new();
        let outside = Tmp::new();
        outside.put("secret.tex", "secret");
        project.put("main.tex", "\\input{escape}");
        symlink(outside.0.join("secret.tex"), project.0.join("escape.tex")).unwrap();
        assert!(matches!(load(&project.0, Path::new("main.tex")), Err(ProjectError::OutsideRoot(_))));
    }
    #[test]
    fn preserves_inline_math() {
        let t = Tmp::new();
        t.put("main.tex", "The value $x^2$ is unchanged.");
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert_eq!(r.source, "The value $x^2$ is unchanged.");
    }
}
