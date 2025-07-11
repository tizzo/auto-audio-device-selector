use anyhow::Result;
use audio_device_monitor::config::Config;
use audio_device_monitor::AudioDeviceMonitor;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize console logging
    tracing_subscriber::fmt::init();
    
    println!("Audio Device Notification Test");
    println!("==============================");
    println!("This example will be implemented in Phase 2");
    println!("It will demonstrate real-time device change monitoring");
    
    // Load configuration
    let config = Config::default();
    
    // Create monitor
    let monitor = AudioDeviceMonitor::new(config)?;
    
    println!("\nStarting device monitoring...");
    println!("Try plugging/unplugging audio devices to see notifications");
    println!("Press Ctrl+C to stop");
    
    // Start monitoring (basic implementation for now)
    monitor.start().await?;
    
    println!("\nMonitoring stopped");
    Ok(())
}