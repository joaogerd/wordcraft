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

## Recommended branches

`feature/scientific-latex-m1-foundation`, then `feature/scientific-latex-m2-semantics`, …, `feature/scientific-latex-m7-stabilization`. Branch each milestone from updated `develop`.

## Milestone documentation

Keep each milestone's PR checklist updated with tests, known limitations and any intentional fallback. Provide an ADR for consequential design changes. Never merge a partially implemented milestone simply to mark progress: use a draft PR until gates pass.

## Not in the first release

Full TeX macro expansion, arbitrary TikZ editable conversion, perfect cross-application page identity, and comprehensive custom class support.
