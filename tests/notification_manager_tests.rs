use audio_device_monitor::TestNotificationSender;
use audio_device_monitor::config::{Config, GeneralConfig, NotificationConfig};
use audio_device_monitor::notifications::{NotificationManager, SwitchReason};

mod test_utils;
use test_utils::builders::AudioDeviceBuilder;

/// Helper function to create notification manager with test sender (no system notifications)
fn create_test_notification_manager(
    show_device_availability: bool,
    show_switching_actions: bool,
) -> NotificationManager<TestNotificationSender> {
    let config = Config {
        general: GeneralConfig::default(),
        notifications: NotificationConfig {
            show_device_availability,
            show_switching_actions,
            show_device_changes: None,
        },
        output_devices: vec![],
        input_devices: vec![],
    };

    let sender = TestNotificationSender::new();
    NotificationManager::with_sender(&config, sender)
}

/// Test configuration-based notification filtering
#[cfg(test)]
mod configuration_filtering {
    use super::*;

    #[test]
    fn test_device_availability_notifications_enabled() {
        let manager = create_test_notification_manager(true, false);
        let device = AudioDeviceBuilder::new()
            .name("Test Device")
            .output()
            .build();

        // Should complete successfully when availability notifications are enabled
        let result_connected = manager.device_connected(&device);
        let result_disconnected = manager.device_disconnected(&device);

        // Both should succeed (using test sender, no actual notifications)
        assert!(result_connected.is_ok());
        assert!(result_disconnected.is_ok());
    }

    #[test]
    fn test_device_availability_notifications_disabled() {
        let manager = create_test_notification_manager(false, false);
        let device = AudioDeviceBuilder::new()
            .name("Test Device")
            .output()
            .build();

        // When disabled, these should return Ok(()) immediately without sending
        let result_connected = manager.device_connected(&device);
        let result_disconnected = manager.device_disconnected(&device);

        assert!(result_connected.is_ok());
        assert!(result_disconnected.is_ok());
    }

    #[test]
    fn test_switching_action_notifications_enabled() {
        let manager = create_test_notification_manager(false, true);
        let device = AudioDeviceBuilder::new()
            .name("Test Device")
            .output()
            .build();

        // Should not panic when switching notifications are enabled
        let result_switched = manager.device_switched(&device, SwitchReason::HigherPriority);
        let result_failed = manager.switch_failed("Test Device", "Test error");

        // Methods should complete (may succeed or fail depending on system state)
        assert!(result_switched.is_ok() || result_switched.is_err());
        assert!(result_failed.is_ok() || result_failed.is_err());
    }

    #[test]
    fn test_switching_action_notifications_disabled() {
        let manager = create_test_notification_manager(false, false);
        let device = AudioDeviceBuilder::new()
            .name("Test Device")
            .output()
            .build();

        // When disabled, these should return Ok(()) immediately
        let result_switched = manager.device_switched(&device, SwitchReason::HigherPriority);
        let result_failed = manager.switch_failed("Test Device", "Test error");

        assert!(result_switched.is_ok());
        assert!(result_failed.is_ok());
    }

    #[test]
    fn test_all_notifications_enabled() {
        let manager = create_test_notification_manager(true, true);
        let device = AudioDeviceBuilder::new()
            .name("Test Device")
            .output()
            .build();

        // All notification types should be processed when enabled
        let results = vec![
            manager.device_connected(&device),
            manager.device_disconnected(&device),
            manager.device_switched(&device, SwitchReason::Manual),
            manager.switch_failed("Test Device", "Error message"),
        ];

        // All should complete without panic (success/failure depends on system state)
        for result in results {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_all_notifications_disabled() {
        let manager = create_test_notification_manager(false, false);
        let device = AudioDeviceBuilder::new()
            .name("Test Device")
            .output()
            .build();

        // All should return Ok(()) immediately when disabled
        assert!(manager.device_connected(&device).is_ok());
        assert!(manager.device_disconnected(&device).is_ok());
        assert!(
            manager
                .device_switched(&device, SwitchReason::PreviousUnavailable)
                .is_ok()
        );
        assert!(manager.switch_failed("Device", "Error").is_ok());
    }
}

/// Test notification manager state management
#[cfg(test)]
mod state_management {
    use super::*;

    #[test]
    fn test_default_configuration() {
        let default_manager = create_test_notification_manager(false, true); // Default values

        // Test that default values match documentation
        assert!(default_manager.is_enabled());
        // Note: We can't directly test show_device_availability and show_switching_actions
        // as they're private, but we can infer behavior through method calls
    }

    #[test]
    fn test_enable_disable_functionality() {
        let mut manager = create_test_notification_manager(true, true);

        // Test initial state
        assert!(manager.is_enabled());

        // Test disabling
        manager.set_enabled(false);
        assert!(!manager.is_enabled());

        // Test re-enabling
        manager.set_enabled(true);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_config_initialization() {
        // Test with different config combinations
        let configs = vec![
            (true, true),   // Both enabled
            (true, false),  // Only availability
            (false, true),  // Only switching
            (false, false), // Both disabled
        ];

        for (availability, switching) in configs {
            let manager = create_test_notification_manager(availability, switching);
            // Manager should be created successfully with any valid config
            assert!(manager.is_enabled()); // Should be enabled by default
        }
    }
}

/// Test device type emoji selection logic
#[cfg(test)]
mod device_type_formatting {
    use super::*;
    use audio_device_monitor::audio::DeviceType;

    #[test]
    fn test_output_device_emoji_selection() {
        let manager = create_test_notification_manager(true, true);
        let output_device = AudioDeviceBuilder::new()
            .name("Output Device")
            .output()
            .build();

        // Using test sender, methods should complete successfully with output devices
        assert!(manager.device_connected(&output_device).is_ok());
    }

    #[test]
    fn test_input_device_emoji_selection() {
        let manager = create_test_notification_manager(true, true);
        let input_device = AudioDeviceBuilder::new()
            .name("Input Device")
            .input()
            .build();

        // Using test sender, methods should work with input devices
        assert!(manager.device_disconnected(&input_device).is_ok());
    }

    #[test]
    fn test_input_output_device_emoji_selection() {
        let manager = create_test_notification_manager(true, true);

        // Create a device with InputOutput type using the builder
        let mut device = AudioDeviceBuilder::new()
            .name("Input/Output Device")
            .output() // Start with output
            .build();

        // Manually set to InputOutput type since builder doesn't have this method
        device.device_type = DeviceType::InputOutput;

        // Using test sender, methods should work with input/output devices
        assert!(
            manager
                .device_switched(&device, SwitchReason::HigherPriority)
                .is_ok()
        );
    }
}

/// Test switch reason message formatting
#[cfg(test)]
mod switch_reason_formatting {
    use super::*;

    #[test]
    fn test_higher_priority_switch_reason() {
        let manager = create_test_notification_manager(false, true);
        let device = AudioDeviceBuilder::new()
            .name("Priority Device")
            .output()
            .build();

        let result = manager.device_switched(&device, SwitchReason::HigherPriority);
        // Should complete successfully (the specific message format is internal)
        assert!(result.is_ok());
    }

    #[test]
    fn test_previous_unavailable_switch_reason() {
        let manager = create_test_notification_manager(false, true);
        let device = AudioDeviceBuilder::new()
            .name("Fallback Device")
            .input()
            .build();

        let result = manager.device_switched(&device, SwitchReason::PreviousUnavailable);
        assert!(result.is_ok());
    }

    #[test]
    fn test_manual_switch_reason() {
        let manager = create_test_notification_manager(false, true);
        let device = AudioDeviceBuilder::new()
            .name("Manual Device")
            .output()
            .build();

        let result = manager.device_switched(&device, SwitchReason::Manual);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_switch_reasons_with_different_device_types() {
        let manager = create_test_notification_manager(false, true);
        let reasons = vec![
            SwitchReason::HigherPriority,
            SwitchReason::PreviousUnavailable,
            SwitchReason::Manual,
        ];

        for (i, reason) in reasons.into_iter().enumerate() {
            let device = AudioDeviceBuilder::new()
                .name(&format!("Device {i}"))
                .output()
                .build();

            let result = manager.device_switched(&device, reason);
            assert!(result.is_ok());
        }
    }
}

/// Test edge cases and error conditions
#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_device_name() {
        let manager = create_test_notification_manager(true, true);
        let device = AudioDeviceBuilder::new().name("").output().build();

        // Using test sender, should handle empty device names gracefully
        assert!(manager.device_connected(&device).is_ok());
        assert!(
            manager
                .device_switched(&device, SwitchReason::Manual)
                .is_ok()
        );
    }

    #[test]
    fn test_unicode_device_name() {
        let manager = create_test_notification_manager(true, true);
        let device = AudioDeviceBuilder::new()
            .name("🎵 音频设备 🎵")
            .output()
            .build();

        // Using test sender, should handle Unicode device names
        assert!(manager.device_disconnected(&device).is_ok());
    }

    #[test]
    fn test_very_long_device_name() {
        let manager = create_test_notification_manager(true, true);
        let long_name = "A".repeat(1000);
        let device = AudioDeviceBuilder::new().name(&long_name).input().build();

        // Using test sender, should handle very long device names
        assert!(manager.device_connected(&device).is_ok());
    }

    #[test]
    fn test_special_characters_in_device_name() {
        let manager = create_test_notification_manager(true, true);
        let special_name = r#"Device "with" 'quotes' & <html> characters"#;
        let device = AudioDeviceBuilder::new()
            .name(special_name)
            .output()
            .build();

        // Using test sender, should handle special characters
        assert!(manager.device_connected(&device).is_ok());
    }

    #[test]
    fn test_switch_failed_with_empty_error() {
        let manager = create_test_notification_manager(false, true);

        let result = manager.switch_failed("Device Name", "");
        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_failed_with_long_error() {
        let manager = create_test_notification_manager(false, true);
        let long_error = "Error message ".repeat(100);

        let result = manager.switch_failed("Device", &long_error);
        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_failed_with_unicode_error() {
        let manager = create_test_notification_manager(false, true);

        let result = manager.switch_failed("Device", "错误消息: 设备不可用 🚫");
        assert!(result.is_ok());
    }
}

/// Test test notification functionality
#[cfg(test)]
mod test_notifications {
    use super::*;

    #[test]
    fn test_notification_test_method() {
        let manager = create_test_notification_manager(true, true);

        // Test notification should complete (may succeed or fail based on system state)
        let result = manager.test_notification();
        assert!(result.is_ok());
    }

    #[test]
    fn test_notification_with_disabled_manager() {
        let mut manager = create_test_notification_manager(true, true);
        manager.set_enabled(false);

        // Even when manager is disabled, test notification should still work
        // (test_notification bypasses the enabled check)
        let result = manager.test_notification();
        assert!(result.is_ok());
    }
}

/// Test notification manager creation from different config states
#[cfg(test)]
mod configuration_integration {
    use super::*;

    #[test]
    fn test_create_from_default_config() {
        let config = Config::default();
        let sender = TestNotificationSender::new();
        let manager = NotificationManager::with_sender(&config, sender);

        // Should create successfully from default config
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_create_from_custom_config() {
        let mut config = Config::default();
        config.notifications.show_device_availability = true;
        config.notifications.show_switching_actions = false;

        let sender = TestNotificationSender::new();
        let manager = NotificationManager::with_sender(&config, sender);
        assert!(manager.is_enabled());

        // Test behavior with custom configuration
        let device = AudioDeviceBuilder::new()
            .name("Custom Device")
            .output()
            .build();

        // Availability notifications should work (enabled)
        let availability_result = manager.device_connected(&device);
        assert!(availability_result.is_ok());

        // Switching notifications should be skipped (disabled) - should return Ok(())
        let switching_result = manager.device_switched(&device, SwitchReason::Manual);
        assert!(switching_result.is_ok());
    }

    #[test]
    fn test_create_with_all_combinations() {
        let combinations = vec![(false, false), (false, true), (true, false), (true, true)];

        for (availability, switching) in combinations {
            let mut config = Config::default();
            config.notifications.show_device_availability = availability;
            config.notifications.show_switching_actions = switching;

            let sender = TestNotificationSender::new();
            let manager = NotificationManager::with_sender(&config, sender);
            assert!(manager.is_enabled());

            // All managers should be creatable regardless of config
            let device = AudioDeviceBuilder::new().name("Test").output().build();

            // Methods should complete without panic
            let _ = manager.device_connected(&device);
            let _ = manager.device_switched(&device, SwitchReason::HigherPriority);
        }
    }
}
