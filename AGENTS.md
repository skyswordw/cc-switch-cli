# Repository Guidelines

## Project Structure

- `src-tauri/`: Rust crate for the `cc-switch` CLI.
  - `src-tauri/src/cli/`: command parsing + interactive TUI flows.
  - `src-tauri/src/services/`: core business logic (providers, MCP, prompts, config, env).
  - `src-tauri/tests/`: Rust integration tests (`*.rs`).
- `assets/`: screenshots and partner assets used in docs.
- `docs/`: design/refactor notes, release notes, and internal plans.
- `scripts/`: release/versioning helpers (shell + Node scripts).

## Build, Test, and Development Commands

Run commands from the repo root unless noted.

- Build debug binary: `cd src-tauri && cargo build`
- Build release binary: `cd src-tauri && cargo build --release`
- Run locally: `cd src-tauri && cargo run --bin cc-switch -- --help`
- Run tests: `cd src-tauri && cargo test`
- Format: `cd src-tauri && cargo fmt`
- Lint (recommended): `cd src-tauri && cargo clippy --all-targets -- -D warnings`

## Coding Style & Naming Conventions

- Rust formatting is standard `rustfmt`; don’t hand-format—run `cargo fmt`.
- Use Rust conventions: `snake_case` for modules/functions, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Keep CLI output stable and user-facing strings i18n-aware (see `src-tauri/src/cli/i18n.rs`).

## CLI/TUI Parity

- If you add a new user-facing feature/command, also add a corresponding entry in the interactive TUI flows (`src-tauri/src/cli/interactive/`) and i18n strings (`src-tauri/src/cli/i18n.rs`) so it’s discoverable from the TUI.

## Testing Guidelines

- Tests live in `src-tauri/tests/*.rs` and should avoid touching real user data; follow the existing pattern that isolates `HOME`.
- Prefer focused integration tests that exercise commands/services end-to-end.

## Commit & Pull Request Guidelines

- Follow existing commit patterns (mostly Conventional Commits): `feat: …`, `fix(scope): …`, `refactor: …`, `docs: …`, `chore: …`.
- PRs should include: what changed, why, how to test (`cargo test`), and any user-facing updates (README/CHANGELOG).
- If you change interactive flows or documentation screenshots, update `assets/screenshots/` accordingly.

## Security & Configuration Tips

- Never commit real API keys or personal config files; the app manages data under `~/.cc-switch/` and writes “live” configs for Claude/Codex/Gemini under their respective home directories.
