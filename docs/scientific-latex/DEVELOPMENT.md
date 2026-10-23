# Development workflow

## Branch policy

- `main`: stable/released code only; never commit feature work directly.
- `develop`: integration branch for milestones which have passed their acceptance gates.
- `feature/scientific-latex-mN-*`: create from latest `develop`; implement and validate a single milestone.
- `fix/scientific-latex-*`: focused fixes based on `develop` unless fixing a release.
- `release/scientific-latex-*`: optional release hardening from `develop`, merged to `main` after release checks.

## Promotion policy

1. Develop on the milestone feature branch; commit small, reviewable changes.
2. Add or update tests, fixtures, docs and known-limitations log.
3. Run `cargo fmt --all -- --check`, relevant `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings` where compatible, and repo `cargo xtask ci` when available.
4. Open **draft PR to develop** until all tests and acceptance criteria pass.
5. Review failures, update PR with test evidence and mark ready.
6. Merge PR into `develop`; verify integration checks there.
7. Begin M(N+1) from the updated `develop`.
8. Promote `develop` to `main` only through a separate tested release PR.

No automatic merging on milestone name or time. Protected branch rules and required CI checks are recommended.

## PR template for milestone

- Scope, design and touched crates
- Acceptance criteria checklist
- Unit, integration, property and regression test results
- Supported and unsupported constructs
- Structural/data-loss inspection
- PDF comparison evidence when applicable
- Security review for file/resource access
- Upstream compatibility and rollback considerations

## Upstream workflow

Periodically sync upstream `storytold/wordcraft` into the fork, resolve conflicts on a dedicated integration branch and rerun WordCraft CI. Do not rebase published `develop` destructively.

## Initial commands

```sh
git fetch origin
git switch develop
git pull --ff-only origin develop
git switch -c feature/scientific-latex-m2-semantics
cargo fmt --all -- --check
cargo test -p wordcraft-formats
```
