//! Filesystem-aware LaTeX project resolution for scientific import.
//! This is deliberately not a full TeX macro interpreter.
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 24;
const MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity { Warning, Error }

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
    Io { path: PathBuf, #[source] source: std::io::Error },
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
    let mut ctx = Context {
        root: root.clone(), stack: Vec::new(), files_read: Vec::new(),
        diagnostics: Vec::new(), remaining: MAX_BYTES,
    };
    let source = ctx.expand(&root.join(main), 0)?;
    Ok(ProjectSource { source, diagnostics: ctx.diagnostics, files_read: ctx.files_read })
}

impl Context {
    fn expand(&mut self, input: &Path, depth: usize) -> Result<String, ProjectError> {
        if depth >= MAX_DEPTH { return Err(ProjectError::TooDeep(MAX_DEPTH)); }
        let file = fs::canonicalize(input).map_err(|source| ProjectError::Io { path: input.into(), source })?;
        if !file.starts_with(&self.root) { return Err(ProjectError::OutsideRoot(file)); }
        if self.stack.contains(&file) { return Err(ProjectError::Cycle(file)); }
        let bytes = fs::read(&file).map_err(|source| ProjectError::Io { path: file.clone(), source })?;
        if bytes.len() > self.remaining { return Err(ProjectError::TooLarge(MAX_BYTES)); }
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
            if bytes[i] == b'\n' { in_comment = false; i += 1; continue; }
            if in_comment { i += 1; continue; }
            if bytes[i] == b'%' && !is_escaped(bytes, i) { in_comment = true; i += 1; continue; }
            if bytes[i] != b'\\' || is_escaped(bytes, i) { i += 1; continue; }
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_alphabetic() { end += 1; }
            let cmd = &source[start..end];
            if cmd != "input" && cmd != "include" { i = end.max(i + 1); continue; }
            let mut arg = end;
            while arg < bytes.len() && (bytes[arg] == b' ' || bytes[arg] == b'\t') { arg += 1; }
            if arg >= bytes.len() || bytes[arg] != b'{' {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Warning, code: "UNSUPPORTED_INCLUDE_SYNTAX",
                    file: file.into(), message: format!("\\{cmd} without braced filename was not expanded"),
                });
                i = end;
                continue;
            }
            let arg_start = arg + 1;
            let Some(close) = (arg_start..bytes.len()).find(|&k| bytes[k] == b'}' || bytes[k] == b'{') else {
                i = end;
                continue;
            };
            if bytes[close] == b'{' { i = end; continue; }
            let name = source[arg_start..close].trim();
            if name.is_empty() || name.contains('\\') || name.contains('\0') {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Warning, code: "DYNAMIC_INCLUDE",
                    file: file.into(), message: format!("unresolved dynamic include: {name}"),
                });
                i = close + 1;
                continue;
            }
            let base = file.parent().unwrap_or(&self.root);
            let candidate = base.join(name);
            let candidate = if candidate.extension().is_some() { candidate } else { candidate.with_extension("tex") };
            out.push_str(&source[cursor..i]);
            let contents = self.expand(&candidate, depth + 1)?;
            out.push_str("\n% wordcraft: expanded include begin\n");
            out.push_str(&contents);
            out.push_str("\n% wordcraft: expanded include end\n");
            cursor = close + 1;
            i = cursor;
        }
        out.push_str(&source[cursor..]);
        Ok(out)
    }
}

fn is_escaped(bytes: &[u8], pos: usize) -> bool {
    let mut n = 0;
    let mut i = pos;
    while i > 0 && bytes[i - 1] == b'\\' { n += 1; i -= 1; }
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
    impl Drop for Tmp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

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
    fn rejects_parent_directory_escape() {
        let t = Tmp::new();
        t.put("main.tex", "\\input{../outside}");
        let e = load(&t.0, Path::new("main.tex")).unwrap_err();
        assert!(matches!(e, ProjectError::Io { .. } | ProjectError::OutsideRoot(_)));
    }
    #[test]
    fn detects_include_cycle() {
        let t = Tmp::new();
        t.put("main.tex", "\\input{a}");
        t.put("a.tex", "\\input{main}");
        assert!(matches!(load(&t.0, Path::new("main.tex")), Err(ProjectError::Cycle(_))));
    }
    #[test]
    fn preserves_inline_math() {
        let t = Tmp::new();
        t.put("main.tex", "The value $x^2$ is unchanged.");
        let r = load(&t.0, Path::new("main.tex")).unwrap();
        assert_eq!(r.source, "The value $x^2$ is unchanged.");
    }
}
