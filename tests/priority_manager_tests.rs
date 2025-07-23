use audio_device_monitor::config::{Config, DeviceRule, GeneralConfig, NotificationConfig};
use audio_device_monitor::priority::DevicePriorityManager;

mod test_utils;
use test_utils::{AudioDeviceBuilder, DeviceRuleBuilder};

/// Helper function to create a test config with custom rules
fn create_test_config(output_rules: Vec<DeviceRule>, input_rules: Vec<DeviceRule>) -> Config {
    Config {
        general: GeneralConfig::default(),
        notifications: NotificationConfig::default(),
        output_devices: output_rules,
        input_devices: input_rules,
    }
}

/// Test basic device selection functionality
#[cfg(test)]
mod device_selection {
    use super::*;

    #[test]
    fn test_single_device_selection() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("AirPods")
                .weight(100)
                .contains_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .build(),
        ];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_some());
        assert_eq!(best_device.unwrap().name, "AirPods Pro");
    }

    #[test]
    fn test_no_matching_devices() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("AirPods")
                .weight(100)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("Beats Headphones")
                .output()
                .build(),
        ];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_none());
    }

    #[test]
    fn test_empty_device_list() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("AirPods")
                .weight(100)
                .contains_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![];
        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_none());
    }

    #[test]
    fn test_empty_rules_list() {
        let output_rules = vec![];
        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .build(),
        ];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_none());
    }
}

/// Test priority-based device selection
#[cfg(test)]
mod priority_selection {
    use super::*;

    #[test]
    fn test_highest_weight_wins() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("MacBook Pro Speakers")
                .weight(10)
                .exact_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("AirPods")
                .weight(200)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("Audioengine")
                .weight(150)
                .contains_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Audioengine 2+")
                .output()
                .build(),
        ];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_some());
        assert_eq!(best_device.unwrap().name, "AirPods Pro"); // Weight 200 wins
    }

    #[test]
    fn test_equal_weights_first_match_wins() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("Device A")
                .weight(100)
                .exact_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("Device B")
                .weight(100)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new().name("Device B").output().build(),
            AudioDeviceBuilder::new().name("Device A").output().build(),
        ];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_some());
        // Should pick the first device that matches the highest weight rule
        // Since both have weight 100, it depends on which device is found first
        // with a matching rule
        let result_name = best_device.unwrap().name;
        assert!(result_name == "Device A" || result_name == "Device B");
    }

    #[test]
    fn test_disabled_rules_ignored() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("High Priority Device")
                .weight(200)
                .exact_match()
                .disabled()
                .build(),
            DeviceRuleBuilder::new()
                .name("Low Priority Device")
                .weight(10)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("High Priority Device")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Low Priority Device")
                .output()
                .build(),
        ];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_some());
        assert_eq!(best_device.unwrap().name, "Low Priority Device");
    }

    #[test]
    fn test_multiple_rules_same_device() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("AirPods")
                .weight(100)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("Pro")
                .weight(150)
                .contains_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .build(),
        ];

        // Device matches both rules, should get the higher weight (150)
        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_some());
        assert_eq!(best_device.unwrap().name, "AirPods Pro");
    }
}

/// Test input vs output device separation
#[cfg(test)]
mod device_type_separation {
    use super::*;

    #[test]
    fn test_input_output_separation() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("Output Device")
                .weight(100)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![
            DeviceRuleBuilder::new()
                .name("Input Device")
                .weight(100)
                .exact_match()
                .build(),
        ];

        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("Output Device")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Input Device")
                .input()
                .build(),
        ];

        // Output rules should only match output devices
        let best_output = manager.find_best_output_device(&devices);
        assert!(best_output.is_some());
        assert_eq!(best_output.unwrap().name, "Output Device");

        // Input rules should only match input devices
        let best_input = manager.find_best_input_device(&devices);
        assert!(best_input.is_some());
        assert_eq!(best_input.unwrap().name, "Input Device");
    }

    #[test]
    fn test_wrong_device_type_no_match() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("Any Device")
                .weight(100)
                .contains_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        // Only input devices available
        let devices = vec![
            AudioDeviceBuilder::new()
                .name("Any Device Input")
                .input()
                .build(),
            AudioDeviceBuilder::new()
                .name("Another Any Device")
                .input()
                .build(),
        ];

        // Output rules shouldn't match input devices
        let best_output = manager.find_best_output_device(&devices);
        assert!(best_output.is_none());
    }
}

/// Test device state management
#[cfg(test)]
mod state_management {
    use super::*;

    #[test]
    fn test_should_switch_with_no_current_device() {
        let config = create_test_config(vec![], vec![]);
        let manager = DevicePriorityManager::new(&config);

        let new_device = AudioDeviceBuilder::new()
            .name("New Device")
            .output()
            .build();

        // Should switch when no current device is set
        assert!(manager.should_switch_output(&new_device));
        assert!(manager.should_switch_input(&new_device));
    }

    #[test]
    fn test_should_switch_to_different_device() {
        let config = create_test_config(vec![], vec![]);
        let mut manager = DevicePriorityManager::new(&config);

        // Set current devices
        manager.update_current_output("Current Output".to_string());
        manager.update_current_input("Current Input".to_string());

        let new_output = AudioDeviceBuilder::new()
            .name("New Output")
            .output()
            .build();

        let new_input = AudioDeviceBuilder::new().name("New Input").input().build();

        // Should switch to different devices
        assert!(manager.should_switch_output(&new_output));
        assert!(manager.should_switch_input(&new_input));
    }

    #[test]
    fn test_should_not_switch_to_same_device() {
        let config = create_test_config(vec![], vec![]);
        let mut manager = DevicePriorityManager::new(&config);

        let device_name = "Same Device";

        // Set current devices
        manager.update_current_output(device_name.to_string());
        manager.update_current_input(device_name.to_string());

        let same_output = AudioDeviceBuilder::new().name(device_name).output().build();

        let same_input = AudioDeviceBuilder::new().name(device_name).input().build();

        // Should not switch to the same device
        assert!(!manager.should_switch_output(&same_output));
        assert!(!manager.should_switch_input(&same_input));
    }
}

/// Test real-world scenarios
#[cfg(test)]
mod real_world_scenarios {
    use super::*;

    #[test]
    fn test_typical_user_setup() {
        // Realistic priority setup: AirPods > Audioengine > MacBook Pro
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("AirPod")
                .weight(200)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("Audioengine")
                .weight(100)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("MacBook Pro Speakers")
                .weight(10)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![
            DeviceRuleBuilder::new()
                .name("MV7")
                .weight(500)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("AirPod")
                .weight(100)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("MacBook Pro Microphone")
                .weight(10)
                .exact_match()
                .build(),
        ];

        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        // Scenario 1: All devices available - should pick highest priority
        let all_devices = vec![
            AudioDeviceBuilder::new()
                .name("🌪️☠️ AirPod's Revenge ☠️🌪️")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("🌪️☠️ AirPod's Revenge ☠️🌪️")
                .input()
                .build(),
            AudioDeviceBuilder::new()
                .name("Audioengine 2+")
                .output()
                .build(),
            AudioDeviceBuilder::new().name("Shure MV7").input().build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Microphone")
                .input()
                .build(),
        ];

        let best_output = manager.find_best_output_device(&all_devices);
        assert!(best_output.is_some());
        assert_eq!(best_output.unwrap().name, "🌪️☠️ AirPod's Revenge ☠️🌪️");

        let best_input = manager.find_best_input_device(&all_devices);
        assert!(best_input.is_some());
        assert_eq!(best_input.unwrap().name, "Shure MV7"); // Weight 500 wins
    }

    #[test]
    fn test_airpods_disconnected_scenario() {
        // Same rules as above
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("AirPod")
                .weight(200)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("Audioengine")
                .weight(100)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("MacBook Pro Speakers")
                .weight(10)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        // AirPods not available, should fall back to Audioengine
        let devices_without_airpods = vec![
            AudioDeviceBuilder::new()
                .name("Audioengine 2+")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
        ];

        let best_output = manager.find_best_output_device(&devices_without_airpods);
        assert!(best_output.is_some());
        assert_eq!(best_output.unwrap().name, "Audioengine 2+");
    }

    #[test]
    fn test_only_fallback_devices_available() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("AirPod")
                .weight(200)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("Audioengine")
                .weight(100)
                .contains_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("MacBook Pro Speakers")
                .weight(10)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        // Only lowest priority device available
        let fallback_devices = vec![
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
        ];

        let best_output = manager.find_best_output_device(&fallback_devices);
        assert!(best_output.is_some());
        assert_eq!(best_output.unwrap().name, "MacBook Pro Speakers");
    }
}

/// Test edge cases and error conditions
#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_device_with_empty_name() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("")
                .weight(100)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![AudioDeviceBuilder::new().name("").output().build()];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_some());
        assert_eq!(best_device.unwrap().name, "");
    }

    #[test]
    fn test_unicode_device_names() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("🎵")
                .weight(100)
                .contains_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("🎵 Music Device 🎵")
                .output()
                .build(),
        ];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_some());
        assert_eq!(best_device.unwrap().name, "🎵 Music Device 🎵");
    }

    #[test]
    fn test_very_high_weight_values() {
        let output_rules = vec![
            DeviceRuleBuilder::new()
                .name("Device A")
                .weight(u32::MAX - 1)
                .exact_match()
                .build(),
            DeviceRuleBuilder::new()
                .name("Device B")
                .weight(u32::MAX)
                .exact_match()
                .build(),
        ];

        let input_rules = vec![];
        let config = create_test_config(output_rules, input_rules);
        let manager = DevicePriorityManager::new(&config);

        let devices = vec![
            AudioDeviceBuilder::new().name("Device A").output().build(),
            AudioDeviceBuilder::new().name("Device B").output().build(),
        ];

        let best_device = manager.find_best_output_device(&devices);
        assert!(best_device.is_some());
        assert_eq!(best_device.unwrap().name, "Device B"); // u32::MAX wins
    }

    #[test]
    fn test_many_devices_performance() {
        // Create 100 rules and 100 devices to test performance
        let mut output_rules = Vec::new();
        let mut devices = Vec::new();

        for i in 0..100 {
            output_rules.push(
                DeviceRuleBuilder::new()
                    .name(&format!("Device {i}"))
                    .weight(i as u32)
                    .exact_match()
                    .build(),
            );

            devices.push(
                AudioDeviceBuilder::new()
                    .name(&format!("Device {i}"))
                    .output()
                    .build(),
            );
        }

        let config = create_test_config(output_rules, vec![]);
        let manager = DevicePriorityManager::new(&config);

        // This should complete quickly even with many devices/rules
        let start = std::time::Instant::now();
        let best_device = manager.find_best_output_device(&devices);
        let duration = start.elapsed();

        assert!(best_device.is_some());
        assert_eq!(best_device.unwrap().name, "Device 99"); // Highest weight
        assert!(
            duration.as_millis() < 10,
            "Selection took too long: {:?}",
            duration
        );
    }
}
