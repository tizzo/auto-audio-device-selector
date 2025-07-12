use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod audio;
mod config;
mod logging;
mod priority;
mod service;
mod system;

use audio::AudioDeviceMonitor;
use config::Config;
use logging::{LoggingConfig, cleanup_old_logs, get_default_log_dir, initialize_logging};
use service::{ServiceManager, daemon::ServiceInstaller};

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

    /// Enable JSON logging format
    #[arg(long)]
    json_logs: bool,

    /// Disable file logging (console only)
    #[arg(long)]
    no_file_logs: bool,

    /// Custom log directory
    #[arg(long)]
    log_dir: Option<String>,
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
    /// Install system service
    InstallService,
    /// Uninstall system service
    UninstallService,
    /// Run as a background service (enhanced daemon)
    Service,
    /// Clean up old log files
    CleanupLogs {
        /// Number of days to keep (default: 30)
        #[arg(short, long, default_value = "30")]
        keep_days: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize enhanced logging
    let logging_config = LoggingConfig {
        level: if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        },
        file_output: !cli.no_file_logs,
        console_output: true,
        log_dir: cli.log_dir.as_ref().map(|d| d.into()),
        json_format: cli.json_logs,
    };

    let _guard = initialize_logging(logging_config)?;

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
        Some(Commands::InstallService) => {
            install_service()?;
        }
        Some(Commands::UninstallService) => {
            uninstall_service()?;
        }
        Some(Commands::Service) => {
            run_service(config).await?;
        }
        Some(Commands::CleanupLogs { keep_days }) => {
            cleanup_logs(keep_days)?;
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

fn install_service() -> Result<()> {
    info!("Installing system service");

    ServiceInstaller::install_launch_agent()?;

    println!("✓ Audio device monitor service installed successfully");
    println!("  Service will start automatically on login");
    println!(
        "  To start now: launchctl load ~/Library/LaunchAgents/com.audiodevicemonitor.daemon.plist"
    );
    println!("  To check status: launchctl list | grep audiodevicemonitor");

    Ok(())
}

fn uninstall_service() -> Result<()> {
    info!("Uninstalling system service");

    ServiceInstaller::uninstall_launch_agent()?;

    println!("✓ Audio device monitor service uninstalled successfully");
    println!(
        "  To stop if running: launchctl unload ~/Library/LaunchAgents/com.audiodevicemonitor.daemon.plist"
    );

    Ok(())
}

async fn run_service(config: Config) -> Result<()> {
    info!("Starting background service mode");

    let mut service_manager = ServiceManager::new(config);

    println!("Audio device monitor service starting...");
    println!("  Enhanced signal handling enabled");
    println!("  Send SIGTERM or SIGINT to stop gracefully");
    println!("  Send SIGHUP to reload configuration");

    service_manager.start().await?;

    println!("Service stopped");
    Ok(())
}

fn cleanup_logs(keep_days: u64) -> Result<()> {
    info!("Cleaning up old log files (keeping {} days)", keep_days);

    let log_dir = get_default_log_dir()?;
    cleanup_old_logs(&log_dir, keep_days)?;

    println!("✓ Log cleanup completed");
    println!("  Log directory: {}", log_dir.display());
    println!("  Kept files newer than {} days", keep_days);

    Ok(())
}
