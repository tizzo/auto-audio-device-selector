use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber;

mod audio;
mod config;
mod priority;
mod system;

use audio::AudioDeviceMonitor;
use config::Config;

#[derive(Parser)]
#[command(name = "audio-device-monitor")]
#[command(about = "macOS audio device monitor with priority-based automatic switching")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available audio devices
    ListDevices {
        /// Show detailed device information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Test device monitoring (prints device changes)
    TestMonitor,
    /// Run in daemon mode
    Daemon,
    /// Validate configuration file
    CheckConfig,
    /// Show current default devices
    ShowDefault,
    /// Switch to a specific device
    Switch {
        /// Device name to switch to
        #[arg(short, long)]
        device: String,
        /// Switch input device instead of output
        #[arg(short, long)]
        input: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("audio_device_monitor={}", log_level))
        .init();

    info!("Starting audio device monitor");

    // Load configuration
    let config = Config::load(cli.config.as_deref())?;
    info!("Configuration loaded successfully");

    // Handle commands
    match cli.command {
        Some(Commands::ListDevices { verbose }) => {
            list_devices(verbose).await?;
        }
        Some(Commands::TestMonitor) => {
            test_monitor().await?;
        }
        Some(Commands::Daemon) => {
            run_daemon(config).await?;
        }
        Some(Commands::CheckConfig) => {
            check_config(&config)?;
        }
        Some(Commands::ShowDefault) => {
            show_default_devices().await?;
        }
        Some(Commands::Switch { device, input }) => {
            switch_device(&device, input).await?;
        }
        None => {
            // Default behavior - run daemon if no command specified
            info!("No command specified, running in daemon mode");
            run_daemon(config).await?;
        }
    }

    Ok(())
}

async fn list_devices(verbose: bool) -> Result<()> {
    info!("Listing audio devices");

    let controller = audio::controller::DeviceController::new()?;
    let devices = controller.enumerate_devices()?;

    println!("Available audio devices:");
    if devices.is_empty() {
        println!("  No audio devices found!");
        return Ok(());
    }

    for (i, device) in devices.iter().enumerate() {
        println!("  {}. {}", i + 1, device);
    }

    // Show default devices
    if let Ok(Some(default_input)) = controller.get_default_input_device() {
        println!("Default input: {}", default_input.name);
    }

    if let Ok(Some(default_output)) = controller.get_default_output_device() {
        println!("Default output: {}", default_output.name);
    }

    if verbose {
        println!("\n--- Detailed Device Information ---");
        for device in &devices {
            if let Ok(info) = controller.get_device_info(device) {
                println!("Device: {}", info.name);
                println!("  UID: {}", info.uid);
                println!("  Type: {}", info.device_type);
                println!("  Default: {}", info.is_default);
                println!();
            }
        }
    }

    Ok(())
}

async fn test_monitor() -> Result<()> {
    info!("Starting device monitor test");

    println!("Testing device change monitoring...");

    // Load configuration and create monitor
    let config = Config::load(None)?;
    let monitor = AudioDeviceMonitor::new(config)?;

    // Start monitoring in async mode
    monitor.start_monitoring_async().await?;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("Monitor test stopped");
    monitor.stop()?;

    Ok(())
}

async fn run_daemon(config: Config) -> Result<()> {
    info!("Starting daemon mode");

    let monitor = AudioDeviceMonitor::new(config)?;

    println!("Audio device monitor daemon started");
    println!("  Real-time device monitoring active");
    println!("  Press Ctrl+C to stop");

    // Start monitoring in async mode
    monitor.start_monitoring_async().await?;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("Daemon stopped");
    monitor.stop()?;

    Ok(())
}

fn check_config(config: &Config) -> Result<()> {
    info!("Validating configuration");

    println!("Configuration validation:");
    println!("  ✓ Configuration file parsed successfully");
    println!("  ✓ Output devices: {}", config.output_devices.len());
    println!("  ✓ Input devices: {}", config.input_devices.len());

    // Additional validation will be added as we implement more features

    Ok(())
}

async fn show_default_devices() -> Result<()> {
    info!("Showing current default devices");

    let controller = audio::controller::DeviceController::new()?;

    println!("Current default devices:");

    if let Ok(Some(default_input)) = controller.get_default_input_device() {
        println!("  Input:  {}", default_input);
    } else {
        println!("  Input:  None available");
    }

    if let Ok(Some(default_output)) = controller.get_default_output_device() {
        println!("  Output: {}", default_output);
    } else {
        println!("  Output: None available");
    }

    Ok(())
}

async fn switch_device(device_name: &str, is_input: bool) -> Result<()> {
    info!(
        "Manual device switch requested: {} ({})",
        device_name,
        if is_input { "input" } else { "output" }
    );

    let controller = audio::controller::DeviceController::new()?;

    println!(
        "Switching {} device to: {}",
        if is_input { "input" } else { "output" },
        device_name
    );

    let result = if is_input {
        controller.set_default_input_device(device_name)
    } else {
        controller.set_default_output_device(device_name)
    };

    match result {
        Ok(()) => {
            println!(
                "✓ Successfully switched {} device to: {}",
                if is_input { "input" } else { "output" },
                device_name
            );
        }
        Err(e) => {
            println!("✗ Failed to switch device: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
