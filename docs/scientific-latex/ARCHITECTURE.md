# Architecture and API contracts

## Pipeline

1. **Project resolver**: locate root .tex, normalize paths, resolve `\\input`/`\\include`, `\\graphicspath`, bibliography, sources and dependencies. Enforce resource and recursion limits; reject traversal and symlink escapes.
2. **LaTeX parser**: tokenize and parse supported constructs, preserve source spans, report unsupported macros/environment semantics. Reuse existing `crates/formats/src/latex.rs` where practical.
3. **Scientific Document Model (SDM)**: stable node IDs, source spans, typed blocks/inlines, math expressions, figure/table captions, label registry, citation registry, layout intents, metadata and diagnostics. No UI dependency.
4. **Document builder**: lower SDM into `wordcraft_doc::Document`, using existing math nodes, styles, numbering, fields and images; output through `wordcraft-docx`. Equations must remain native editable OMML.
5. **Validation**: reopen the DOCX and check texts, ordered blocks, math trees, figures, table structures, labels, citation targets and editability before assessing visuals.
6. **Optional visual fidelity**: align semantic blocks between a trusted reference PDF and a rendered DOCX; tune only permitted Word document properties and retain the most faithful valid iteration.

## Dependency policy

Keep `wordcraft-doc` and `wordcraft-docx` generic; do not introduce dependencies on LaTeX into those foundational crates. Initially place small project-resolution code in `wordcraft-formats` or a self-contained `wordcraft-latex-project` crate. A separate `scientific-doc` crate is justified when SDM stabilizes. Keep the visual comparator outside the basic parser to avoid an engine dependency cycle. Rust workspace forbids unsafe code and uses bounded parsing.

## Planned public interface (not implemented yet)

```rust
enum FidelityMode { Visual, Balanced, Editable }
struct ImportOptions {
    mode: FidelityMode,
    reference_pdf: Option<PathBuf>,
    template_docx: Option<PathBuf>,
    fail_on_loss: bool,
}
struct ImportResult {
    document: wordcraft_doc::Document,
    report: ConversionReport,
}
```

`ConversionReport` will contain severity, diagnostic code, source file and offset, fallback employed, unsupported features, resource resolution, structural checks and optional visual metrics.

## Math and OOXML

Prefer existing `wordcraft_doc::math::MNode` rather than writing an independent math representation. Preserve equation environment/alignment/numbering outside the math expression itself. Round-trip LaTeX math → structured tree → OMML → DOCX reopen and compare supported AST semantics.

## Floats, bibliography and cross-references

Resolve image data instead of emitting `[Picture: …]`. Produce actual editable Word tables and caption/numbering styles. Map labels to stable bookmarks/fields or equivalent native structures; citations must reference resolved bibliographic records or report fallback.

## Trust boundary

Do not execute arbitrary TeX during document import. Optional TeX compilation for reference PDF requires isolation, no shell-escape, restricted filesystem/network access and resource limits. Untrusted ZIP files must not escape their extraction root.
