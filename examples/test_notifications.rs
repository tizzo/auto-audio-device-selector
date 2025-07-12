use anyhow::Result;
use audio_device_monitor::AudioDeviceMonitor;
use audio_device_monitor::config::Config;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize console logging
    tracing_subscriber::fmt::init();

    println!("Audio Device Notification Test");
    println!("==============================");

    // Load configuration
    let config = Config::default();

    // Create monitor
    let monitor = AudioDeviceMonitor::new(config)?;

    println!("\nStarting device monitoring...");
    println!("Monitoring for 10 seconds to test functionality");

    // Start monitoring in async mode
    monitor.start_monitoring_async().await?;

    // Wait for 10 seconds to see some notifications
    tokio::time::sleep(Duration::from_secs(10)).await;

    println!("\nStopping monitoring...");
    monitor.stop()?;

    println!("Monitoring test complete!");
    Ok(())
}
