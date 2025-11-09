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
   API_URL=https://steps.wlling.net
   API_TOKEN=your_api_token_here
   DISCORD_CLIENT_ID=1428159322432471223
   DISCORD_LARGE_IMAGE_KEY=man_walking_emoji_copy
   ```

### Environment Variables

- **`API_URL`** (required): The base URL of your step tracking API
- **`API_TOKEN`** (required): Your API authentication token
- **`DISCORD_CLIENT_ID`** (required): Discord application client ID (must be a valid u64)
- **`DISCORD_LARGE_IMAGE_KEY`** (required): Discord Rich Presence large image key

## Usage

Run the application:
```bash
cargo run
```

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

