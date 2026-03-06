# AGENTS.md

## Tech Stack
- Language: Rust (edition 2021).
- Binaries: `rpc` (console Rich Presence client) and `gui` (desktop GUI + tray controller).
- Core crates: `discord-rpc-client`, `reqwest` (blocking + JSON), `serde`, `serde_json`, `chrono`, `dotenv`.
- GUI crates (feature-gated): `eframe`/`egui`, `tray-icon`, `directories`, `anyhow`.
- Utility crate: `once_cell` (used in tests).

## Build and Features
- `gui` binary is feature-gated via `required-features = ["gui"]`.
- `Cargo.toml` feature map:
  - `default = []`
  - `gui = ["dep:eframe", "dep:tray-icon", "dep:directories"]`
- Non-GUI builds (`rpc`) avoid desktop-native dependencies.

## Coding Standards
- Style: Standard Rust 2021 conventions with modules in `src/` and binaries in `src/bin/`.
- Error handling:
  - `rpc`: `Result<Box<dyn Error>>`, explicit fallbacks, and some `expect` for required config.
  - `gui`: `anyhow::Result` + context for IO/process errors.
- Configuration:
  - `rpc` uses environment variables loaded through `dotenv`.
  - `gui` reads/writes a `.env` in OS project config dir (`ProjectDirs`) with cwd fallback.
- Formatting/linting: No custom rustfmt/clippy config present.

## Architecture
- Two-process model:
  - `rpc` handles polling APIs and publishing Discord Rich Presence.
  - `gui` edits config and controls lifecycle of `rpc` as a child process.
- Shared API DTOs in `src/models.rs`, re-exported by `src/lib.rs`.
- Multi-activity rotation in `rpc`:
  - Steps, Water, and Sleep each have their own Discord RPC client.
  - Rotation order: Steps -> Water -> Sleep; disabled activities are skipped.
  - 30-second sleep between activity update attempts.
- Side effects:
  - Optional OBS text file output via `OBS_*_FILE` env vars.
- Resilience:
  - Panic hook + reconnect flag for Discord RPC background thread failures.
  - Tray backend fallback in GUI (`RealTrayBackend` -> `NoTrayBackend`).

## API Contracts
- `GET {API_URL}/api/steps/summary?token={API_TOKEN}` -> `StepsSummaryResponse`
- `GET {API_URL}/api/water/summary?token={API_TOKEN}` -> `WaterSummaryResponse`
- `GET {API_URL}/api/sleep/summary?token={API_TOKEN}&date={YYYY-MM-DD}` -> `SleepResponse`
- Error body fallback model: `ErrorResponse`

## Runtime Notes
- Required env vars: `API_URL`, `API_TOKEN`, and per-activity Discord IDs/image keys if enabled.
- Enable flags default to true: `ENABLE_STEPS`, `ENABLE_WATER`, `ENABLE_SLEEP`.
- `GUI_DISABLE_TRAY=true` forces no-tray mode.
- Linux GUI native deps:
  - Ubuntu CI installs: `pkg-config`, `libglib2.0-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `libxdo-dev`, `libssl-dev`.
  - Fedora CI installs: `pkgconf-pkg-config`, `glib2-devel`, `gtk3-devel`, `libappindicator-gtk3-devel`, `libxdo-devel`, `openssl-devel`.
- GUI close behavior:
  - Tray available: close hides window.
  - Tray unavailable: close exits and stops RPC.

## Testing and CI
- Unit/smoke tests currently live in binary files:
  - `src/bin/rpc.rs`: number/sleep formatting tests.
  - `src/bin/gui.rs`: env parsing/encoding tests + no-tray startup smoke test.
- CI workflow: `.github/workflows/ci.yml`.
  - Single `test-matrix` job with labels: Ubuntu, macOS, Windows, Fedora.
  - Fedora is executed inside a `fedora:latest` Docker container on an Ubuntu runner.
  - All variants run `cargo check --bin rpc`, `cargo test --bin rpc`, and `cargo test --features gui --bin gui`.
  - Linux variants install native dependencies for GUI linking.
