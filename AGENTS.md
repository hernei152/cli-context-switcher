# Repository Guidelines

## Project Structure & Module Organization

This repository contains a single Rust binary. `src/main.rs` holds CLI parsing, PTY lifecycle management, session persistence, transcript replay, and its unit tests. `Cargo.toml` defines dependencies and release settings; keep `Cargo.lock` committed so builds remain reproducible. `test.sh` is the end-to-end PTY smoke test. User-facing behavior and installation instructions live in `README.md`. Build output belongs in `target/` and must not be committed.

## Build, Test, and Development Commands

- `cargo build`: compile a debug binary for quick iteration.
- `cargo build --release`: produce the optimized, stripped binary at `target/release/hernei`; required before running `test.sh`.
- `cargo run -- <command> [args]`: run Hernei locally, for example `cargo run -- bash`.
- `cargo test`: run the unit tests embedded in `src/main.rs`.
- `./test.sh`: exercise PTY recording, exit-code propagation, naming, and session restoration. It requires the Unix `script` utility.
- `cargo fmt --check`: verify standard Rust formatting.
- `cargo clippy --all-targets -- -D warnings`: catch common Rust mistakes and treat warnings as failures.

## Coding Style & Naming Conventions

Use standard `rustfmt` output (four-space indentation) and idiomatic Rust ownership and error handling. Prefer `anyhow::Result` with contextual errors at I/O and process boundaries. Follow the existing naming style: `snake_case` functions and variables, `PascalCase` types, and descriptive Spanish names/comments consistent with the current codebase. Keep terminal state changes guarded by RAII types with `Drop` cleanup, as done by `ModoRaw` and `PantallaAlt`. Avoid panics in runtime paths; malformed or missing session data should return an error or degrade safely.

## Testing Guidelines

Add focused `#[test]` cases to the existing test module for pure parsing and transcript behavior. Test names should describe behavior in `snake_case`, such as `el_redibujado_no_queda_duplicado`. Changes involving the PTY, terminal modes, wrapped command arguments, or files under `~/.hernei` should also extend and run `test.sh`. No numeric coverage target is enforced; cover regressions and error paths introduced by each change.

## Commit & Pull Request Guidelines

Recent commits use short Spanish subjects, usually `scope: descripción`, for example `hernei: aceptar el nombre después del comando` or `docs: agregar instrucciones de instalación`. Keep commits focused and use the imperative or concise infinitive style. Pull requests should explain the user-visible effect, list validation commands, and link relevant issues. Include terminal output or a short recording when TUI behavior changes, and update `README.md` when CLI flags, installation, or session behavior changes.
