# Scientific LaTeX Importer

Status: **M1 in progress**. The goal is to convert scientific LaTeX projects into editable DOCX documents with a balanced priority: **semantic correctness > editability > document structure > typography > pagination**. Pandoc is **not** a required runtime dependency.

## Entry points

- [Architecture](ARCHITECTURE.md) — pipelines, reuse of WordCraft crates, dependency boundaries.
- [Milestone plan](ROADMAP.md) — M1–M7, outputs and acceptance gates.
- [Development workflow](DEVELOPMENT.md) — branches, pull requests, CI, and integration into `develop`.
- [Quality policy](QUALITY.md) — corpus, lossless assertions, objective measurements.

## Current WordCraft baseline

The existing `crates/formats/src/latex.rs` parses a useful LaTeX subset, but does not expand user macros, represents `\\includegraphics` as a textual placeholder, converts citation keys into text, and does not resolve `\\eqref` into display numbers. Reuse the existing `wordcraft-doc` mathematical tree, `wordcraft-docx` serializer, layout engine, and CLI.

## Scope

Inputs: `.tex`, project directory or ZIP, images, bibliography, optional reference PDF. Outputs: `.docx`, conversion diagnostics, optional rendered comparison. An unsupported construct must generate an explicit diagnostic; **silent semantic loss is not acceptable**.

Work starts on feature branches from `develop`. Every completed milestone is merged into `develop` only after its acceptance tests pass. `main` remains the release branch.
