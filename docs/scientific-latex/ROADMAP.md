# Delivery plan — M1–M7

A milestone is **complete** only when code, tests, documentation and acceptance evidence are committed in the feature branch, CI passes, and a reviewed PR is merged into `develop`. Creating a branch or draft is not milestone completion.

| Milestone | Scope | Exit gate |
|---|---|---|
| **M1** Foundation | Audit existing modules; project resolver for root .tex, includes, path safety and limits; initial fixtures; contracts and docs | Resolver unit tests + formatting/clippy/CI passing; no regression in existing import; basic examples indexed |
| **M2** Semantics | Project AST/SDM, stable IDs, source spans, supported macros and diagnostic model | Cross-file semantics represented; unsupported commands diagnosed, not silently lost |
| **M3** Scientific content | Native OMML equations, real images, editable tables, DOCX builder | Math round-trip and content preservation fixture tests |
| **M4** References and styles | BibTeX, citation styles, labels/refs, captions, section layout, Word styles | References and numbering resolved on reopen; specified supported citation cases pass |
| **M5** Fidelity engine | Semantic block matching, typography and geometry measures, bounded optimizer | Reproducible PDF comparison; no semantic degradation on optimization |
| **M6** Product integration | CLI, GUI, structured report, progress and failures | Documented CLI and UI integration tests |
| **M7** Stabilization | Cross-editor testing, fuzzing, robustness and release review | Versioned corpus and quality acceptance gates satisfied |

## M1 acceptance record (2026-10-23)

Implementation is complete on `feature/scientific-latex-m1-foundation`; the milestone remains
unpromoted until PR #1 is reviewed and merged into `develop`.

- [x] Existing LaTeX importer and format dispatch audited without changing their public behavior.
- [x] Bounded Rust project resolver exported from `wordcraft-formats`; no Pandoc runtime dependency.
- [x] Braced `\input` and `\include` resolve relative to the including file, including TeX whitespace
  and comments before the filename.
- [x] Canonical containment rejects parent traversal and symlink escape; active-stack tracking rejects
  include cycles; input is capped at 32 MiB cumulatively and nesting at 24 files.
- [x] Comments, escaped commands, `\verb`, and literal environments remain unexpanded; malformed or
  dynamic include arguments remain in source and receive diagnostics.
- [x] Included scientific text replaces only the include command; the resolver adds no marker text or
  whitespace. Equations, units, numbers, and unknown macros are preserved byte for byte.
- [x] Synthetic fixtures are indexed in `tests/scientific-import/README.md` and the multifile fixture
  is exercised by a resolver regression test.
- [x] `cargo fmt --all -- --check` and `cargo test -p wordcraft-formats` pass (261 tests).
- [x] `cargo clippy -p wordcraft-formats --all-targets --no-deps -- -D warnings` passes for the changed
  crate. The exact command without `--no-deps` is blocked by pre-existing `nonminimal_bool` warnings
  in `crates/doc/src/math_linear.rs:559`, outside this milestone.
- [ ] `cargo xtask ci` and remote PR checks pass; record final results in PR #1 before review.

### Known M1 limitations

- This is deliberately not a TeX macro interpreter. Dynamic filenames and unbraced include syntax are
  preserved with warnings instead of being executed.
- `TEXINPUTS`, `\includeonly`, archive extraction, `\graphicspath`, bibliography resources, and macro
  expansion are deferred to later milestones.
- Source files must be UTF-8. A non-UTF-8 file fails explicitly instead of being decoded heuristically.
- Filesystem containment uses lexical and canonical checks. A hostile process concurrently replacing
  path components during a read is outside the M1 threat model; importer-controlled archive extraction
  must use its own confined extraction API when implemented.

## Recommended branches

`feature/scientific-latex-m1-foundation`, then `feature/scientific-latex-m2-semantics`, …, `feature/scientific-latex-m7-stabilization`. Branch each milestone from updated `develop`.

## Milestone documentation

Keep each milestone's PR checklist updated with tests, known limitations and any intentional fallback. Provide an ADR for consequential design changes. Never merge a partially implemented milestone simply to mark progress: use a draft PR until gates pass.

## Not in the first release

Full TeX macro expansion, arbitrary TikZ editable conversion, perfect cross-application page identity, and comprehensive custom class support.
