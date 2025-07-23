use anyhow::Result;
use tracing::{debug, error, info, warn};

use crate::audio::AudioDevice;
use crate::config::Config;

/// Manages system notifications for audio device events
pub struct NotificationManager {
    enabled: bool,
    show_device_changes: bool,
    show_switching_actions: bool,
}

impl NotificationManager {
    pub fn new(config: &Config) -> Self {
        Self {
            enabled: true, // Can be controlled by config in the future
            show_device_changes: config.notifications.show_device_changes,
            show_switching_actions: config.notifications.show_switching_actions,
        }
    }

    /// Send notification when a device comes online
    pub fn device_connected(&self, device: &AudioDevice) -> Result<()> {
        if !self.enabled || !self.show_device_changes {
            return Ok(());
        }

        let device_type = match device.device_type {
            crate::audio::DeviceType::Input => "🎤",
            crate::audio::DeviceType::Output => "🔊",
            crate::audio::DeviceType::InputOutput => "🎧",
        };

        let title = "Audio Device Connected";
        let body = format!("{} {} is now available", device_type, device.name);

        self.send_notification(title, &body, NotificationType::DeviceChange)?;

        info!("Sent device connected notification for: {}", device.name);
        Ok(())
    }

    /// Send notification when a device goes offline
    pub fn device_disconnected(&self, device: &AudioDevice) -> Result<()> {
        if !self.enabled || !self.show_device_changes {
            return Ok(());
        }

        let device_type = match device.device_type {
            crate::audio::DeviceType::Input => "🎤",
            crate::audio::DeviceType::Output => "🔊",
            crate::audio::DeviceType::InputOutput => "🎧",
        };

        let title = "Audio Device Disconnected";
        let body = format!("{} {} is no longer available", device_type, device.name);

        self.send_notification(title, &body, NotificationType::DeviceChange)?;

        info!("Sent device disconnected notification for: {}", device.name);
        Ok(())
    }

    /// Send notification when automatic switching occurs
    pub fn device_switched(&self, device: &AudioDevice, reason: SwitchReason) -> Result<()> {
        if !self.enabled || !self.show_switching_actions {
            return Ok(());
        }

        let device_type = match device.device_type {
            crate::audio::DeviceType::Input => "🎤 Input",
            crate::audio::DeviceType::Output => "🔊 Output",
            crate::audio::DeviceType::InputOutput => "🎧 Input/Output",
        };

        let title = "Audio Device Switched";
        let body = match reason {
            SwitchReason::HigherPriority => {
                format!(
                    "{} switched to {} (higher priority)",
                    device_type, device.name
                )
            }
            SwitchReason::PreviousUnavailable => {
                format!(
                    "{} switched to {} (previous device unavailable)",
                    device_type, device.name
                )
            }
            SwitchReason::Manual => {
                format!("{} manually switched to {}", device_type, device.name)
            }
        };

        self.send_notification(title, &body, NotificationType::SwitchAction)?;

        info!(
            "Sent device switched notification: {} -> {}",
            device_type, device.name
        );
        Ok(())
    }

    /// Send notification when switching fails
    pub fn switch_failed(&self, device_name: &str, error: &str) -> Result<()> {
        if !self.enabled || !self.show_switching_actions {
            return Ok(());
        }

        let title = "Audio Device Switch Failed";
        let body = format!("Failed to switch to {device_name}: {error}");

        self.send_notification(title, &body, NotificationType::Error)?;

        warn!("Sent switch failed notification for: {}", device_name);
        Ok(())
    }

    /// Send a generic system notification using native macOS osascript
    fn send_notification(
        &self,
        title: &str,
        body: &str,
        _notification_type: NotificationType,
    ) -> Result<()> {
        debug!("Sending notification: {} - {}", title, body);

        // Send notification using native macOS osascript
        self.send_native_macos_notification(title, body)?;

        debug!(
            "Successfully sent notification via native macOS osascript: {}",
            title
        );
        Ok(())
    }

    /// Check if notifications are enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable notifications
    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        info!(
            "Notifications {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Test notification (for debugging)
    pub fn test_notification(&self) -> Result<()> {
        info!("Starting notification test...");

        let title = "Audio Device Monitor";
        let body = "Notification system is working correctly!";

        info!("Sending native macOS osascript notification...");

        match self.send_native_macos_notification(title, body) {
            Ok(_) => {
                info!("Native macOS notification sent successfully");
                info!("Check your notifications (should appear in top-right corner)");
                info!("This notification method works reliably for unsigned apps");
            }
            Err(e) => {
                error!("Failed to send notification: {}", e);
                error!("This might be due to:");
                error!("1. Do Not Disturb mode is enabled");
                error!("2. osascript is not available or restricted");
                error!("3. System-level notification restrictions");
                return Err(anyhow::anyhow!("Failed to send notification: {}", e));
            }
        }

        info!("Test notification completed");
        Ok(())
    }

    /// Send notification using native macOS osascript (more reliable for unsigned apps)
    fn send_native_macos_notification(&self, title: &str, body: &str) -> Result<()> {
        use std::process::Command;

        let script = format!(
            r#"display notification "{}" with title "{}" subtitle "" sound name """#,
            body.replace('"', "\\\""),
            title.replace('"', "\\\"")
        );

        let output = Command::new("osascript").args(["-e", &script]).output()?;

        if output.status.success() {
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("osascript failed: {}", error))
        }
    }
}

/// Types of notifications for different styling/sounds
#[derive(Debug, Clone)]
enum NotificationType {
    DeviceChange, // Device connected/disconnected
    SwitchAction, // Automatic switching occurred
    Error,        // Something went wrong
}

/// Reasons for device switching (for notification context)
#[derive(Debug, Clone)]
#[allow(dead_code)] // All variants kept for API completeness
pub enum SwitchReason {
    HigherPriority,      // A higher priority device became available
    PreviousUnavailable, // Previous device became unavailable
    Manual,              // User manually switched
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self {
            enabled: true,
            show_device_changes: true,
            show_switching_actions: true,
        }
    }
}
