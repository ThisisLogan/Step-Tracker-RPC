use discord_rpc::models::*;
use discord_rpc_client::{Client, Event};
use reqwest;
use std::{env, thread, time::Duration};
use chrono::{Local, TimeZone};

// Get API URL from environment variable
fn get_api_url() -> String {
    env::var("API_URL")
        .expect("API_URL must be set in .env file")
}

// Get Discord Client ID from environment variable or use default
fn get_discord_client_id() -> u64 {
    env::var("DISCORD_CLIENT_ID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1428159322432471223)
}

// Get image key from environment variable, default to configured value
fn get_large_image_key() -> String {
    env::var("DISCORD_LARGE_IMAGE_KEY").unwrap_or_else(|_| "man_walking_emoji_copy".to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Set a panic hook to catch panics from Discord RPC background threads
    // Note: This won't prevent crashes in background threads, but will log them
    std::panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            format!("{:?}", panic_info)
        };
        
        if message.contains("Socket is not connected") || message.contains("NotConnected") {
            eprintln!("⚠️  Discord RPC connection lost (socket not connected). This is usually harmless and will be handled by reconnection logic.");
        } else {
            eprintln!("⚠️  Panic in Discord RPC: {}", message);
        }
    }));

    // Get token from environment variable
    let token = env::var("API_TOKEN")
        .expect("API_TOKEN must be set in .env file");

    // Get configuration from environment
    let api_url = get_api_url();
    let discord_client_id = get_discord_client_id();
    let large_image_key = get_large_image_key();

    println!("Connecting to API: {}", api_url);
    println!("Using Discord Client ID: {}", discord_client_id);

    // Main loop with reconnection logic
    loop {
        match run_rpc_client(&api_url, &token, discord_client_id, &large_image_key) {
            Ok(_) => {
                eprintln!("Discord RPC client exited normally. Restarting in 5 seconds...");
                thread::sleep(Duration::from_secs(5));
            }
            Err(e) => {
                eprintln!("Discord RPC client error: {}. Restarting in 5 seconds...", e);
                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

fn run_rpc_client(
    api_url: &str,
    token: &str,
    discord_client_id: u64,
    large_image_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create Discord RPC client
    let mut drpc = Client::new(discord_client_id);

    // Register event handlers
    drpc.on_ready(|_ctx| {
        println!("Discord RPC connected!");
    });

    drpc.on_event(Event::Ready, |_ctx| {
        println!("Discord RPC ready!");
    });

    // Start up the client connection
    drpc.start();

    // Give Discord RPC a moment to connect
    thread::sleep(Duration::from_secs(2));

    // Main loop: fetch steps and update RPC status
    loop {
        match fetch_steps_summary(api_url, token) {
            Ok(summary) => {
                let (start_timestamp, end_timestamp) = get_day_timestamps();
                let details = format!("Today: {}", format_number(summary.daily));
                let state = format!(
                    "Monthly: {} | Yearly: {}",
                    format_number(summary.monthly),
                    format_number(summary.yearly)
                );
                
                println!(
                    "Fetched steps - Today: {}, Monthly: {}, Yearly: {}",
                    summary.daily, summary.monthly, summary.yearly
                );

                // Try to set activity, but don't crash if it fails
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    drpc.set_activity(|act| {
                        let mut activity = act.state(&state)
                            .details(&details)
                            .timestamps(|timestamps| {
                                timestamps.start(start_timestamp).end(end_timestamp)
                            });
                        
                        // Add image if configured
                        if !large_image_key.is_empty() {
                            activity = activity.assets(|assets| {
                                assets.large_image(large_image_key)
                                    .large_text("I'm walking here!")
                            });
                        }
                        
                        activity
                    })
                }));

                match result {
                    Ok(Ok(_)) => {
                        // Success
                    }
                    Ok(Err(e)) => {
                        eprintln!("Failed to set activity: {}", e);
                        // If we can't set activity, the connection might be lost
                        // Return error to trigger reconnection
                        return Err(format!("Discord RPC connection lost: {}", e).into());
                    }
                    Err(_) => {
                        eprintln!("Panic caught while setting activity. Reconnecting...");
                        return Err("Panic in set_activity".into());
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching steps: {}", e);
                // Try to set error state, but don't crash if it fails
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    drpc.set_activity(|act| {
                        act.state("Unable to fetch steps").details("API connection error")
                    })
                }));
            }
        }

        // Update every 30 seconds
        thread::sleep(Duration::from_secs(30));
    }
}

fn fetch_steps_summary(api_url: &str, token: &str) -> Result<StepsSummaryResponse, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/api/steps/summary?token={}", api_url, token);

    let response = client.get(&url).send()?;

    let status = response.status();
    if status.is_success() {
        let summary: StepsSummaryResponse = response.json()?;
        Ok(summary)
    } else {
        // Try to parse error response, fallback to status code
        let error_msg = if let Ok(error_response) = response.json::<ErrorResponse>() {
            error_response.error
        } else {
            format!("HTTP {} {}", status, status.canonical_reason().unwrap_or("Unknown"))
        };
        Err(error_msg.into())
    }
}

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

fn get_day_timestamps() -> (u64, u64) {
    let now = Local::now();
    let today = now.date_naive();
    
    // Start timestamp: today at 00:00:00 (midnight start of day)
    let today_start = Local.from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .expect("Failed to create today midnight");
    let start_timestamp = today_start.timestamp() as u64;
    
    // End timestamp: today at 23:59:59 (end of day)
    // Since this runs every 30 seconds, if we're past 23:59:59, 
    // now.date_naive() will already be the next day, so this will be correct
    let today_end = Local.from_local_datetime(&today.and_hms_opt(23, 59, 59).unwrap())
        .single()
        .expect("Failed to create today end");
    let end_timestamp = today_end.timestamp() as u64;
    
    (start_timestamp, end_timestamp)
}

