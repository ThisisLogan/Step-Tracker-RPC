# AGENTS.md

## Tech Stack
- Language: Rust (edition 2021).
- Binaries: `rpc` (console Rich Presence client) and `gui` (desktop GUI + tray controller).
- Crates: `discord-rpc-client`, `reqwest` (blocking + JSON), `serde`, `serde_json`, `chrono`, `dotenv`, `eframe`/`egui`, `tray-icon`, `anyhow`, `directories`.

## Coding Standards
- Style: Standard Rust 2021 conventions with modules in `src/` and binaries in `src/bin/`.
- Error handling: `rpc` uses `Result<Box<dyn Error>>` plus `expect`/`unwrap` for required config; `gui` uses `anyhow::Result` with context.
- Configuration: Environment-first via `.env` (loaded by `dotenv`), required keys enforced, feature toggles default to `true`.
- Formatting/linting: No explicit `rustfmt` or `clippy` config found; defaults are assumed if used.

## Architectural Patterns
- Two-process model: `rpc` handles polling + Discord Rich Presence; `gui` edits config and starts/stops `rpc`.
- Shared data models: API DTOs live in `src/models.rs` and are re-exported via `src/lib.rs`.
- Polling workflow: Periodic HTTP polling (every 30 seconds per README) to fetch steps/water/sleep summaries and update presence.
- Optional side effects: Writes text files for OBS if `OBS_*_FILE` env vars are set.
- Resilience: Reconnection logic for Discord RPC plus a panic hook to log background thread failures.
