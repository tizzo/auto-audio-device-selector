use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info, warn};

use super::signals::{SignalHandler, SignalType};
use crate::audio::AudioDeviceMonitor;
use crate::config::Config;

/// Manages the background service lifecycle
pub struct ServiceManager {
    config: Config,
    signal_handler: SignalHandler,
    // Used by the service lifecycle management system for device monitoring
    #[allow(dead_code)]
    monitor: Option<AudioDeviceMonitor>,
}

impl ServiceManager {
    // Called by legacy service systems that need tokio-based background service management
    #[allow(dead_code)]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            signal_handler: SignalHandler::new(),
            monitor: None,
        }
    }

    /// Start the background service
    // Called by legacy service launcher for tokio-based background service execution
    #[allow(dead_code)]
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting audio device monitor service");

        // Initialize the audio device monitor
        self.monitor = Some(AudioDeviceMonitor::new(self.config.clone())?);
        let monitor = self.monitor.as_ref().unwrap();

        // Create signal channel
        let (signal_tx, mut signal_rx) = mpsc::unbounded_channel::<SignalType>();

        // Create signal handler with sender
        self.signal_handler = SignalHandler::with_sender(signal_tx);
        let signal_handler = self.signal_handler.clone();
        let shutdown_flag = signal_handler.shutdown_flag();

        // Start signal handling in a separate task
        tokio::spawn(async move {
            if let Err(e) = signal_handler.listen_for_signals().await {
                error!("Signal handler error: {}", e);
            }
        });

        // Start the device monitoring
        monitor.start_monitoring_async().await?;

        info!("Service started successfully, entering main loop");

        // Main service loop
        loop {
            tokio::select! {
                // Check for signals
                signal = signal_rx.recv() => {
                    match signal {
                        Some(SignalType::Shutdown) => {
                            info!("Shutdown signal received, stopping service");
                            break;
                        }
                        Some(SignalType::Reload) => {
                            info!("Reload signal received, reloading configuration");
                            if let Err(e) = self.reload_config(None).await {
                                error!("Failed to reload configuration: {}", e);
                            }
                        }
                        None => {
                            warn!("Signal channel closed");
                            break;
                        }
                    }
                }
                // Check for shutdown via flag (fallback)
                _ = sleep(Duration::from_millis(100)) => {
                    if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        info!("Shutdown flag set, stopping service");
                        break;
                    }
                }
            }
        }

        // Cleanup
        self.shutdown().await?;

        Ok(())
    }

    /// Shutdown the service gracefully
    // Called by signal handlers and cleanup routines for graceful service shutdown
    #[allow(dead_code)]
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down audio device monitor service");

        if let Some(monitor) = &self.monitor {
            monitor.stop()?;
        }

        info!("Service shutdown completed");
        Ok(())
    }

    /// Check if the service is running
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        !self.signal_handler.is_shutdown_requested()
    }

    /// Get the current configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Reload configuration (for SIGHUP support)
    // Called by signal handlers when SIGHUP is received for configuration hot-reload
    #[allow(dead_code)]
    pub async fn reload_config(&mut self, config_path: Option<&str>) -> Result<()> {
        info!("Reloading configuration");

        let new_config = Config::load(config_path)?;

        // Stop current monitor
        if let Some(monitor) = &self.monitor {
            monitor.stop()?;
        }

        // Update config and restart monitor
        self.config = new_config;
        self.monitor = Some(AudioDeviceMonitor::new(self.config.clone())?);

        if let Some(monitor) = &self.monitor {
            monitor.start_monitoring_async().await?;
        }

        info!("Configuration reloaded successfully");
        Ok(())
    }
}

/// Service installation utilities
pub struct ServiceInstaller;

impl ServiceInstaller {
    /// Install the service as a macOS LaunchAgent
    pub fn install_launch_agent() -> Result<()> {
        info!("Installing macOS LaunchAgent");

        let plist_content = Self::generate_launch_agent_plist()?;
        let plist_path = Self::get_launch_agent_path()?;

        // Create the LaunchAgents directory if it doesn't exist
        if let Some(parent) = plist_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write the plist file
        std::fs::write(&plist_path, plist_content)?;

        info!("LaunchAgent installed to: {}", plist_path.display());
        info!(
            "To load the service, run: launchctl load {}",
            plist_path.display()
        );

        Ok(())
    }

    /// Uninstall the LaunchAgent
    pub fn uninstall_launch_agent() -> Result<()> {
        info!("Uninstalling macOS LaunchAgent");

        let plist_path = Self::get_launch_agent_path()?;

        if plist_path.exists() {
            std::fs::remove_file(&plist_path)?;
            info!("LaunchAgent removed from: {}", plist_path.display());
            info!(
                "To unload the service, run: launchctl unload {}",
                plist_path.display()
            );
        } else {
            warn!("LaunchAgent plist not found at: {}", plist_path.display());
        }

        Ok(())
    }

    fn generate_launch_agent_plist() -> Result<String> {
        let current_exe = std::env::current_exe()?;
        let exe_path = current_exe.to_string_lossy();

        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.audiodevicemonitor.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe_path}</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/audio-device-monitor.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/audio-device-monitor.err</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
    </dict>
</dict>
</plist>"#
        );

        Ok(plist)
    }

    fn get_launch_agent_path() -> Result<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        Ok(home_dir.join("Library/LaunchAgents/com.audiodevicemonitor.daemon.plist"))
    }
}
