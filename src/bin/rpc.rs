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

// Get Steps Discord Client ID from environment variable
fn get_steps_discord_client_id() -> u64 {
    env::var("STEPS_DISCORD_CLIENT_ID")
        .expect("STEPS_DISCORD_CLIENT_ID must be set in .env file")
        .parse()
        .expect("STEPS_DISCORD_CLIENT_ID must be a valid u64")
}

// Get Steps image key from environment variable
fn get_steps_large_image_key() -> String {
    env::var("STEPS_DISCORD_LARGE_IMAGE_KEY")
        .expect("STEPS_DISCORD_LARGE_IMAGE_KEY must be set in .env file")
}

// Get Water Discord Client ID from environment variable
fn get_water_discord_client_id() -> u64 {
    env::var("WATER_DISCORD_CLIENT_ID")
        .expect("WATER_DISCORD_CLIENT_ID must be set in .env file")
        .parse()
        .expect("WATER_DISCORD_CLIENT_ID must be a valid u64")
}

// Get Water image key from environment variable
fn get_water_large_image_key() -> String {
    env::var("WATER_DISCORD_LARGE_IMAGE_KEY")
        .expect("WATER_DISCORD_LARGE_IMAGE_KEY must be set in .env file")
}

// Check if steps activity is enabled (default: true)
fn is_steps_enabled() -> bool {
    env::var("ENABLE_STEPS")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true"
}

// Check if water activity is enabled (default: true)
fn is_water_enabled() -> bool {
    env::var("ENABLE_WATER")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true"
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
    let steps_discord_client_id = get_steps_discord_client_id();
    let steps_large_image_key = get_steps_large_image_key();
    let water_discord_client_id = get_water_discord_client_id();
    let water_large_image_key = get_water_large_image_key();
    let steps_enabled = is_steps_enabled();
    let water_enabled = is_water_enabled();

    println!("Connecting to API: {}", api_url);
    println!("Using Steps Discord Client ID: {} (enabled: {})", steps_discord_client_id, steps_enabled);
    println!("Using Water Discord Client ID: {} (enabled: {})", water_discord_client_id, water_enabled);

    // Main loop with reconnection logic - alternate between steps and water
    loop {
        // Run both RPC clients, alternating updates
        match run_dual_rpc_clients(
            &api_url,
            &token,
            steps_discord_client_id,
            &steps_large_image_key,
            steps_enabled,
            water_discord_client_id,
            &water_large_image_key,
            water_enabled,
        ) {
            Ok(_) => {
                eprintln!("RPC clients exited normally. Restarting in 5 seconds...");
                thread::sleep(Duration::from_secs(5));
            }
            Err(e) => {
                eprintln!("RPC client error: {}. Restarting in 5 seconds...", e);
                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

fn run_dual_rpc_clients(
    api_url: &str,
    token: &str,
    steps_discord_client_id: u64,
    steps_large_image_key: &str,
    steps_enabled: bool,
    water_discord_client_id: u64,
    water_large_image_key: &str,
    water_enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Only create clients if they're enabled
    let mut steps_drpc_opt = if steps_enabled {
        let mut drpc = Client::new(steps_discord_client_id);
        drpc.on_ready(|_ctx| {
            println!("Steps Discord RPC connected!");
        });
        drpc.on_event(Event::Ready, |_ctx| {
            println!("Steps Discord RPC ready!");
        });
        drpc.start();
        Some(drpc)
    } else {
        println!("Steps RPC is disabled");
        None
    };

    let mut water_drpc_opt = if water_enabled {
        let mut drpc = Client::new(water_discord_client_id);
        drpc.on_ready(|_ctx| {
            println!("Water Discord RPC connected!");
        });
        drpc.on_event(Event::Ready, |_ctx| {
            println!("Water Discord RPC ready!");
        });
        drpc.start();
        Some(drpc)
    } else {
        println!("Water RPC is disabled");
        None
    };

    // Give Discord RPC a moment to connect
    thread::sleep(Duration::from_secs(2));

    // Determine which activity to show first
    // If only one is enabled, always show that one
    // If both are enabled, start with steps
    let mut show_steps = if steps_enabled && !water_enabled {
        true // Only steps enabled
    } else if water_enabled && !steps_enabled {
        false // Only water enabled
    } else {
        true // Both enabled, start with steps
    };

    // Main loop: alternate between updating steps and water
    loop {
        if show_steps && steps_enabled {
            println!("🔄 Switching to Steps RPC...");
            
            // Set steps activity first (before clearing water)
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

                    if let Some(ref mut steps_drpc) = steps_drpc_opt {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            steps_drpc.set_activity(|act| {
                                let mut activity = act.state(&state)
                                    .details(&details)
                                    .timestamps(|timestamps| {
                                        timestamps.start(start_timestamp).end(end_timestamp)
                                    });
                                
                                if !steps_large_image_key.is_empty() {
                                    activity = activity.assets(|assets| {
                                        assets.large_image(steps_large_image_key)
                                            .large_text("I'm walking here!")
                                    });
                                }
                                
                                activity
                            })
                        }));

                        match result {
                            Ok(Ok(_)) => {
                                println!("✅ Steps activity set successfully");
                                // Now clear water activity after setting steps
                                if water_enabled {
                                    if let Some(ref mut water_drpc) = water_drpc_opt {
                                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                            let _ = water_drpc.clear_activity();
                                        }));
                                    }
                                }
                                // Switch to water next only if water is enabled
                                if water_enabled {
                                    show_steps = false;
                                }
                                // If water is disabled, stay on steps
                            }
                            Ok(Err(e)) => {
                                eprintln!("Failed to set steps activity: {}", e);
                                return Err(format!("Steps Discord RPC connection lost: {}", e).into());
                            }
                            Err(_) => {
                                eprintln!("Panic caught while setting steps activity. Reconnecting...");
                                return Err("Panic in set_activity".into());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching steps: {}", e);
                    if let Some(ref mut steps_drpc) = steps_drpc_opt {
                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            steps_drpc.set_activity(|act| {
                                act.state("Unable to fetch steps").details("API connection error")
                            })
                        }));
                    }
                    // Switch to water even on error, but only if water is enabled
                    if water_enabled {
                        show_steps = false;
                    }
                }
            }
        } else if !show_steps && water_enabled {
            println!("🔄 Switching to Water RPC...");
            
            // Set water activity first (before clearing steps)
            match fetch_water_summary(api_url, token) {
                Ok(summary) => {
                    let (start_timestamp, end_timestamp) = get_day_timestamps();
                    let details = format!("Today: {}", summary.daily_display);
                    let state = format!(
                        "Monthly: {} | Yearly: {}",
                        summary.monthly_display,
                        summary.yearly_display
                    );
                    
                    println!(
                        "Fetched water - Today: {}, Monthly: {}, Yearly: {}",
                        summary.daily_display, summary.monthly_display, summary.yearly_display
                    );

                    if let Some(ref mut water_drpc) = water_drpc_opt {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            water_drpc.set_activity(|act| {
                                let mut activity = act.state(&state)
                                    .details(&details)
                                    .timestamps(|timestamps| {
                                        timestamps.start(start_timestamp).end(end_timestamp)
                                    });
                                
                                if !water_large_image_key.is_empty() {
                                    activity = activity.assets(|assets| {
                                        assets.large_image(water_large_image_key)
                                            .large_text("Staying hydrated!")
                                    });
                                }
                                
                                activity
                            })
                        }));

                        match result {
                            Ok(Ok(_)) => {
                                println!("✅ Water activity set successfully");
                                // Now clear steps activity after setting water
                                if steps_enabled {
                                    if let Some(ref mut steps_drpc) = steps_drpc_opt {
                                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                            let _ = steps_drpc.clear_activity();
                                        }));
                                    }
                                }
                                // Switch to steps next only if steps is enabled
                                if steps_enabled {
                                    show_steps = true;
                                }
                                // If steps is disabled, stay on water
                            }
                            Ok(Err(e)) => {
                                eprintln!("Failed to set water activity: {}", e);
                                return Err(format!("Water Discord RPC connection lost: {}", e).into());
                            }
                            Err(_) => {
                                eprintln!("Panic caught while setting water activity. Reconnecting...");
                                return Err("Panic in set_activity".into());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching water: {}", e);
                    if let Some(ref mut water_drpc) = water_drpc_opt {
                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            water_drpc.set_activity(|act| {
                                act.state("Unable to fetch water").details("API connection error")
                            })
                        }));
                    }
                    // Switch to steps even on error, but only if steps is enabled
                    if steps_enabled {
                        show_steps = true;
                    }
                }
            }
        } else {
            // Both disabled or invalid state - just wait
            thread::sleep(Duration::from_secs(60));
            continue;
        }

        // Wait 60 seconds before next update
        thread::sleep(Duration::from_secs(60));
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

fn fetch_water_summary(api_url: &str, token: &str) -> Result<WaterSummaryResponse, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/api/water/summary?token={}", api_url, token);

    let response = client.get(&url).send()?;

    let status = response.status();
    if status.is_success() {
        let summary: WaterSummaryResponse = response.json()?;
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
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
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

