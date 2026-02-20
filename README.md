# Step Tracker RPC

A Discord Rich Presence application that displays your daily, monthly, and yearly step counts in real-time on your Discord profile.

## Features

- 📊 Real-time step count display on Discord
- 📅 Daily, monthly, and yearly step tracking
- 🔄 Automatic updates every 30 seconds
- 🔁 Automatic reconnection on connection loss
- ⚙️ Configurable via environment variables

## Prerequisites

- Rust (latest stable version recommended)
- Discord desktop app (for Rich Presence to work)
- API access token from your step tracking service

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/ThisisLogan/Step-Tracker-RPC.git
   cd Step-Tracker-RPC
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Configuration

1. Create a `.env` file in the project root:
   ```bash
   cp .env.example .env
   ```

2. Edit the `.env` file with your configuration:
   ```env
   # Required
   API_URL=https://steps.wlling.net
   API_TOKEN=your_api_token_here

   # Enable/disable each Rich Presence (defaults: true)
   ENABLE_STEPS=true
   ENABLE_WATER=true
   ENABLE_SLEEP=true

   # Steps RPC (required if ENABLE_STEPS=true)
   STEPS_DISCORD_CLIENT_ID=1428159322432471223
   STEPS_DISCORD_LARGE_IMAGE_KEY=man_walking_emoji_copy

   # Water RPC (required if ENABLE_WATER=true)
   WATER_DISCORD_CLIENT_ID=123456789012345678
   WATER_DISCORD_LARGE_IMAGE_KEY=water_icon_key

   # Sleep RPC (required if ENABLE_SLEEP=true)
   SLEEP_DISCORD_CLIENT_ID=123456789012345678
   SLEEP_DISCORD_LARGE_IMAGE_KEY=sleep_icon_key

   # Optional: write output files for OBS/text sources
   # OBS_STEPS_FILE=/path/to/steps.txt
   # OBS_WATER_FILE=/path/to/water.txt
   # OBS_SLEEP_FILE=/path/to/sleep.txt
   ```

### Environment Variables

- **`API_URL`** (required): The base URL of your step tracking API
- **`API_TOKEN`** (required): Your API authentication token
- **`ENABLE_STEPS`** (optional): `true`/`false` (default: `true`)
- **`ENABLE_WATER`** (optional): `true`/`false` (default: `true`)
- **`ENABLE_SLEEP`** (optional): `true`/`false` (default: `true`)
- **`STEPS_DISCORD_CLIENT_ID`**: Discord application client ID for steps (u64)
- **`STEPS_DISCORD_LARGE_IMAGE_KEY`**: Steps large image key
- **`WATER_DISCORD_CLIENT_ID`**: Discord application client ID for water (u64)
- **`WATER_DISCORD_LARGE_IMAGE_KEY`**: Water large image key
- **`SLEEP_DISCORD_CLIENT_ID`**: Discord application client ID for sleep (u64)
- **`SLEEP_DISCORD_LARGE_IMAGE_KEY`**: Sleep large image key
- **`OBS_STEPS_FILE`** (optional): path to write steps text output for OBS
- **`OBS_WATER_FILE`** (optional): path to write water text output for OBS
- **`OBS_SLEEP_FILE`** (optional): path to write sleep text output for OBS

## Usage

Run the RPC (console):
```bash
cargo run --bin rpc
```

Run the GUI (cross-platform):
```bash
cargo run --bin gui
```

The GUI can:
- Edit and save a local `.env`
- Start/stop the `rpc` process
- Hide to run in the background (closing the window hides it; use the tray icon menu to show/quit)

The application will:
1. Connect to your Discord client
2. Fetch step data from the API every 30 seconds
3. Update your Discord Rich Presence status with:
   - **Details**: Today's step count
   - **State**: Monthly and yearly step counts
   - **Timestamps**: Start and end of the current day

## How It Works

The application:
- Connects to Discord via the Discord RPC protocol
- Periodically fetches step summary data from your configured API endpoint
- Formats and displays the data in your Discord profile
- Automatically reconnects if the connection is lost

## API Endpoint

The application expects an API endpoint at:
```
GET {API_URL}/api/steps/summary?token={API_TOKEN}
```

The endpoint should return JSON in the following format:
```json
{
  "daily": 12345,
  "monthly": 234567,
  "yearly": 1234567
}
```

## Troubleshooting

### Discord Rich Presence not showing
- Make sure Discord desktop app is running (not the web version)
- Check that your Discord client ID is correct
- Verify the Discord RPC connection in the console output

### API connection errors
- Verify your `API_URL` and `API_TOKEN` are correct in the `.env` file
- Check that the API endpoint is accessible
- Review error messages in the console output

### Connection lost errors
- The application will automatically attempt to reconnect
- If issues persist, restart the application

## License

See [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

