use audio_device_monitor::{ConfigLoader, MockFileSystem};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Integration tests for ConfigLoader with file system abstraction
/// These tests verify configuration loading, validation, and hot reload capabilities

#[cfg(test)]
mod config_loader_tests {
    use super::*;

    #[test]
    fn test_config_loading_with_mock_filesystem() {
        let file_system = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");

        let config_content = r#"
[general]
check_interval_ms = 1500
log_level = "debug"
daemon_mode = true

[notifications]
show_device_availability = true
show_switching_actions = false

[[output_devices]]
name = "Test Device"
weight = 75
match_type = "contains"
enabled = true
"#;
        file_system.add_file(&config_path, config_content.to_string());

        let config_loader = ConfigLoader::new(file_system.clone(), config_path.clone());
        let config = config_loader.load_config().unwrap();

        assert_eq!(config.general.check_interval_ms, 1500);
        assert_eq!(config.general.log_level, "debug");
        assert!(config.general.daemon_mode);
        assert!(config.notifications.show_device_availability);
        assert!(!config.notifications.show_switching_actions);
        assert_eq!(config.output_devices.len(), 1);
        assert_eq!(config.output_devices[0].name, "Test Device");
        assert_eq!(config.output_devices[0].weight, 75);

        // Verify file system was called
        let read_calls = file_system.get_read_calls();
        assert_eq!(read_calls.len(), 1);
        assert_eq!(read_calls[0], config_path);
    }

    #[test]
    fn test_config_modification_detection() {
        let file_system = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");

        let initial_config = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
"#;
        file_system.add_file(&config_path, initial_config.to_string());

        let config_loader = ConfigLoader::new(file_system.clone(), config_path.clone());

        // Load initial config
        let _config = config_loader.load_config().unwrap();

        // Get the actual modification time of the created file
        let initial_time = file_system
            .modification_times
            .lock()
            .unwrap()
            .get(&config_path)
            .copied()
            .unwrap_or(SystemTime::UNIX_EPOCH);

        // Simulate file being unchanged (same modification time)
        assert!(!config_loader.is_config_modified(initial_time).unwrap());

        // Wait a tiny bit to ensure time difference, then update the file
        std::thread::sleep(Duration::from_millis(1));

        // Update the file (mock file system automatically updates modification time)
        let updated_config = r#"
[general]
check_interval_ms = 2000
log_level = "debug"
daemon_mode = true

[notifications]
show_device_availability = true
show_switching_actions = false
"#;
        file_system.add_file(&config_path, updated_config.to_string());

        // Now modification should be detected
        assert!(config_loader.is_config_modified(initial_time).unwrap());

        // Load updated config
        let updated_config = config_loader.load_config().unwrap();
        assert_eq!(updated_config.general.check_interval_ms, 2000);
        assert_eq!(updated_config.general.log_level, "debug");
        assert!(updated_config.general.daemon_mode);
    }

    #[test]
    fn test_config_error_handling() {
        let file_system = MockFileSystem::new();
        let config_path = PathBuf::from("/test/invalid_config.toml");

        // Test file read failure (ConfigLoader creates default config if file doesn't exist)
        file_system.set_read_failure(true);
        file_system.add_file(&config_path, "some content".to_string());
        let config_loader = ConfigLoader::new(file_system.clone(), config_path.clone());
        let result = config_loader.load_config();
        assert!(result.is_err());

        // Reset for next test
        file_system.set_read_failure(false);

        // Test invalid TOML
        let invalid_toml = r#"
[general
check_interval_ms = "not a number"
invalid syntax here
"#;
        file_system.add_file(&config_path, invalid_toml.to_string());

        let result = config_loader.load_config();
        assert!(result.is_err());

        // Test missing required fields
        let incomplete_config = r#"
[general]
check_interval_ms = 1000
# Missing log_level and daemon_mode
"#;
        file_system.add_file(&config_path, incomplete_config.to_string());

        let result = config_loader.load_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validation() {
        let file_system = MockFileSystem::new();
        let config_path = PathBuf::from("/test/validation_config.toml");

        // Test configuration with various edge cases
        let edge_case_config = r#"
[general]
check_interval_ms = 0
log_level = "trace"
daemon_mode = false

[notifications]
show_device_availability = true
show_switching_actions = true

[[output_devices]]
name = ""
weight = 0
match_type = "exact"
enabled = false

[[output_devices]]
name = "Very Long Device Name That Might Cause Issues In Some Systems"
weight = 999999
match_type = "contains"
enabled = true

[[input_devices]]
name = "Special Characters: !@#$%^&*()"
weight = 50
match_type = "exact"
enabled = true
"#;
        file_system.add_file(&config_path, edge_case_config.to_string());

        let config_loader = ConfigLoader::new(file_system.clone(), config_path.clone());
        let config = config_loader.load_config().unwrap();

        // Verify edge cases are handled
        assert_eq!(config.general.check_interval_ms, 0);
        assert_eq!(config.general.log_level, "trace");
        assert_eq!(config.output_devices.len(), 2);
        assert_eq!(config.input_devices.len(), 1);

        // Verify empty name and zero weight are preserved
        assert_eq!(config.output_devices[0].name, "");
        assert_eq!(config.output_devices[0].weight, 0);
        assert!(!config.output_devices[0].enabled);

        // Verify high weight is preserved
        assert_eq!(config.output_devices[1].weight, 999999);

        // Verify special characters in device name
        assert_eq!(
            config.input_devices[0].name,
            "Special Characters: !@#$%^&*()"
        );
    }

    #[test]
    fn test_multiple_config_reloads() {
        let file_system = MockFileSystem::new();
        let config_path = PathBuf::from("/test/multi_reload_config.toml");

        let config_loader = ConfigLoader::new(file_system.clone(), config_path.clone());

        // First configuration
        let config1 = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Device A"
weight = 100
match_type = "exact"
enabled = true
"#;
        file_system.add_file(&config_path, config1.to_string());

        let config = config_loader.load_config().unwrap();
        assert_eq!(config.general.check_interval_ms, 1000);
        assert_eq!(config.output_devices.len(), 1);
        assert_eq!(config.output_devices[0].name, "Device A");

        // Second configuration
        let config2 = r#"
[general]
check_interval_ms = 2000
log_level = "debug"
daemon_mode = true

[notifications]
show_device_availability = true
show_switching_actions = false

[[output_devices]]
name = "Device B"
weight = 200
match_type = "contains"
enabled = true

[[output_devices]]
name = "Device C"
weight = 150
match_type = "exact"
enabled = false
"#;
        file_system.add_file(&config_path, config2.to_string());

        let config = config_loader.load_config().unwrap();
        assert_eq!(config.general.check_interval_ms, 2000);
        assert_eq!(config.general.log_level, "debug");
        assert!(config.general.daemon_mode);
        assert_eq!(config.output_devices.len(), 2);
        assert_eq!(config.output_devices[0].name, "Device B");
        assert_eq!(config.output_devices[1].name, "Device C");

        // Third configuration (back to minimal)
        let config3 = r#"
[general]
check_interval_ms = 500
log_level = "warn"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = false
"#;
        file_system.add_file(&config_path, config3.to_string());

        let config = config_loader.load_config().unwrap();
        assert_eq!(config.general.check_interval_ms, 500);
        assert_eq!(config.general.log_level, "warn");
        assert!(!config.general.daemon_mode);
        assert_eq!(config.output_devices.len(), 0);
        assert_eq!(config.input_devices.len(), 0);

        // Verify all reads were tracked
        let read_calls = file_system.get_read_calls();
        assert_eq!(read_calls.len(), 3);
    }

    #[test]
    fn test_config_path_handling() {
        let file_system = MockFileSystem::new();
        let config_path = PathBuf::from("/complex/path/with/subdirs/config.toml");

        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
"#;
        file_system.add_file(&config_path, config_content.to_string());

        let config_loader = ConfigLoader::new(file_system.clone(), config_path.clone());

        // Verify path is stored correctly
        assert_eq!(config_loader.get_config_path(), &config_path);

        // Verify loading works with complex path
        let config = config_loader.load_config().unwrap();
        assert_eq!(config.general.check_interval_ms, 1000);

        // Verify the correct path was accessed
        let read_calls = file_system.get_read_calls();
        assert_eq!(read_calls.len(), 1);
        assert_eq!(read_calls[0], config_path);
    }

    #[test]
    fn test_concurrent_config_access() {
        let file_system = MockFileSystem::new();
        let config_path = PathBuf::from("/test/concurrent_config.toml");

        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Shared Device"
weight = 100
match_type = "exact"
enabled = true
"#;
        file_system.add_file(&config_path, config_content.to_string());

        // Create multiple config loaders sharing the same file system
        let config_loader1 = ConfigLoader::new(file_system.clone(), config_path.clone());
        let config_loader2 = ConfigLoader::new(file_system.clone(), config_path.clone());

        // Both should be able to load the configuration
        let config1 = config_loader1.load_config().unwrap();
        let config2 = config_loader2.load_config().unwrap();

        assert_eq!(
            config1.general.check_interval_ms,
            config2.general.check_interval_ms
        );
        assert_eq!(config1.output_devices.len(), config2.output_devices.len());
        assert_eq!(
            config1.output_devices[0].name,
            config2.output_devices[0].name
        );

        // Verify both loaders accessed the file
        let read_calls = file_system.get_read_calls();
        assert_eq!(read_calls.len(), 2);
        assert!(read_calls.iter().all(|path| path == &config_path));
    }
}
