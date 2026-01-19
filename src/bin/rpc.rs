use discord_rpc::models::*;
use discord_rpc_client::{Client, Event};
use reqwest;
use std::{env, thread, time::Duration};
use std::fs;
use chrono::{Local, TimeZone, Datelike};

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

// Get Sleep Discord Client ID from environment variable
fn get_sleep_discord_client_id() -> u64 {
    env::var("SLEEP_DISCORD_CLIENT_ID")
        .expect("SLEEP_DISCORD_CLIENT_ID must be set in .env file")
        .parse()
        .expect("SLEEP_DISCORD_CLIENT_ID must be a valid u64")
}

// Get Sleep image key from environment variable
fn get_sleep_large_image_key() -> String {
    env::var("SLEEP_DISCORD_LARGE_IMAGE_KEY")
        .expect("SLEEP_DISCORD_LARGE_IMAGE_KEY must be set in .env file")
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

// Check if sleep activity is enabled (default: true)
fn is_sleep_enabled() -> bool {
    env::var("ENABLE_SLEEP")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true"
}

// Get output file paths for OBS (optional)
fn get_obs_steps_file() -> Option<String> {
    env::var("OBS_STEPS_FILE").ok()
}

fn get_obs_water_file() -> Option<String> {
    env::var("OBS_WATER_FILE").ok()
}

fn get_obs_sleep_file() -> Option<String> {
    env::var("OBS_SLEEP_FILE").ok()
}

// Write steps data to text file for OBS
fn write_obs_steps_file(
    steps_data: &StepsSummaryResponse,
    file_path: &str,
) {
    let text = format!(
        "Today: {}\nMonthly: {}\nYearly: {}",
        format_number(steps_data.daily),
        format_number(steps_data.monthly),
        format_number(steps_data.yearly)
    );

    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create directory for OBS steps file: {}", e);
            return;
        }
    }
    
    if let Err(e) = fs::write(file_path, text) {
        eprintln!("Failed to write OBS steps file: {}", e);
    } else {
        println!("✅ OBS steps data written to {}", file_path);
    }
}

// Write water data to text file for OBS
fn write_obs_water_file(
    water_data: &WaterSummaryResponse,
    file_path: &str,
) {
    let text = format!(
        "Today: {}\nMonthly: {}\nYearly: {}",
        water_data.daily_display,
        water_data.monthly_display,
        water_data.yearly_display
    );

    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create directory for OBS water file: {}", e);
            return;
        }
    }
    
    if let Err(e) = fs::write(file_path, text) {
        eprintln!("Failed to write OBS water file: {}", e);
    } else {
        println!("✅ OBS water data written to {}", file_path);
    }
}

// Write sleep data to text file for OBS
fn write_obs_sleep_file(
    sleep_data: &SleepResponse,
    file_path: &str,
) {
    let formatted = format_sleep_minutes(sleep_data.daily_minutes);
    let text = format!("Today: {}", formatted);

    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create directory for OBS sleep file: {}", e);
            return;
        }
    }
    
    if let Err(e) = fs::write(file_path, text) {
        eprintln!("Failed to write OBS sleep file: {}", e);
    } else {
        println!("✅ OBS sleep data written to {}", file_path);
    }
}

// Format minutes as hours and minutes
fn format_sleep_minutes(minutes: i64) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 {
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else {
        format!("{}m", mins)
    }
}

// Get today's date in YYYY-MM-DD format
fn get_today_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

// Calculate total minutes since the start of the year
fn get_minutes_since_year_start() -> i64 {
    let now = Local::now();
    let year_start = Local.with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0)
        .single()
        .expect("Failed to create year start");
    let duration = now.signed_duration_since(year_start);
    duration.num_minutes()
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
    let sleep_discord_client_id = get_sleep_discord_client_id();
    let sleep_large_image_key = get_sleep_large_image_key();
    let steps_enabled = is_steps_enabled();
    let water_enabled = is_water_enabled();
    let sleep_enabled = is_sleep_enabled();
    let obs_steps_file = get_obs_steps_file();
    let obs_water_file = get_obs_water_file();
    let obs_sleep_file = get_obs_sleep_file();

    println!("Connecting to API: {}", api_url);
    println!("Using Steps Discord Client ID: {} (enabled: {})", steps_discord_client_id, steps_enabled);
    println!("Using Water Discord Client ID: {} (enabled: {})", water_discord_client_id, water_enabled);
    println!("Using Sleep Discord Client ID: {} (enabled: {})", sleep_discord_client_id, sleep_enabled);
    if let Some(ref file) = obs_steps_file {
        println!("OBS steps file: {}", file);
    }
    if let Some(ref file) = obs_water_file {
        println!("OBS water file: {}", file);
    }
    if let Some(ref file) = obs_sleep_file {
        println!("OBS sleep file: {}", file);
    }

    // Main loop with reconnection logic - alternate between steps, water, and sleep
    loop {
        // Run all RPC clients, alternating updates
        match run_triple_rpc_clients(
            &api_url,
            &token,
            steps_discord_client_id,
            &steps_large_image_key,
            steps_enabled,
            water_discord_client_id,
            &water_large_image_key,
            water_enabled,
            sleep_discord_client_id,
            &sleep_large_image_key,
            sleep_enabled,
            obs_steps_file.as_deref(),
            obs_water_file.as_deref(),
            obs_sleep_file.as_deref(),
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

fn run_triple_rpc_clients(
    api_url: &str,
    token: &str,
    steps_discord_client_id: u64,
    steps_large_image_key: &str,
    steps_enabled: bool,
    water_discord_client_id: u64,
    water_large_image_key: &str,
    water_enabled: bool,
    sleep_discord_client_id: u64,
    sleep_large_image_key: &str,
    sleep_enabled: bool,
    obs_steps_file: Option<&str>,
    obs_water_file: Option<&str>,
    obs_sleep_file: Option<&str>,
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

    let mut sleep_drpc_opt = if sleep_enabled {
        let mut drpc = Client::new(sleep_discord_client_id);
        drpc.on_ready(|_ctx| {
            println!("Sleep Discord RPC connected!");
        });
        drpc.on_event(Event::Ready, |_ctx| {
            println!("Sleep Discord RPC ready!");
        });
        drpc.start();
        Some(drpc)
    } else {
        println!("Sleep RPC is disabled");
        None
    };

    // Give Discord RPC a moment to connect
    thread::sleep(Duration::from_secs(2));

    // Determine which activity to show first
    // Cycle through: steps -> water -> sleep -> repeat
    // Start with the first enabled activity
    let mut current_activity = if steps_enabled {
        0 // steps
    } else if water_enabled {
        1 // water
    } else if sleep_enabled {
        2 // sleep
    } else {
        // All disabled - just wait
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    };

    // Main loop: cycle through steps -> water -> sleep -> repeat
    loop {
        match current_activity {
            0 if steps_enabled => {
                println!("🔄 Switching to Steps RPC...");
                
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
                        
                        if let Some(file_path) = obs_steps_file {
                            write_obs_steps_file(&summary, file_path);
                        }

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
                                    // Clear other activities
                                    if let Some(ref mut water_drpc) = water_drpc_opt {
                                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                            let _ = water_drpc.clear_activity();
                                        }));
                                    }
                                    if let Some(ref mut sleep_drpc) = sleep_drpc_opt {
                                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                            let _ = sleep_drpc.clear_activity();
                                        }));
                                    }
                                    // Move to next activity
                                    current_activity = get_next_activity(1, steps_enabled, water_enabled, sleep_enabled);
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
                        current_activity = get_next_activity(1, steps_enabled, water_enabled, sleep_enabled);
                    }
                }
            }
            1 if water_enabled => {
                println!("🔄 Switching to Water RPC...");
                
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
                        
                        if let Some(file_path) = obs_water_file {
                            write_obs_water_file(&summary, file_path);
                        }

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
                                    // Clear other activities
                                    if let Some(ref mut steps_drpc) = steps_drpc_opt {
                                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                            let _ = steps_drpc.clear_activity();
                                        }));
                                    }
                                    if let Some(ref mut sleep_drpc) = sleep_drpc_opt {
                                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                            let _ = sleep_drpc.clear_activity();
                                        }));
                                    }
                                    // Move to next activity
                                    current_activity = get_next_activity(2, steps_enabled, water_enabled, sleep_enabled);
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
                        current_activity = get_next_activity(2, steps_enabled, water_enabled, sleep_enabled);
                    }
                }
            }
            2 if sleep_enabled => {
                println!("🔄 Switching to Sleep RPC...");
                
                let today_date = get_today_date();
                match fetch_sleep(api_url, token, &today_date) {
                    Ok(sleep_data) => {
                        let (start_timestamp, end_timestamp) = get_day_timestamps();
                        let daily_formatted = format_sleep_minutes(sleep_data.daily_minutes);
                        let monthly_formatted = format_sleep_minutes(sleep_data.monthly_minutes);
                        let yearly_formatted = format_sleep_minutes(sleep_data.yearly_minutes);
                        
                        // Calculate total minutes since year start
                        let year_minutes = get_minutes_since_year_start();
                        let year_formatted = format_sleep_minutes(year_minutes);
                        
                        let details = format!(
                            "Today: {} | Hours since start of year: {}",
                            daily_formatted,
                            year_formatted
                        );
                        let state = format!(
                            "Monthly: {} | Yearly: {}",
                            monthly_formatted,
                            yearly_formatted
                        );
                        
                        println!(
                            "Fetched sleep - Today: {} ({}m), Monthly: {} ({}m), Yearly: {} ({}m)",
                            daily_formatted, sleep_data.daily_minutes,
                            monthly_formatted, sleep_data.monthly_minutes,
                            yearly_formatted, sleep_data.yearly_minutes
                        );
                        
                        if let Some(file_path) = obs_sleep_file {
                            write_obs_sleep_file(&sleep_data, file_path);
                        }

                        if let Some(ref mut sleep_drpc) = sleep_drpc_opt {
                            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                sleep_drpc.set_activity(|act| {
                                    let mut activity = act.state(state)
                                        .details(&details)
                                        .timestamps(|timestamps| {
                                            timestamps.start(start_timestamp).end(end_timestamp)
                                        });
                                    
                                    if !sleep_large_image_key.is_empty() {
                                        activity = activity.assets(|assets| {
                                            assets.large_image(sleep_large_image_key)
                                                .large_text("Getting rest!")
                                        });
                                    }
                                    
                                    activity
                                })
                            }));

                            match result {
                                Ok(Ok(_)) => {
                                    println!("✅ Sleep activity set successfully");
                                    // Clear other activities
                                    if let Some(ref mut steps_drpc) = steps_drpc_opt {
                                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                            let _ = steps_drpc.clear_activity();
                                        }));
                                    }
                                    if let Some(ref mut water_drpc) = water_drpc_opt {
                                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                            let _ = water_drpc.clear_activity();
                                        }));
                                    }
                                    // Move to next activity (back to steps)
                                    current_activity = get_next_activity(0, steps_enabled, water_enabled, sleep_enabled);
                                }
                                Ok(Err(e)) => {
                                    eprintln!("Failed to set sleep activity: {}", e);
                                    return Err(format!("Sleep Discord RPC connection lost: {}", e).into());
                                }
                                Err(_) => {
                                    eprintln!("Panic caught while setting sleep activity. Reconnecting...");
                                    return Err("Panic in set_activity".into());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching sleep: {}", e);
                        if let Some(ref mut sleep_drpc) = sleep_drpc_opt {
                            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                sleep_drpc.set_activity(|act| {
                                    act.state("Unable to fetch sleep").details("API connection error")
                                })
                            }));
                        }
                        current_activity = get_next_activity(0, steps_enabled, water_enabled, sleep_enabled);
                    }
                }
            }
            _ => {
                // Current activity is disabled or invalid, find next enabled one
                current_activity = get_next_activity(current_activity, steps_enabled, water_enabled, sleep_enabled);
                if !steps_enabled && !water_enabled && !sleep_enabled {
                    // All disabled - just wait
                    thread::sleep(Duration::from_secs(60));
                    continue;
                }
            }
        }

        // Wait 60 seconds before next update
        thread::sleep(Duration::from_secs(60));
    }
}

// Helper function to get next enabled activity (0=steps, 1=water, 2=sleep)
fn get_next_activity(current: usize, steps_enabled: bool, water_enabled: bool, sleep_enabled: bool) -> usize {
    let enabled = [steps_enabled, water_enabled, sleep_enabled];
    
    // Start from next position
    for i in 1..=3 {
        let idx = (current + i) % 3;
        if enabled[idx] {
            return idx;
        }
    }
    
    // Fallback to first enabled
    if steps_enabled { 0 }
    else if water_enabled { 1 }
    else if sleep_enabled { 2 }
    else { 0 }
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

fn fetch_sleep(api_url: &str, token: &str, date: &str) -> Result<SleepResponse, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/api/sleep/summary?token={}&date={}", api_url, token, date);

    let response = client.get(&url).send()?;

    let status = response.status();
    if status.is_success() {
        let sleep: SleepResponse = response.json()?;
        Ok(sleep)
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

