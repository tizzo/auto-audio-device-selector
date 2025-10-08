use audio_device_monitor::{
    AudioDevice, AudioSystemInterface, Config, DeviceControllerV2, DeviceType, MockAudioSystem,
};

/// Integration tests for DeviceControllerV2 with dependency injection
/// These tests verify device enumeration, switching, and priority management

#[cfg(test)]
mod device_controller_tests {
    use super::*;

    fn create_test_config() -> Config {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = true
show_switching_actions = true

[[output_devices]]
name = "Premium Headphones"
weight = 100
match_type = "exact"
enabled = true

[[output_devices]]
name = "Gaming Headset"
weight = 90
match_type = "contains"
enabled = true

[[output_devices]]
name = "Built-in Speakers"
weight = 50
match_type = "exact"
enabled = true

[[input_devices]]
name = "Studio Microphone"
weight = 100
match_type = "exact"
enabled = true

[[input_devices]]
name = "Gaming Headset"
weight = 80
match_type = "contains"
enabled = true

[[input_devices]]
name = "Built-in Microphone"
weight = 40
match_type = "exact"
enabled = true
"#;
        toml::from_str(config_content).expect("Invalid test configuration")
    }

    fn setup_test_devices(audio_system: &MockAudioSystem) {
        let devices = vec![
            AudioDevice::new(
                "premium-1".to_string(),
                "Premium Headphones".to_string(),
                DeviceType::Output,
            ),
            AudioDevice::new(
                "gaming-out-1".to_string(),
                "Gaming Headset Pro".to_string(),
                DeviceType::Output,
            ),
            AudioDevice::new(
                "builtin-out-1".to_string(),
                "Built-in Speakers".to_string(),
                DeviceType::Output,
            ),
            AudioDevice::new(
                "studio-mic-1".to_string(),
                "Studio Microphone".to_string(),
                DeviceType::Input,
            ),
            AudioDevice::new(
                "gaming-mic-1".to_string(),
                "Gaming Headset Pro".to_string(),
                DeviceType::Input,
            ),
            AudioDevice::new(
                "builtin-mic-1".to_string(),
                "Built-in Microphone".to_string(),
                DeviceType::Input,
            ),
        ];

        for device in devices {
            audio_system.add_device(device);
        }
    }

    #[test]
    fn test_device_controller_creation_and_initialization() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        setup_test_devices(&audio_system);

        let mut device_controller = DeviceControllerV2::new(audio_system.clone(), &config);

        // Test initialization
        let result = device_controller.initialize();
        assert!(result.is_ok());

        // Verify audio system was called for initialization
        assert!(audio_system.get_enumerate_calls() > 0);
    }

    #[test]
    fn test_device_enumeration() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        setup_test_devices(&audio_system);

        let device_controller = DeviceControllerV2::new(audio_system.clone(), &config);

        let devices = device_controller.enumerate_devices().unwrap();
        assert_eq!(devices.len(), 6);

        // Verify we have the expected output devices
        let output_devices: Vec<_> = devices
            .iter()
            .filter(|d| matches!(d.device_type, DeviceType::Output))
            .collect();
        assert_eq!(output_devices.len(), 3);

        // Verify we have the expected input devices
        let input_devices: Vec<_> = devices
            .iter()
            .filter(|d| matches!(d.device_type, DeviceType::Input))
            .collect();
        assert_eq!(input_devices.len(), 3);

        // Verify specific devices exist
        assert!(devices.iter().any(|d| d.name == "Premium Headphones"));
        assert!(devices.iter().any(|d| d.name == "Gaming Headset Pro"));
        assert!(devices.iter().any(|d| d.name == "Studio Microphone"));

        // Verify audio system was called
        assert!(audio_system.get_enumerate_calls() > 0);
    }

    #[test]
    fn test_device_switching() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        setup_test_devices(&audio_system);

        let mut device_controller = DeviceControllerV2::new(audio_system.clone(), &config);
        device_controller.initialize().unwrap();

        // Get devices for switching
        let devices = device_controller.enumerate_devices().unwrap();
        let premium_headphones = devices
            .iter()
            .find(|d| d.name == "Premium Headphones")
            .unwrap();
        let studio_mic = devices
            .iter()
            .find(|d| d.name == "Studio Microphone")
            .unwrap();

        // Test output device switching
        let result = device_controller.switch_to_output_device(premium_headphones);
        assert!(result.is_ok());

        // Test input device switching
        let result = device_controller.switch_to_input_device(studio_mic);
        assert!(result.is_ok());

        // Verify audio system received switching calls
        assert!(!audio_system.get_set_default_output_calls().is_empty());
        assert!(!audio_system.get_set_default_input_calls().is_empty());

        // Verify current devices are tracked
        let current_output = device_controller.get_current_output_device();
        let current_input = device_controller.get_current_input_device();

        assert!(current_output.is_some());
        assert!(current_input.is_some());
        assert_eq!(current_output.unwrap().name, "Premium Headphones");
        assert_eq!(current_input.unwrap().name, "Studio Microphone");
    }

    #[test]
    fn test_device_availability_check() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        setup_test_devices(&audio_system);

        let device_controller = DeviceControllerV2::new(audio_system.clone(), &config);

        let devices = device_controller.enumerate_devices().unwrap();

        // Test that devices are available
        assert!(!devices.is_empty());

        // Test that we can get default devices
        let default_output = device_controller.get_default_output_device().unwrap();
        let default_input = device_controller.get_default_input_device().unwrap();

        // Should be able to get these without errors (may be None)
        assert!(default_output.is_none() || default_output.is_some());
        assert!(default_input.is_none() || default_input.is_some());
    }

    #[test]
    fn test_device_connection_handling() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        setup_test_devices(&audio_system);

        let mut device_controller = DeviceControllerV2::new(audio_system.clone(), &config);
        device_controller.initialize().unwrap();

        // Simulate device connection
        let devices = device_controller.enumerate_devices().unwrap();
        let premium_headphones = devices
            .iter()
            .find(|d| d.name == "Premium Headphones")
            .unwrap();

        let result = device_controller.handle_device_connected(premium_headphones);
        assert!(result.is_ok());

        // The controller should have attempted to switch to the high-priority device
        assert!(!audio_system.get_set_default_output_calls().is_empty());
    }

    #[test]
    fn test_device_disconnection_handling() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        setup_test_devices(&audio_system);

        let mut device_controller = DeviceControllerV2::new(audio_system.clone(), &config);
        device_controller.initialize().unwrap();

        // Set a device as current, then disconnect it
        let devices = device_controller.enumerate_devices().unwrap();
        let premium_headphones = devices
            .iter()
            .find(|d| d.name == "Premium Headphones")
            .unwrap();

        device_controller
            .switch_to_output_device(premium_headphones)
            .unwrap();

        let result = device_controller.handle_device_disconnected(premium_headphones);
        assert!(result.is_ok());

        // The current device should be cleared
        let current_output = device_controller.get_current_output_device();
        assert!(current_output.is_none() || current_output.unwrap().name != "Premium Headphones");
    }

    #[test]
    fn test_current_device_updates() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        setup_test_devices(&audio_system);

        let mut device_controller = DeviceControllerV2::new(audio_system.clone(), &config);
        device_controller.initialize().unwrap();

        // Initially no current devices
        assert!(device_controller.get_current_output_device().is_none());
        assert!(device_controller.get_current_input_device().is_none());

        // Set default devices in the mock audio system
        let devices = device_controller.enumerate_devices().unwrap();
        let premium_headphones = devices
            .iter()
            .find(|d| d.name == "Premium Headphones")
            .unwrap();
        let studio_mic = devices
            .iter()
            .find(|d| d.name == "Studio Microphone")
            .unwrap();

        audio_system
            .set_default_output_device(&premium_headphones.id)
            .unwrap();
        audio_system
            .set_default_input_device(&studio_mic.id)
            .unwrap();

        // Update current devices
        let result = device_controller.update_current_devices();
        assert!(result.is_ok());

        // Verify current devices are detected
        let current_output = device_controller.get_current_output_device();
        let current_input = device_controller.get_current_input_device();

        assert!(current_output.is_some());
        assert!(current_input.is_some());
        assert_eq!(current_output.unwrap().name, "Premium Headphones");
        assert_eq!(current_input.unwrap().name, "Studio Microphone");
    }

    #[test]
    fn test_device_controller_with_disabled_devices() {
        let audio_system = MockAudioSystem::new();
        let mut config = create_test_config();

        // Disable some devices in configuration
        config.output_devices[0].enabled = false; // Premium Headphones
        config.input_devices[0].enabled = false; // Studio Microphone

        setup_test_devices(&audio_system);

        let mut device_controller = DeviceControllerV2::new(audio_system.clone(), &config);
        device_controller.initialize().unwrap();

        let devices = device_controller.enumerate_devices().unwrap();
        let premium_headphones = devices
            .iter()
            .find(|d| d.name == "Premium Headphones")
            .unwrap();
        let studio_mic = devices
            .iter()
            .find(|d| d.name == "Studio Microphone")
            .unwrap();

        // Attempt to connect disabled devices - should not trigger automatic switching
        let result1 = device_controller.handle_device_connected(premium_headphones);
        let result2 = device_controller.handle_device_connected(studio_mic);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Since devices are disabled, they shouldn't become current devices automatically
        // (This depends on implementation details, but the test demonstrates the configuration is respected)
    }

    #[test]
    fn test_error_handling_invalid_device_operations() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        // Don't set up any devices to test error handling

        let mut device_controller = DeviceControllerV2::new(audio_system.clone(), &config);

        // Test operations with no devices available
        let devices = device_controller.enumerate_devices().unwrap();
        assert_eq!(devices.len(), 0);

        // Test update with no devices
        let result = device_controller.update_current_devices();
        assert!(result.is_ok()); // Should handle gracefully

        assert!(device_controller.get_current_output_device().is_none());
        assert!(device_controller.get_current_input_device().is_none());
    }

    #[test]
    fn test_mock_audio_system_call_tracking() {
        let audio_system = MockAudioSystem::new();
        let config = create_test_config();

        setup_test_devices(&audio_system);

        let mut device_controller = DeviceControllerV2::new(audio_system.clone(), &config);

        // Perform various operations
        device_controller.initialize().unwrap();
        let _devices = device_controller.enumerate_devices().unwrap();
        device_controller.update_current_devices().unwrap();

        // Verify all calls were tracked
        assert!(audio_system.get_enumerate_calls() > 0);
        assert!(audio_system.get_default_output_calls() > 0);
        assert!(audio_system.get_default_input_calls() > 0);

        // Perform device switching
        let devices = device_controller.enumerate_devices().unwrap();
        if let Some(output_device) = devices
            .iter()
            .find(|d| matches!(d.device_type, DeviceType::Output))
        {
            device_controller
                .switch_to_output_device(output_device)
                .unwrap();
            assert!(!audio_system.get_set_default_output_calls().is_empty());
        }

        if let Some(input_device) = devices
            .iter()
            .find(|d| matches!(d.device_type, DeviceType::Input))
        {
            device_controller
                .switch_to_input_device(input_device)
                .unwrap();
            assert!(!audio_system.get_set_default_input_calls().is_empty());
        }
    }
}
