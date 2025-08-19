use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn};

mod audio;
mod config;
mod logging;
mod notifications;
mod priority;
mod service;
mod system;

use audio::AudioDeviceMonitor;
use config::Config;
use logging::{LoggingConfig, cleanup_old_logs, get_default_log_dir, initialize_logging};
use notifications::DefaultNotificationManager;
use service::{AudioDeviceService, daemon::ServiceInstaller};

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
    /// Clean up old log files
    CleanupLogs {
        /// Number of days to keep (default: 30)
        #[arg(short, long, default_value = "30")]
        keep_days: u64,
    },
    /// Test notification system
    TestNotification,
    /// Show detailed information about a specific device
    DeviceInfo {
        /// Device name to inspect
        #[arg(short, long)]
        device: String,
    },
    /// Check if a device is currently available
    CheckDevice {
        /// Device name to check
        #[arg(short, long)]
        device: String,
    },
    /// Show current service status and configuration
    Status,
    /// Show current active/selected devices
    ShowCurrent,
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
            run_daemon(cli.config.as_deref()).await?;
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
        Some(Commands::CleanupLogs { keep_days }) => {
            cleanup_logs(keep_days)?;
        }
        Some(Commands::TestNotification) => {
            test_notification()?;
        }
        Some(Commands::DeviceInfo { device }) => {
            device_info(&device).await?;
        }
        Some(Commands::CheckDevice { device }) => {
            check_device(&device).await?;
        }
        Some(Commands::Status) => {
            show_status().await?;
        }
        Some(Commands::ShowCurrent) => {
            show_current_devices().await?;
        }
        None => {
            // Default behavior - run daemon if no command specified
            info!("No command specified, running in daemon mode");
            run_daemon(cli.config.as_deref()).await?;
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

async fn run_daemon(config_path: Option<&str>) -> Result<()> {
    info!("Starting daemon mode");

    // Create the service with either custom or default config path
    let mut service = if let Some(path) = config_path {
        let config_path = std::path::PathBuf::from(path);
        AudioDeviceService::new_production(config_path)?
    } else {
        AudioDeviceService::new_with_default_config()?
    };

    println!("Audio device monitor daemon started");
    println!("  Enhanced signal handling enabled");
    println!("  Send SIGTERM or SIGINT to stop gracefully");
    println!("  Send SIGHUP to reload configuration");

    // Start the service (this will block until shutdown)
    service.start()?;

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

    let controller = audio::controller::DeviceController::new()?;

    println!("Current default devices:");

    if let Ok(Some(default_input)) = controller.get_default_input_device() {
        println!("  Input:  {default_input}");
    } else {
        println!("  Input:  None available");
    }

    if let Ok(Some(default_output)) = controller.get_default_output_device() {
        println!("  Output: {default_output}");
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
    let config = Config::load(None)?;
    let notification_manager = DefaultNotificationManager::new(&config);

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

            // Send manual switch notification
            if let Ok(devices) = controller.enumerate_devices() {
                if let Some(device) = devices.iter().find(|d| d.name == device_name) {
                    if let Err(e) = notification_manager
                        .device_switched(device, crate::notifications::SwitchReason::Manual)
                    {
                        warn!("Failed to send manual switch notification: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to switch device: {e}");

            // Send switch failed notification
            if let Err(notification_err) =
                notification_manager.switch_failed(device_name, &e.to_string())
            {
                warn!(
                    "Failed to send switch failed notification: {}",
                    notification_err
                );
            }

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

fn cleanup_logs(keep_days: u64) -> Result<()> {
    info!("Cleaning up old log files (keeping {} days)", keep_days);

    let log_dir = get_default_log_dir()?;
    cleanup_old_logs(&log_dir, keep_days)?;

    println!("✓ Log cleanup completed");
    println!("  Log directory: {}", log_dir.display());
    println!("  Kept files newer than {keep_days} days");

    Ok(())
}

fn test_notification() -> Result<()> {
    info!("Testing notification system");

    let config = Config::load(None)?;
    let notification_manager = DefaultNotificationManager::new(&config);

    println!("🔔 Testing macOS Notification System");
    println!("=====================================");
    println!();

    println!("📱 Sending test notification...");
    notification_manager.test_notification()?;

    println!();
    println!("✅ Notification sent successfully!");
    println!();
    println!("🔍 If you don't see the notification, try:");
    println!("   1. Click the 🕐 clock icon in top-right corner");
    println!("   2. Check if 'Do Not Disturb' is disabled");
    println!("   3. Open System Preferences > Notifications & Focus");
    println!("   4. Look for 'Audio Device Monitor' in the app list");
    println!("   5. Enable 'Allow Notifications' and 'Show in Notification Center'");
    println!();
    println!("💡 On first run, macOS may ask for notification permission");
    println!("   Grant permission when prompted, then run this test again");

    Ok(())
}

async fn device_info(device_name: &str) -> Result<()> {
    info!("Getting device information for: {}", device_name);

    let controller = audio::controller::DeviceController::new()?;
    let devices = controller.enumerate_devices()?;

    // Find the device
    let device = devices
        .iter()
        .find(|d| d.name.contains(device_name) || d.name == device_name)
        .ok_or_else(|| anyhow::anyhow!("Device '{}' not found", device_name))?;

    // Get detailed info
    if let Ok(info) = controller.get_device_info(device) {
        println!("Device Information:");
        println!("  Name: {}", info.name);
        println!("  UID: {}", info.uid);
        println!("  Type: {}", info.device_type);
        println!("  Default: {}", if info.is_default { "Yes" } else { "No" });
        println!(
            "  Available: {}",
            if device.is_available { "Yes" } else { "No" }
        );
    } else {
        println!(
            "Device '{}' found but detailed info unavailable",
            device.name
        );
    }

    Ok(())
}

async fn check_device(device_name: &str) -> Result<()> {
    info!("Checking device availability: {}", device_name);

    let controller = audio::controller::DeviceController::new()?;

    // Check if device is available using the controller method
    match controller.enumerate_devices() {
        Ok(devices) => {
            let device = devices
                .iter()
                .find(|d| d.name.contains(device_name) || d.name == device_name);

            match device {
                Some(d) => {
                    println!(
                        "Device '{}': {}",
                        device_name,
                        if d.is_available {
                            "✓ Available"
                        } else {
                            "✗ Unavailable"
                        }
                    );
                }
                None => {
                    println!("Device '{device_name}': ✗ Not Found");
                }
            }
        }
        Err(e) => {
            println!("Failed to check device availability: {e}");
        }
    }

    Ok(())
}

async fn show_status() -> Result<()> {
    info!("Showing service status");

    println!("Audio Device Monitor Status:");
    println!("============================");

    // Load and show config
    let config = Config::load(None)?;
    println!("  Configuration:");
    println!("    Check interval: {}ms", config.general.check_interval_ms);
    println!("    Log level: {}", config.general.log_level);
    println!("    Output device rules: {}", config.output_devices.len());
    println!("    Input device rules: {}", config.input_devices.len());

    // Show current devices
    let controller = audio::controller::DeviceController::new()?;

    if let Ok(Some(output)) = controller.get_default_output_device() {
        println!("    Current output: {}", output.name);
    }

    if let Ok(Some(input)) = controller.get_default_input_device() {
        println!("    Current input: {}", input.name);
    }

    // Show process info
    println!("    Process ID: {}", std::process::id());

    Ok(())
}

async fn show_current_devices() -> Result<()> {
    info!("Showing current active devices");

    let controller = audio::controller::DeviceController::new()?;

    println!("Current Active Devices:");
    println!("======================");

    if let Ok(Some(output)) = controller.get_default_output_device() {
        println!("  🔊 Output: {}", output.name);
        println!("     UID: {}", output.id);
        println!("     Type: {}", output.device_type);
    } else {
        println!("  🔊 Output: None available");
    }

    if let Ok(Some(input)) = controller.get_default_input_device() {
        println!("  🎤 Input: {}", input.name);
        println!("     UID: {}", input.id);
        println!("     Type: {}", input.device_type);
    } else {
        println!("  🎤 Input: None available");
    }

    Ok(())
}
