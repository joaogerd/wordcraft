# Quality and acceptance criteria

## Mandatory integrity invariants

- No silent removal or mutation of text, numbers, units, or supported scientific notation.
- Preserve ordering of logical scientific content.
- Supported equations must be native, editable Word math, verified by DOCX reopen and tree comparison.
- All included figures must be embedded or explicitly reported as unresolved.
- All supported references and citations must resolve or cause an actionable diagnostic.
- Any visual optimization must preserve all verified semantic invariants.

## Proposed M7 quantitative goals

These are **targets, not current measured performance**:
- 100% of text and numeric information recovered, or explicitly diagnosed where unsupported.
- 100% of supported equations structurally correct and editable in the tested corpus.
- 100% of supported references resolved and tested.
- At least 95% typography compliance using fixed attribute tolerances.
- At least 90% of matched layout blocks within a normalized vertical displacement of 2% page height.
- DOCX opens, edits and round-trips without critical loss in tested WordCraft, Microsoft Word and LibreOffice versions.

Never average away a semantic failure with a good visual score.

## Corpus structure

```
tests/scientific-import/
  fixtures/
    basic-article/
    multifile/
    math/
    figures/
    tables/
    bibliography/
    unsafe-input/
  golden/
  regression/
```

Use synthetic, redistributable examples at first; obtain explicit rights before committing real published paper source. Separate baseline and held-out test sets.

## Performance and security

Bound source bytes, recursion, token counts, output objects and archive extraction size. Deny traversal outside project root and symlink escape. Use fuzzing and malformed-file tests. Never silently execute macros or shell commands contained in imported LaTeX.
