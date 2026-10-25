# AGENTS.md

Instructions for AI coding agents. See [README.md](README.md) for project overview and structure.

## Project intention

Terminal-based Elite-style space trading game in Rust: exploration, commodity trading, and dynamic events. Implementations are driven by feature specs and BDD-style tests.

**Feature specs and implementation details** — See the [features/](features/) directory. Each feature has its own `.md` spec (e.g. `features/02-star-systems.md`). Read the relevant spec before implementing or changing a feature.

## Commands

- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt`

Run tests and lint before committing.

## Workflow

1. **Feature spec first** — Add a spec as an `.md` file in `features/` before coding.
2. **BDD-style tests** — Write outside-in / feature tests before implementing.
3. **Remove unused code** after completing a feature.
4. **Ask before implementing** — Confirm with the user before making code changes.

## Commits

- Use [Conventional Commits](https://www.conventionalcommits.org/): one line, no scope (e.g. `feat: add travel between systems`).
- Message must be a human-readable summary of the change, not a diff.
- Tests must be green and the linter clean before committing.
- Never use `git push --force`; `--force-with-lease` is acceptable if needed.

## Code

- Idiomatic Rust: `snake_case` for functions and variables, `PascalCase` for types, expressive names (e.g. `is_ready`, `has_data`).
- Propagate errors with `Result`/`Option` and `?`; use `anyhow` or `thiserror` for descriptive errors. Handle edge cases early.
- Keep modules focused; avoid duplicated logic.
- Keep code synchronous unless something genuinely needs async.
- Add unit tests alongside the code and feature-level tests for behaviour.

## Communication

- Be direct: give the information needed, and say so plainly if the user is wrong. No filler praise.
