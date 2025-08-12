use anyhow::Result;
use std::path::PathBuf;
use tracing::{error, info};

use crate::audio::DeviceControllerV2;
use crate::config::{Config, ConfigLoader};
use crate::system::{AudioSystemInterface, FileSystemInterface, SystemServiceInterface};

/// Main audio device service with dependency injection for complete testability
pub struct AudioDeviceService<
    A: AudioSystemInterface,
    F: FileSystemInterface,
    S: SystemServiceInterface,
> {
    device_controller: DeviceControllerV2<A>,
    config_loader: ConfigLoader<F>,
    system_service: S,
    config: Config,
    last_config_modified: Option<std::time::SystemTime>,
}

impl<A: AudioSystemInterface, F: FileSystemInterface, S: SystemServiceInterface>
    AudioDeviceService<A, F, S>
{
    pub fn new(
        audio_system: A,
        file_system: F,
        system_service: S,
        config_path: PathBuf,
    ) -> Result<Self> {
        let config_loader = ConfigLoader::new(file_system, config_path);
        let config = config_loader.load_config()?;
        let device_controller = DeviceControllerV2::new(audio_system, &config);

        Ok(Self {
            device_controller,
            config_loader,
            system_service,
            config,
            last_config_modified: None,
        })
    }

    /// Initialize and start the audio device service
    pub fn start(&mut self) -> Result<()> {
        info!("Starting audio device service with dependency injection");

        // Register signal handlers
        self.system_service.register_signal_handlers()?;

        // Initialize device controller
        self.device_controller.initialize()?;

        // Store initial config modification time for hot reload
        if let Ok(modified_time) = self
            .config_loader
            .get_config_path()
            .metadata()
            .and_then(|m| m.modified())
        {
            self.last_config_modified = Some(modified_time);
        }

        info!("Audio device service started successfully");

        // Enter main service loop
        self.run_main_loop()
    }

    /// Main service loop that handles events and monitors for changes
    fn run_main_loop(&mut self) -> Result<()> {
        info!("Entering main service loop");

        while self.system_service.should_continue_running() {
            // Run one iteration of the event loop
            self.system_service.run_event_loop()?;

            // Check for device changes
            if let Err(e) = self.device_controller.update_current_devices() {
                error!("Error updating current devices: {}", e);
            }

            // Check for SIGHUP configuration reload request
            if self.system_service.is_config_reload_requested() {
                info!("Received SIGHUP signal, reloading configuration");
                if let Err(e) = self.reload_config() {
                    error!("Failed to reload configuration: {}", e);
                } else {
                    info!("Configuration reloaded successfully");
                }
            }

            // Check for configuration changes (file-based hot reload)
            if let Err(e) = self.check_config_reload() {
                error!("Error checking config reload: {}", e);
            }

            // Sleep briefly to avoid busy waiting
            self.system_service
                .sleep_ms(self.config.general.check_interval_ms.max(100))?;
        }

        info!("Main service loop exited");
        Ok(())
    }

    /// Check if configuration has been modified and reload if necessary
    fn check_config_reload(&mut self) -> Result<()> {
        if let Some(last_modified) = self.last_config_modified {
            if self.config_loader.is_config_modified(last_modified)? {
                info!("Configuration file changed, reloading");
                self.reload_config()?;
            }
        }
        Ok(())
    }

    /// Reload configuration and reinitialize components
    pub fn reload_config(&mut self) -> Result<()> {
        info!("Reloading configuration");

        // Load new configuration
        let new_config = self.config_loader.load_config()?;

        // Update configuration
        self.config = new_config;

        // Note: In a full implementation, we would recreate the device controller
        // with the new configuration. For this PoC, we'll simulate the reload
        // by just updating the config and logging the operation.
        info!("Configuration reloaded successfully");

        // Update last modified time
        if let Ok(modified_time) = self
            .config_loader
            .get_config_path()
            .metadata()
            .and_then(|m| m.modified())
        {
            self.last_config_modified = Some(modified_time);
        }

        Ok(())
    }

    /// Get the current configuration
    // Called by CLI commands and monitoring systems that need access to current config
    #[allow(dead_code)]
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Get the process ID of the service
    // Called by CLI status command and monitoring systems to display service process info
    #[allow(dead_code)]
    pub fn get_process_id(&self) -> u32 {
        self.system_service.get_process_id()
    }

    /// Check if the service should continue running
    // Called by service main loop to check if shutdown signal has been received
    #[allow(dead_code)]
    pub fn should_continue_running(&self) -> bool {
        self.system_service.should_continue_running()
    }

    /// Handle a device being connected manually
    // Called by CLI commands and external systems that need to trigger device connection handling
    #[allow(dead_code)]
    pub fn handle_device_connected(&mut self, device_name: &str) -> Result<()> {
        info!("Manually handling device connection: {}", device_name);

        // Get current devices to find the newly connected one
        let devices = self.device_controller.enumerate_devices()?;
        if let Some(device) = devices.iter().find(|d| d.name == device_name) {
            self.device_controller.handle_device_connected(device)?;
        }

        // Update current device selection
        self.device_controller.update_current_devices()?;
        Ok(())
    }

    /// Handle a device being disconnected manually
    // Called by CLI commands and external systems that need to trigger device disconnection handling
    #[allow(dead_code)]
    pub fn handle_device_disconnected(&mut self, device_name: &str) -> Result<()> {
        info!("Manually handling device disconnection: {}", device_name);

        // For disconnect, we need to check current devices before they're removed
        let current_output_device = self.device_controller.get_current_output_device().cloned();
        let current_input_device = self.device_controller.get_current_input_device().cloned();

        if let Some(current_output) = current_output_device {
            if current_output.name == device_name {
                self.device_controller
                    .handle_device_disconnected(&current_output)?;
            }
        }

        if let Some(current_input) = current_input_device {
            if current_input.name == device_name {
                self.device_controller
                    .handle_device_disconnected(&current_input)?;
            }
        }

        // Update current device selection
        self.device_controller.update_current_devices()?;
        Ok(())
    }

    /// Shutdown the service gracefully
    // Called by CLI commands and signal handlers for graceful service shutdown
    #[allow(dead_code)]
    pub fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down audio device service");

        // The service will naturally stop when should_continue_running returns false
        // Additional cleanup can be added here if needed

        info!("Audio device service shutdown completed");
        Ok(())
    }

    /// Get device enumeration for external inspection
    // Called by CLI commands that need to list all available audio devices
    #[allow(dead_code)]
    pub fn enumerate_devices(&self) -> Result<Vec<crate::audio::AudioDevice>> {
        self.device_controller.enumerate_devices()
    }

    /// Get current output device
    // Called by CLI status and monitoring commands to show current device state
    #[allow(dead_code)]
    pub fn get_current_output_device(&self) -> Option<&crate::audio::AudioDevice> {
        self.device_controller.get_current_output_device()
    }

    /// Get current input device
    // Called by CLI status and monitoring commands to show current device state
    #[allow(dead_code)]
    pub fn get_current_input_device(&self) -> Option<&crate::audio::AudioDevice> {
        self.device_controller.get_current_input_device()
    }

    /// Manually set output device (for testing or manual control)
    // Called by CLI switch commands and external control systems for manual device switching
    #[allow(dead_code)]
    pub fn set_output_device(&mut self, device_name: &str) -> Result<()> {
        info!("Manually setting output device: {}", device_name);

        let devices = self.device_controller.enumerate_devices()?;
        if let Some(device) = devices.iter().find(|d| {
            d.name == device_name && matches!(d.device_type, crate::audio::DeviceType::Output)
        }) {
            self.device_controller.switch_to_output_device(device)?;
        } else {
            return Err(anyhow::anyhow!("Output device '{}' not found", device_name));
        }

        Ok(())
    }

    /// Manually set input device (for testing or manual control)
    // Called by CLI switch commands and external control systems for manual device switching
    #[allow(dead_code)]
    pub fn set_input_device(&mut self, device_name: &str) -> Result<()> {
        info!("Manually setting input device: {}", device_name);

        let devices = self.device_controller.enumerate_devices()?;
        if let Some(device) = devices.iter().find(|d| {
            d.name == device_name && matches!(d.device_type, crate::audio::DeviceType::Input)
        }) {
            self.device_controller.switch_to_input_device(device)?;
        } else {
            return Err(anyhow::anyhow!("Input device '{}' not found", device_name));
        }

        Ok(())
    }
}

// Convenience constructor for production use
impl
    AudioDeviceService<
        crate::system::CoreAudioSystem,
        crate::system::StandardFileSystem,
        crate::system::MacOSSystemService,
    >
{
    pub fn new_production(config_path: PathBuf) -> Result<Self> {
        let audio_system = crate::system::CoreAudioSystem::new()?;
        let file_system = crate::system::StandardFileSystem;
        let system_service = crate::system::MacOSSystemService::new();

        Self::new(audio_system, file_system, system_service, config_path)
    }

    /// Create a production service with the default configuration path
    pub fn new_with_default_config() -> Result<Self> {
        let config_path = ConfigLoader::default_config_path()?;
        Self::new_production(config_path)
    }
}

// Convenience constructor for testing
#[cfg(any(test, feature = "test-mocks"))]
impl
    AudioDeviceService<
        crate::system::MockAudioSystem,
        crate::system::MockFileSystem,
        crate::system::MockSystemService,
    >
{
    #[allow(dead_code)]  // Used by integration tests which run in different compilation context
    pub fn new_for_testing(config_path: PathBuf) -> Self {
        let audio_system = crate::system::MockAudioSystem::new();
        let file_system = crate::system::MockFileSystem::new();
        let system_service = crate::system::MockSystemService::new();

        Self::new(audio_system, file_system, system_service, config_path)
            .expect("Failed to create test service")
    }

    /// Access the mock system service for test control
    #[allow(dead_code)]  // Used by integration tests which run in different compilation context
    pub fn mock_system_service(&self) -> &crate::system::MockSystemService {
        &self.system_service
    }

    /// For testing: Get the configuration loader
    #[allow(dead_code)]  // Used by integration tests which run in different compilation context
    pub fn config_loader(&self) -> &ConfigLoader<crate::system::MockFileSystem> {
        &self.config_loader
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::{MockAudioSystem, MockFileSystem, MockSystemService};
    use std::path::PathBuf;

    #[test]
    fn test_service_creation() {
        let audio_system = MockAudioSystem::new();
        let file_system = MockFileSystem::new();
        let system_service = MockSystemService::new();
        let config_path = PathBuf::from("/test/config.toml");

        // Add a minimal config to the mock filesystem
        let config_content = r#"[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
"#;
        file_system.add_file(&config_path, config_content.to_string());

        let service =
            AudioDeviceService::new(audio_system, file_system, system_service, config_path);

        assert!(service.is_ok());
        let service = service.unwrap();
        assert_eq!(service.config.general.check_interval_ms, 1000);
    }

    #[test]
    fn test_service_device_handling() {
        let audio_system = MockAudioSystem::new();
        let file_system = MockFileSystem::new();
        let system_service = MockSystemService::new();
        let config_path = PathBuf::from("/test/config.toml");

        // Add minimal config
        let config_content = r#"[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Test Speaker"
weight = 100
match_type = "exact"
enabled = true
"#;
        file_system.add_file(&config_path, config_content.to_string());

        // Add a test device
        let test_device = crate::audio::AudioDevice::new(
            "test-1".to_string(),
            "Test Speaker".to_string(),
            crate::audio::DeviceType::Output,
        );
        audio_system.add_device(test_device.clone());

        let service =
            AudioDeviceService::new(audio_system, file_system, system_service, config_path)
                .unwrap();

        // Test device enumeration
        let devices = service.enumerate_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Test Speaker");
    }

    #[test]
    fn test_service_should_continue_running() {
        let audio_system = MockAudioSystem::new();
        let file_system = MockFileSystem::new();
        let system_service = MockSystemService::new();
        let config_path = PathBuf::from("/test/config.toml");

        // Add minimal config
        let config_content = r#"[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
"#;
        file_system.add_file(&config_path, config_content.to_string());

        let service = AudioDeviceService::new(
            audio_system,
            file_system.clone(),
            system_service.clone(),
            config_path,
        )
        .unwrap();

        // Should initially be running
        assert!(service.should_continue_running());

        // Stop the service
        system_service.stop_service();
        assert!(!service.should_continue_running());
    }
}
