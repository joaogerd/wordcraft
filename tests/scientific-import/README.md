# Scientific import corpus

All M1 sources in this directory are synthetic WordCraft test material and may be redistributed
under the repository licence.

| Fixture | Purpose | M1 expectation |
|---|---|---|
| `fixtures/basic-article/main.tex` | Small scientific article with section and equation | Source remains unchanged when no project include is present |
| `fixtures/multifile/main.tex` | Nested `\input`/`\include` project | Files resolve relative to their including file in source order |
| `fixtures/unsafe-input/main.tex` | Parent-directory traversal attempt | Resolver returns `ProjectError::OutsideRoot` |

Later milestones may add golden DOCX/semantic outputs. M1 tests the filesystem resolver only.
