use anyhow::Result;
use audio_device_monitor::{
    AudioDeviceService, MockAudioSystem, MockFileSystem, MockSystemService, SystemServiceInterface,
};
use std::path::PathBuf;

/// Integration tests for the complete dependency injection architecture
/// These tests verify that all components work together seamlessly

#[cfg(test)]
mod integration_tests {
    use super::*;
    use audio_device_monitor::{AudioDevice, DeviceType};

    /// Test fixture that creates a complete test environment
    struct ServiceTestFixture {
        pub audio_system: MockAudioSystem,
        pub file_system: MockFileSystem,
        pub system_service: MockSystemService,
        pub config_path: PathBuf,
    }

    impl ServiceTestFixture {
        fn new() -> Self {
            let audio_system = MockAudioSystem::new();
            let file_system = MockFileSystem::new();
            let system_service = MockSystemService::new();
            let config_path = PathBuf::from("/test/integration_config.toml");

            Self {
                audio_system,
                file_system,
                system_service,
                config_path,
            }
        }

        fn setup_default_config(&self) {
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
name = "Built-in Speakers"
weight = 50
match_type = "exact"
enabled = true

[[input_devices]]
name = "Premium Microphone"
weight = 100
match_type = "exact"
enabled = true

[[input_devices]]
name = "Built-in Microphone"
weight = 50
match_type = "exact"
enabled = true
"#;
            self.file_system
                .add_file(&self.config_path, config_content.to_string());
        }

        fn setup_test_devices(&self) {
            // Add output devices
            let premium_headphones = AudioDevice::new(
                "premium-1".to_string(),
                "Premium Headphones".to_string(),
                DeviceType::Output,
            );
            let built_in_speakers = AudioDevice::new(
                "builtin-out-1".to_string(),
                "Built-in Speakers".to_string(),
                DeviceType::Output,
            );

            // Add input devices
            let premium_mic = AudioDevice::new(
                "premium-mic-1".to_string(),
                "Premium Microphone".to_string(),
                DeviceType::Input,
            );
            let built_in_mic = AudioDevice::new(
                "builtin-mic-1".to_string(),
                "Built-in Microphone".to_string(),
                DeviceType::Input,
            );

            self.audio_system.add_device(premium_headphones);
            self.audio_system.add_device(built_in_speakers);
            self.audio_system.add_device(premium_mic);
            self.audio_system.add_device(built_in_mic);
        }

        fn create_service(
            &self,
        ) -> Result<AudioDeviceService<MockAudioSystem, MockFileSystem, MockSystemService>>
        {
            AudioDeviceService::new(
                self.audio_system.clone(),
                self.file_system.clone(),
                self.system_service.clone(),
                self.config_path.clone(),
            )
        }
    }

    #[test]
    fn test_service_creation_and_initialization() {
        let fixture = ServiceTestFixture::new();
        fixture.setup_default_config();
        fixture.setup_test_devices();

        let service = fixture.create_service();
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(service.get_config().general.check_interval_ms, 1000);
        assert_eq!(service.get_config().output_devices.len(), 2);
        assert_eq!(service.get_config().input_devices.len(), 2);

        // Verify device enumeration works
        let devices = service.enumerate_devices().unwrap();
        assert_eq!(devices.len(), 4);

        // Verify we have the expected devices
        assert!(devices.iter().any(|d| d.name == "Premium Headphones"));
        assert!(devices.iter().any(|d| d.name == "Built-in Speakers"));
        assert!(devices.iter().any(|d| d.name == "Premium Microphone"));
        assert!(devices.iter().any(|d| d.name == "Built-in Microphone"));
    }

    #[test]
    fn test_device_connection_handling() {
        let fixture = ServiceTestFixture::new();
        fixture.setup_default_config();
        fixture.setup_test_devices();

        let mut service = fixture.create_service().unwrap();

        // Initially, devices should be available
        let devices = service.enumerate_devices().unwrap();
        assert_eq!(devices.len(), 4);

        // Test handling device connection
        let result = service.handle_device_connected("Premium Headphones");
        assert!(result.is_ok());

        // Verify the audio system received calls
        let audio_calls = fixture.audio_system.get_enumerate_calls();
        assert!(audio_calls > 0);
    }

    #[test]
    fn test_device_switching() {
        let fixture = ServiceTestFixture::new();
        fixture.setup_default_config();
        fixture.setup_test_devices();

        let mut service = fixture.create_service().unwrap();

        // Test switching to premium headphones
        let result = service.set_output_device("Premium Headphones");
        assert!(result.is_ok());

        // Test switching to premium microphone
        let result = service.set_input_device("Premium Microphone");
        assert!(result.is_ok());

        // Verify audio system received switching calls
        let switch_calls = fixture.audio_system.get_set_default_output_calls();
        assert!(switch_calls.len() > 0);

        let input_switch_calls = fixture.audio_system.get_set_default_input_calls();
        assert!(input_switch_calls.len() > 0);
    }

    #[test]
    fn test_configuration_hot_reload() {
        let fixture = ServiceTestFixture::new();
        fixture.setup_default_config();
        fixture.setup_test_devices();

        let mut service = fixture.create_service().unwrap();

        // Verify initial configuration
        assert_eq!(service.get_config().general.check_interval_ms, 1000);

        // Update configuration in the mock file system
        let updated_config = r#"
[general]
check_interval_ms = 2000
log_level = "debug"
daemon_mode = true

[notifications]
show_device_availability = false
show_switching_actions = false

[[output_devices]]
name = "Premium Headphones"
weight = 200
match_type = "exact"
enabled = true
"#;
        fixture
            .file_system
            .add_file(&fixture.config_path, updated_config.to_string());

        // Trigger configuration reload
        let result = service.reload_config();
        assert!(result.is_ok());

        // Verify configuration was updated
        assert_eq!(service.get_config().general.check_interval_ms, 2000);
        assert_eq!(service.get_config().general.log_level, "debug");
        assert!(service.get_config().general.daemon_mode);
        assert_eq!(service.get_config().output_devices.len(), 1);
        assert_eq!(service.get_config().output_devices[0].weight, 200);
    }

    #[test]
    fn test_service_lifecycle_management() {
        let fixture = ServiceTestFixture::new();
        fixture.setup_default_config();
        fixture.setup_test_devices();

        let service = fixture.create_service().unwrap();

        // Test service state
        assert!(service.should_continue_running());

        // Test process ID access
        let pid = service.get_process_id();
        assert!(pid > 0);

        // Test graceful shutdown
        fixture.system_service.stop_service();
        assert!(!service.should_continue_running());
    }

    #[test]
    fn test_error_handling_missing_device() {
        let fixture = ServiceTestFixture::new();
        fixture.setup_default_config();
        // Note: Not setting up devices to test error handling

        let mut service = fixture.create_service().unwrap();

        // Test switching to non-existent device
        let result = service.set_output_device("Non-existent Device");
        assert!(result.is_err());

        let result = service.set_input_device("Non-existent Device");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_invalid_config() {
        let fixture = ServiceTestFixture::new();

        // Add invalid TOML configuration
        let invalid_config = r#"
[general
check_interval_ms = "not a number"
invalid_field = true
"#;
        fixture
            .file_system
            .add_file(&fixture.config_path, invalid_config.to_string());

        // Service creation should fail with invalid config
        let result = fixture.create_service();
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_system_interactions() {
        let fixture = ServiceTestFixture::new();
        fixture.setup_default_config();
        fixture.setup_test_devices();

        let mut service = fixture.create_service().unwrap();

        // Perform various operations to test mock interactions
        let _devices = service.enumerate_devices().unwrap();
        let _result = service.set_output_device("Premium Headphones");
        let _result = service.reload_config();

        // Verify mock systems tracked the calls
        assert!(fixture.audio_system.get_enumerate_calls() > 0);
        assert!(fixture.file_system.get_read_calls().len() > 0);
        assert!(fixture.system_service.get_process_id() > 0);

        // Verify file system operations
        let read_calls = fixture.file_system.get_read_calls();
        assert!(read_calls.iter().any(|path| path == &fixture.config_path));
    }

    #[test]
    fn test_complete_workflow_simulation() {
        let fixture = ServiceTestFixture::new();
        fixture.setup_default_config();
        fixture.setup_test_devices();

        let mut service = fixture.create_service().unwrap();

        // Simulate a complete workflow:

        // 1. Start with device enumeration
        let devices = service.enumerate_devices().unwrap();
        assert_eq!(devices.len(), 4);

        // 2. Connect a high-priority device
        service
            .handle_device_connected("Premium Headphones")
            .unwrap();

        // 3. Verify the service would switch to it
        service.set_output_device("Premium Headphones").unwrap();

        // 4. Update configuration to change priorities
        let updated_config = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = true
show_switching_actions = true

[[output_devices]]
name = "Built-in Speakers"
weight = 150
match_type = "exact"
enabled = true

[[output_devices]]
name = "Premium Headphones"
weight = 100
match_type = "exact"
enabled = true
"#;
        fixture
            .file_system
            .add_file(&fixture.config_path, updated_config.to_string());

        // 5. Reload configuration
        service.reload_config().unwrap();

        // 6. Verify priority change took effect
        assert_eq!(
            service.get_config().output_devices[0].name,
            "Built-in Speakers"
        );
        assert_eq!(service.get_config().output_devices[0].weight, 150);
        assert_eq!(service.get_config().output_devices[1].weight, 100);

        // 7. Test graceful shutdown
        assert!(service.should_continue_running());
        fixture.system_service.stop_service();
        assert!(!service.should_continue_running());

        // Verify all mock systems were properly exercised
        assert!(fixture.audio_system.get_enumerate_calls() > 0);
        assert!(fixture.file_system.get_read_calls().len() > 0);
        assert!(fixture.system_service.get_process_id() > 0);
    }
}
