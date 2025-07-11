use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber;

mod config;
mod audio;
mod system;
mod priority;

use config::Config;
use audio::AudioDeviceMonitor;

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
    
    // This will be implemented with cpal device enumeration
    println!("Available audio devices:");
    println!("  [Implementation pending - Phase 1]");
    
    if verbose {
        println!("  Verbose mode enabled - will show detailed device info");
    }
    
    Ok(())
}

async fn test_monitor() -> Result<()> {
    info!("Starting device monitor test");
    
    println!("Testing device change monitoring...");
    println!("  [Implementation pending - Phase 2]");
    println!("  Press Ctrl+C to stop");
    
    // This will be implemented with CoreAudio property listeners
    tokio::signal::ctrl_c().await?;
    println!("Monitor test stopped");
    
    Ok(())
}

async fn run_daemon(config: Config) -> Result<()> {
    info!("Starting daemon mode");
    
    let _monitor = AudioDeviceMonitor::new(config)?;
    
    println!("Audio device monitor daemon started");
    println!("  [Implementation pending - Phase 4]");
    println!("  Press Ctrl+C to stop");
    
    // This will be the main daemon loop
    tokio::signal::ctrl_c().await?;
    println!("Daemon stopped");
    
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
    
    println!("Current default devices:");
    println!("  [Implementation pending - Phase 1]");
    
    Ok(())
}