# Step Tracker RPC

Step Tracker RPC is a Rust project with two binaries:

- `rpc`: console process that polls your API and updates Discord Rich Presence
- `gui`: desktop controller for editing config and starting/stopping `rpc`

The Rich Presence rotates between enabled activities:

- Steps
- Water
- Sleep

## Project layout

- `src/bin/rpc.rs`: polling, formatting, Discord RPC updates, OBS text file writes
- `src/bin/gui.rs`: desktop UI, tray integration, `.env` management, process control for `rpc`
- `src/models.rs`: shared API DTOs
- `.github/workflows/ci.yml`: cross-OS CI test workflow

## Requirements

- Rust stable toolchain
- Discord desktop app running locally
- API token for your step tracker backend

For GUI builds on Linux, native packages are required (GTK/AppIndicator/XDo/OpenSSL dev packages).

## Required system packages

`rpc` only:

- No extra native desktop packages are required beyond Rust toolchain/runtime.

`gui` on Linux:

- Requires native GUI/linker dependencies in addition to Rust.

Ubuntu/Debian package set (matches CI):

```bash
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libglib2.0-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  libxdo-dev \
  libssl-dev
```

Fedora package set (matches CI):

```bash
sudo dnf install -y \
  git \
  rust \
  cargo \
  pkgconf-pkg-config \
  glib2-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  libxdo-devel \
  openssl-devel
```

Notes:

- `libxdo-*` resolves linker errors like `unable to find library -lxdo`.
- `openssl*-devel` resolves `openssl-sys` discovery/build errors.
- macOS/Windows usually do not need extra package-manager installs for this project beyond standard Rust toolchains.

## Build and run

Run console RPC:

```bash
cargo run --bin rpc
```

Run GUI:

```bash
cargo run --features gui --bin gui
```

Build release binaries:

```bash
cargo build --release --bin rpc
cargo build --release --features gui --bin gui
```

## Configuration

### `rpc` config source

`rpc` loads environment variables with `dotenv` (from the process working directory).

### `gui` config file location

`gui` reads/writes a `.env` at:

- Project dirs path: `com/ThisisLogan/StepTrackerRPC/.env` (OS-specific config directory)
- Fallback: current working directory `.env`

You can edit and save values directly in the GUI.

### Environment variables

Required:

- `API_URL`
- `API_TOKEN`

Feature toggles (default `true`):

- `ENABLE_STEPS`
- `ENABLE_WATER`
- `ENABLE_SLEEP`

Discord app config:

- `STEPS_DISCORD_CLIENT_ID`
- `STEPS_DISCORD_LARGE_IMAGE_KEY`
- `WATER_DISCORD_CLIENT_ID`
- `WATER_DISCORD_LARGE_IMAGE_KEY`
- `SLEEP_DISCORD_CLIENT_ID`
- `SLEEP_DISCORD_LARGE_IMAGE_KEY`

Optional OBS/text output files:

- `OBS_STEPS_FILE`
- `OBS_WATER_FILE`
- `OBS_SLEEP_FILE`

Optional GUI/tray behavior:

- `GUI_DISABLE_TRAY=true` to force no-tray mode

## API expectations

`rpc` calls:

- `GET {API_URL}/api/steps/summary?token={API_TOKEN}`
- `GET {API_URL}/api/water/summary?token={API_TOKEN}`
- `GET {API_URL}/api/sleep/summary?token={API_TOKEN}&date={YYYY-MM-DD}`

Response models are defined in `src/models.rs`:

- `StepsSummaryResponse`
- `WaterSummaryResponse`
- `SleepResponse`
- `ErrorResponse`

## Runtime behavior

- Creates one Discord RPC client per enabled activity.
- Cycles display in order: Steps -> Water -> Sleep (skips disabled entries).
- Sleeps for 30 seconds after each activity update attempt.
- Reconnects Discord RPC clients on connection/panic signals.
- If all activities are disabled, it idles and sleeps.

## OBS file output

If configured, the process writes plain text files:

- Steps: today/monthly/yearly (abbreviated numbers)
- Water: today/monthly/yearly display strings from API
- Sleep: today value formatted as `xh ym`

Parent directories are created automatically.

## GUI behavior

- Load/save `.env`
- Start/stop `rpc` child process
- Show process logs in-app
- Tray icon/menu (best effort)
  - Close window hides to tray when tray is available
  - If tray is unavailable, close exits and stops RPC cleanly

## Testing

Current tests include:

- `src/bin/rpc.rs`
  - `format_sleep_minutes`
  - `format_number`
- `src/bin/gui.rs`
  - bool parsing
  - env value encoding
  - env read/write round-trip
  - no-tray GUI startup smoke test

Run tests locally:

```bash
cargo test --bin rpc
cargo test --features gui --bin gui
```

## CI

GitHub Actions workflow `.github/workflows/ci.yml` runs a single matrix card:

- `ubuntu-latest`
- `macos-latest`
- `windows-latest`
- `fedora-latest` (via Docker on Ubuntu runner)

Each entry runs:

- `cargo check --bin rpc`
- `cargo test --bin rpc`
- `cargo test --features gui --bin gui`

Linux entries install required system packages before GUI tests.

## License

See [LICENSE](LICENSE).
