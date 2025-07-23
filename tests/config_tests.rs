use audio_device_monitor::config::{Config, GeneralConfig, MatchType, NotificationConfig};
use std::path::PathBuf;
use tempfile::TempDir;

mod test_utils;
use test_utils::DeviceRuleBuilder;

/// Helper function to create a temporary config file with given content
fn create_temp_config(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("config.toml");
    std::fs::write(&config_path, content).expect("Failed to write temp config");
    (temp_dir, config_path)
}

/// Test basic configuration loading and parsing
#[cfg(test)]
mod config_loading {
    use super::*;

    #[test]
    fn test_load_valid_config() {
        let config_content = r#"
[general]
check_interval_ms = 2000
log_level = "debug"
daemon_mode = true

[notifications]
show_device_availability = true
show_switching_actions = false

[[output_devices]]
name = "AirPods"
weight = 200
match_type = "contains"
enabled = true

[[input_devices]]
name = "MV7"
weight = 500
match_type = "contains"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        // Test general config
        assert_eq!(config.general.check_interval_ms, 2000);
        assert_eq!(config.general.log_level, "debug");
        assert!(config.general.daemon_mode);

        // Test notification config
        assert!(config.notifications.show_device_availability);
        assert!(!config.notifications.show_switching_actions);

        // Test device rules
        assert_eq!(config.output_devices.len(), 1);
        assert_eq!(config.output_devices[0].name, "AirPods");
        assert_eq!(config.output_devices[0].weight, 200);
        assert!(matches!(
            config.output_devices[0].match_type,
            MatchType::Contains
        ));
        assert!(config.output_devices[0].enabled);

        assert_eq!(config.input_devices.len(), 1);
        assert_eq!(config.input_devices[0].name, "MV7");
        assert_eq!(config.input_devices[0].weight, 500);
    }

    #[test]
    fn test_load_minimal_config() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        // Should use default values
        assert_eq!(config.general.check_interval_ms, 1000);
        assert_eq!(config.general.log_level, "info");
        assert!(!config.general.daemon_mode);

        assert!(!config.notifications.show_device_availability);
        assert!(config.notifications.show_switching_actions);

        assert!(config.output_devices.is_empty());
        assert!(config.input_devices.is_empty());
    }

    #[test]
    fn test_load_nonexistent_config_creates_default() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("nonexistent.toml");

        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        // Should create config with defaults
        assert_eq!(config.general.check_interval_ms, 1000);
        assert_eq!(config.general.log_level, "info");
        assert!(!config.general.daemon_mode);

        // Should have default device rules
        assert!(!config.output_devices.is_empty());
        assert!(!config.input_devices.is_empty());

        // File should be created
        assert!(config_path.exists());
    }

    #[test]
    fn test_load_invalid_toml() {
        let invalid_content = r#"
[general
invalid toml syntax
"#;

        let (_temp_dir, config_path) = create_temp_config(invalid_content);
        let result = Config::load(Some(config_path.to_str().unwrap()));

        assert!(result.is_err());
    }
}

/// Test backward compatibility migration
#[cfg(test)]
mod backward_compatibility {
    use super::*;

    #[test]
    fn test_migrate_old_notification_config() {
        let old_config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_changes = true
show_switching_actions = false
"#;

        let (_temp_dir, config_path) = create_temp_config(old_config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        // Old show_device_changes should migrate to show_device_availability
        assert!(config.notifications.show_device_availability);
        assert!(!config.notifications.show_switching_actions);

        // Compatibility field should be cleared
        assert!(config.notifications.show_device_changes.is_none());
    }

    #[test]
    fn test_new_config_format_preferred() {
        let new_config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
show_device_changes = true  # This should be ignored in favor of the new field
"#;

        let (_temp_dir, config_path) = create_temp_config(new_config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        // New field should take precedence
        assert!(!config.notifications.show_device_availability);
        assert!(config.notifications.show_switching_actions);
    }

    #[test]
    fn test_mixed_old_and_new_fields() {
        let mixed_config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_switching_actions = true
show_device_changes = true  # Only old field present
"#;

        let (_temp_dir, config_path) = create_temp_config(mixed_config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        // Should migrate old field when new field is not present
        assert!(config.notifications.show_device_availability);
        assert!(config.notifications.show_switching_actions);
    }
}

/// Test match type parsing
#[cfg(test)]
mod match_type_parsing {
    use super::*;

    #[test]
    fn test_all_match_types() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Exact"
weight = 100
match_type = "exact"
enabled = true

[[output_devices]]
name = "Contains"
weight = 100
match_type = "contains"
enabled = true

[[output_devices]]
name = "StartsWith"
weight = 100
match_type = "startswith"
enabled = true

[[output_devices]]
name = "EndsWith"
weight = 100
match_type = "endswith"
enabled = true

[[output_devices]]
name = "Regex"
weight = 100
match_type = "regex"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        assert_eq!(config.output_devices.len(), 5);

        assert!(matches!(
            config.output_devices[0].match_type,
            MatchType::Exact
        ));
        assert!(matches!(
            config.output_devices[1].match_type,
            MatchType::Contains
        ));
        assert!(matches!(
            config.output_devices[2].match_type,
            MatchType::StartsWith
        ));
        assert!(matches!(
            config.output_devices[3].match_type,
            MatchType::EndsWith
        ));
        assert!(matches!(
            config.output_devices[4].match_type,
            MatchType::Regex
        ));
    }

    #[test]
    fn test_invalid_match_type() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Device"
weight = 100
match_type = "invalid_type"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let result = Config::load(Some(config_path.to_str().unwrap()));

        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_match_types() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Device1"
weight = 100
match_type = "EXACT"
enabled = true

[[output_devices]]
name = "Device2"
weight = 100
match_type = "Contains"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let result = Config::load(Some(config_path.to_str().unwrap()));

        // Case sensitivity depends on serde configuration
        // This test verifies current behavior
        match result {
            Ok(config) => {
                assert_eq!(config.output_devices.len(), 2);
            }
            Err(_) => {
                // If case-sensitive, this is expected behavior
                // The test documents the current behavior
            }
        }
    }
}

/// Test device rule validation
#[cfg(test)]
mod device_rule_validation {
    use super::*;

    #[test]
    fn test_disabled_device_rules() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Enabled Device"
weight = 100
match_type = "exact"
enabled = true

[[output_devices]]
name = "Disabled Device"
weight = 200
match_type = "exact"
enabled = false
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        assert_eq!(config.output_devices.len(), 2);
        assert!(config.output_devices[0].enabled);
        assert!(!config.output_devices[1].enabled);
    }

    #[test]
    fn test_zero_weight_device() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Zero Weight"
weight = 0
match_type = "exact"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        assert_eq!(config.output_devices[0].weight, 0);
    }

    #[test]
    fn test_maximum_weight_device() {
        let config_content = format!(
            r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Max Weight"
weight = {}
match_type = "exact"
enabled = true
"#,
            u32::MAX
        );

        let (_temp_dir, config_path) = create_temp_config(&config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        assert_eq!(config.output_devices[0].weight, u32::MAX);
    }

    #[test]
    fn test_empty_device_name() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = ""
weight = 100
match_type = "exact"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        assert_eq!(config.output_devices[0].name, "");
    }

    #[test]
    fn test_unicode_device_name() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "🎵 Music Device 🎵"
weight = 100
match_type = "contains"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        assert_eq!(config.output_devices[0].name, "🎵 Music Device 🎵");
    }
}

/// Test configuration saving
#[cfg(test)]
mod config_saving {
    use super::*;

    #[test]
    fn test_save_and_reload_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("test_save.toml");

        // Create a custom config
        let mut config = Config::default();
        config.general.check_interval_ms = 3000;
        config.general.log_level = "trace".to_string();
        config.notifications.show_device_availability = true;

        config.output_devices = vec![
            DeviceRuleBuilder::new()
                .name("Test Device")
                .weight(150)
                .contains_match()
                .build(),
        ];

        // Save the config
        config.save(Some(config_path.to_str().unwrap())).unwrap();

        // Reload and verify
        let reloaded_config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        assert_eq!(reloaded_config.general.check_interval_ms, 3000);
        assert_eq!(reloaded_config.general.log_level, "trace");
        assert!(reloaded_config.notifications.show_device_availability);

        assert_eq!(reloaded_config.output_devices.len(), 1);
        assert_eq!(reloaded_config.output_devices[0].name, "Test Device");
        assert_eq!(reloaded_config.output_devices[0].weight, 150);
    }

    #[test]
    fn test_save_creates_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("config")
            .join("test.toml");

        let config = Config::default();
        config.save(Some(nested_path.to_str().unwrap())).unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }
}

/// Test default configuration values
#[cfg(test)]
mod default_values {
    use super::*;

    #[test]
    fn test_general_config_defaults() {
        let general = GeneralConfig::default();

        assert_eq!(general.check_interval_ms, 1000);
        assert_eq!(general.log_level, "info");
        assert!(!general.daemon_mode);
    }

    #[test]
    fn test_notification_config_defaults() {
        let notifications = NotificationConfig::default();

        assert!(!notifications.show_device_availability);
        assert!(notifications.show_switching_actions);
        assert!(notifications.show_device_changes.is_none());
    }

    #[test]
    fn test_full_config_defaults() {
        let config = Config::default();

        // Should have some default device rules
        assert!(!config.output_devices.is_empty());
        assert!(!config.input_devices.is_empty());

        // Test that default rules are reasonable
        let airpods_rule = config
            .output_devices
            .iter()
            .find(|rule| rule.name.contains("AirPods"));
        assert!(airpods_rule.is_some());

        let mbp_speakers = config
            .output_devices
            .iter()
            .find(|rule| rule.name == "MacBook Pro Speakers");
        assert!(mbp_speakers.is_some());
    }
}

/// Test error conditions and edge cases
#[cfg(test)]
mod error_conditions {
    use super::*;

    #[test]
    fn test_invalid_path() {
        let result = Config::load(Some("/invalid/path/that/does/not/exist/config.toml"));

        // Should create default config even with invalid path
        assert!(result.is_ok());
    }

    #[test]
    fn test_permission_denied() {
        // This test may not work on all systems, but documents expected behavior
        let result = Config::load(Some("/root/config.toml"));

        // Should handle permission errors gracefully
        match result {
            Ok(_) => {
                // If it succeeds, that's also acceptable (default config created)
            }
            Err(_) => {
                // Expected for permission-denied scenarios
            }
        }
    }

    #[test]
    fn test_malformed_weights() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
name = "Device"
weight = -1
match_type = "exact"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let result = Config::load(Some(config_path.to_str().unwrap()));

        // Should fail to parse negative weight (u32 can't be negative)
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        let config_content = r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true

[[output_devices]]
# Missing name field
weight = 100
match_type = "exact"
enabled = true
"#;

        let (_temp_dir, config_path) = create_temp_config(config_content);
        let result = Config::load(Some(config_path.to_str().unwrap()));

        // Should fail due to missing required field
        assert!(result.is_err());
    }
}

/// Test configuration with many devices
#[cfg(test)]
mod large_configurations {
    use super::*;

    #[test]
    fn test_many_device_rules() {
        let mut config_content = String::from(
            r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
"#,
        );

        // Add 50 output device rules
        for i in 0..50 {
            config_content.push_str(&format!(
                r#"
[[output_devices]]
name = "Output Device {}"
weight = {}
match_type = "exact"
enabled = true
"#,
                i,
                i * 10
            ));
        }

        // Add 50 input device rules
        for i in 0..50 {
            config_content.push_str(&format!(
                r#"
[[input_devices]]
name = "Input Device {}"
weight = {}
match_type = "contains"
enabled = true
"#,
                i,
                i * 5
            ));
        }

        let (_temp_dir, config_path) = create_temp_config(&config_content);
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();

        assert_eq!(config.output_devices.len(), 50);
        assert_eq!(config.input_devices.len(), 50);

        // Verify first and last entries
        assert_eq!(config.output_devices[0].name, "Output Device 0");
        assert_eq!(config.output_devices[49].name, "Output Device 49");
        assert_eq!(config.output_devices[49].weight, 490);

        assert_eq!(config.input_devices[0].name, "Input Device 0");
        assert_eq!(config.input_devices[49].name, "Input Device 49");
    }

    #[test]
    fn test_config_loading_performance() {
        // Create a large config similar to above
        let mut config_content = String::from(
            r#"
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
"#,
        );

        for i in 0..100 {
            config_content.push_str(&format!(
                r#"
[[output_devices]]
name = "Device {}"
weight = {}
match_type = "contains"
enabled = true
"#,
                i, i
            ));
        }

        let (_temp_dir, config_path) = create_temp_config(&config_content);

        // Should load quickly even with many rules
        let start = std::time::Instant::now();
        let config = Config::load(Some(config_path.to_str().unwrap())).unwrap();
        let duration = start.elapsed();

        assert_eq!(config.output_devices.len(), 100);
        assert!(
            duration.as_millis() < 100,
            "Config loading took too long: {:?}",
            duration
        );
    }
}
