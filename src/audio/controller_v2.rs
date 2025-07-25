use anyhow::Result;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::notifications::{NotificationManager, SwitchReason};
use crate::priority::DevicePriorityManager;
use crate::system::AudioSystemInterface;

use super::device::{AudioDevice, DeviceInfo};

/// Refactored DeviceController that accepts an AudioSystemInterface for dependency injection
pub struct DeviceController<A: AudioSystemInterface> {
    audio_system: A,
    priority_manager: DevicePriorityManager,
    notification_manager: NotificationManager,
    current_output: Option<AudioDevice>,
    current_input: Option<AudioDevice>,
}

impl<A: AudioSystemInterface> DeviceController<A> {
    pub fn new(audio_system: A, config: &Config) -> Self {
        Self {
            audio_system,
            priority_manager: DevicePriorityManager::new(config),
            notification_manager: NotificationManager::new(config),
            current_output: None,
            current_input: None,
        }
    }

    /// Initialize the controller and start monitoring for device changes
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing device controller with dependency injection");

        // Set initial devices
        self.update_current_devices()?;

        // Set up device change monitoring
        self.start_monitoring()?;

        info!("Device controller initialization complete");
        Ok(())
    }

    /// Start monitoring for device changes
    pub fn start_monitoring(&mut self) -> Result<()> {
        info!("Starting device change monitoring");

        // Create a callback that will handle device changes
        // Note: In a real implementation, we'd need to handle the callback lifetime properly
        let callback = Box::new(|| {
            debug!("Device change detected");
            // In practice, this would need to trigger a method on the controller
            // For now, we just log the event
        });

        self.audio_system.add_device_change_listener(callback)?;
        info!("Device change monitoring started");
        Ok(())
    }

    /// Update the current devices based on system defaults and priority rules
    pub fn update_current_devices(&mut self) -> Result<()> {
        debug!("Updating current device state");

        // Get all available devices
        let available_devices = self.audio_system.enumerate_devices()?;
        debug!("Found {} available devices", available_devices.len());

        // Find the best output device
        let best_output = self
            .priority_manager
            .find_best_output_device(&available_devices);
        if let Some(ref device) = best_output {
            if self.current_output.as_ref().map(|d| &d.id) != Some(&device.id) {
                info!("Switching to output device: {}", device.name);
                self.switch_to_output_device(device)?;
            }
        }

        // Find the best input device
        let best_input = self
            .priority_manager
            .find_best_input_device(&available_devices);
        if let Some(ref device) = best_input {
            if self.current_input.as_ref().map(|d| &d.id) != Some(&device.id) {
                info!("Switching to input device: {}", device.name);
                self.switch_to_input_device(device)?;
            }
        }

        Ok(())
    }

    /// Switch to a specific output device
    pub fn switch_to_output_device(&mut self, device: &AudioDevice) -> Result<()> {
        info!(
            "Switching to output device: {} ({})",
            device.name, device.id
        );

        // Use device name for switching (matching current DeviceController interface)
        self.audio_system.set_default_output_device(&device.name)?;

        // Update internal state
        let previous_device = self.current_output.clone();
        self.current_output = Some(device.clone());

        // Send notification
        let switch_reason = if previous_device.is_some() {
            SwitchReason::HigherPriority
        } else {
            SwitchReason::Manual
        };

        if let Err(e) = self
            .notification_manager
            .device_switched(device, switch_reason)
        {
            error!("Failed to send device switched notification: {}", e);
        }

        info!("Successfully switched to output device: {}", device.name);
        Ok(())
    }

    /// Switch to a specific input device
    pub fn switch_to_input_device(&mut self, device: &AudioDevice) -> Result<()> {
        info!("Switching to input device: {} ({})", device.name, device.id);

        // Use device name for switching (matching current DeviceController interface)
        self.audio_system.set_default_input_device(&device.name)?;

        // Update internal state
        let previous_device = self.current_input.clone();
        self.current_input = Some(device.clone());

        // Send notification
        let switch_reason = if previous_device.is_some() {
            SwitchReason::HigherPriority
        } else {
            SwitchReason::Manual
        };

        if let Err(e) = self
            .notification_manager
            .device_switched(device, switch_reason)
        {
            error!("Failed to send device switched notification: {}", e);
        }

        info!("Successfully switched to input device: {}", device.name);
        Ok(())
    }

    /// Get all available devices using the injected audio system
    pub fn enumerate_devices(&self) -> Result<Vec<AudioDevice>> {
        self.audio_system.enumerate_devices()
    }

    /// Get the current default output device
    pub fn get_default_output_device(&self) -> Result<Option<AudioDevice>> {
        self.audio_system.get_default_output_device()
    }

    /// Get the current default input device
    pub fn get_default_input_device(&self) -> Result<Option<AudioDevice>> {
        self.audio_system.get_default_input_device()
    }

    /// Get the currently active output device (internal state)
    pub fn get_current_output_device(&self) -> Option<&AudioDevice> {
        self.current_output.as_ref()
    }

    /// Get the currently active input device (internal state)
    pub fn get_current_input_device(&self) -> Option<&AudioDevice> {
        self.current_input.as_ref()
    }

    /// Get device information (for backward compatibility)
    pub fn get_device_info(&self, device: &AudioDevice) -> Result<DeviceInfo> {
        Ok(DeviceInfo {
            name: device.name.clone(),
            uid: device.uid.clone().unwrap_or_else(|| device.id.clone()),
            device_type: device.device_type.clone(),
            sample_rate: None,
            channels: None,
            is_default: device.is_default,
        })
    }

    /// Check if a device is currently available
    pub fn is_device_available(&self, device_id: &str) -> Result<bool> {
        self.audio_system.is_device_available(device_id)
    }

    /// Handle a device being connected (for external notification)
    pub fn handle_device_connected(&self, device: &AudioDevice) -> Result<()> {
        if let Err(e) = self.notification_manager.device_connected(device) {
            error!("Failed to send device connected notification: {}", e);
        }
        Ok(())
    }

    /// Handle a device being disconnected (for external notification)
    pub fn handle_device_disconnected(&self, device: &AudioDevice) -> Result<()> {
        if let Err(e) = self.notification_manager.device_disconnected(device) {
            error!("Failed to send device disconnected notification: {}", e);
        }
        Ok(())
    }

    /// Process device changes (to be called when device change callback is triggered)
    pub fn handle_device_change(&mut self) -> Result<()> {
        debug!("Processing device change event");
        self.update_current_devices()
    }

    /// Set the default output device by name (for backward compatibility)
    pub fn set_default_output_device(&self, device_name: &str) -> Result<()> {
        info!("Setting default output device to: {}", device_name);
        self.audio_system.set_default_output_device(device_name)
    }

    /// Set the default input device by name (for backward compatibility)
    pub fn set_default_input_device(&self, device_name: &str) -> Result<()> {
        info!("Setting default input device to: {}", device_name);
        self.audio_system.set_default_input_device(device_name)
    }
}

// Convenience constructor for production use with CoreAudioSystem
impl DeviceController<crate::system::CoreAudioSystem> {
    pub fn new_production(config: &Config) -> Result<Self> {
        let audio_system = crate::system::CoreAudioSystem::new()?;
        Ok(Self::new(audio_system, config))
    }
}
