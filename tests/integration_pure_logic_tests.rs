use audio_device_monitor::TestNotificationSender;
use audio_device_monitor::config::{Config, GeneralConfig, NotificationConfig};
use audio_device_monitor::notifications::{NotificationManager, SwitchReason};
use audio_device_monitor::priority::DevicePriorityManager;

mod test_utils;
use test_utils::{AudioDeviceBuilder, DeviceRuleBuilder};

/// Test complete device selection and notification flow
#[cfg(test)]
mod end_to_end_flows {
    use super::*;

    #[test]
    fn test_device_connection_full_flow() {
        // Create a realistic config
        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: true,
                show_switching_actions: true,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("AirPods")
                    .weight(200)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("MacBook Pro Speakers")
                    .weight(10)
                    .exact_match()
                    .build(),
            ],
            input_devices: vec![
                DeviceRuleBuilder::new()
                    .name("MV7")
                    .weight(500)
                    .contains_match()
                    .build(),
            ],
        };

        // Create components
        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        // Simulate available devices
        let available_devices = vec![
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new().name("Shure MV7").input().build(),
        ];

        // Test device selection
        let best_output = priority_manager.find_best_output_device(&available_devices);
        let best_input = priority_manager.find_best_input_device(&available_devices);

        assert!(best_output.is_some());
        assert_eq!(best_output.as_ref().unwrap().name, "MacBook Pro Speakers");
        assert!(best_input.is_some());
        assert_eq!(best_input.as_ref().unwrap().name, "Shure MV7");

        // Test notifications
        assert!(
            notification_manager
                .device_connected(&best_output.unwrap())
                .is_ok()
        );
        assert!(
            notification_manager
                .device_switched(&best_input.unwrap(), SwitchReason::HigherPriority)
                .is_ok()
        );
    }

    #[test]
    fn test_priority_upgrade_scenario() {
        // Config with clear priority hierarchy
        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: false,
                show_switching_actions: true,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("AirPods")
                    .weight(300)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("Audioengine")
                    .weight(200)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("MacBook Pro Speakers")
                    .weight(10)
                    .exact_match()
                    .build(),
            ],
            input_devices: vec![],
        };

        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        // Start with low priority devices
        let initial_devices = vec![
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Audioengine 2+")
                .output()
                .build(),
        ];

        let initial_best = priority_manager.find_best_output_device(&initial_devices);
        assert!(initial_best.is_some());
        assert_eq!(initial_best.as_ref().unwrap().name, "Audioengine 2+");

        // Higher priority device connects
        let upgraded_devices = vec![
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Audioengine 2+")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .build(),
        ];

        let upgraded_best = priority_manager.find_best_output_device(&upgraded_devices);
        assert!(upgraded_best.is_some());
        assert_eq!(upgraded_best.as_ref().unwrap().name, "AirPods Pro");

        // Should trigger switching notification
        assert!(
            notification_manager
                .device_switched(&upgraded_best.unwrap(), SwitchReason::HigherPriority)
                .is_ok()
        );
    }

    #[test]
    fn test_device_disconnection_fallback_flow() {
        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: true,
                show_switching_actions: true,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("Headphones")
                    .weight(100)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("Speakers")
                    .weight(50)
                    .contains_match()
                    .build(),
            ],
            input_devices: vec![],
        };

        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        // Start with both devices
        let full_devices = vec![
            AudioDeviceBuilder::new()
                .name("Gaming Headphones")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Desktop Speakers")
                .output()
                .build(),
        ];

        let initial_best = priority_manager.find_best_output_device(&full_devices);
        assert_eq!(initial_best.as_ref().unwrap().name, "Gaming Headphones");

        // High priority device disconnects
        let fallback_devices = vec![
            AudioDeviceBuilder::new()
                .name("Desktop Speakers")
                .output()
                .build(),
        ];

        let fallback_best = priority_manager.find_best_output_device(&fallback_devices);
        assert_eq!(fallback_best.as_ref().unwrap().name, "Desktop Speakers");

        // Should notify about disconnection and switching
        assert!(
            notification_manager
                .device_disconnected(&initial_best.unwrap())
                .is_ok()
        );
        assert!(
            notification_manager
                .device_switched(&fallback_best.unwrap(), SwitchReason::PreviousUnavailable)
                .is_ok()
        );
    }
}

/// Test configuration impact on component behavior
#[cfg(test)]
mod configuration_impact {
    use super::*;

    #[test]
    fn test_disabled_notifications_affect_all_components() {
        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: false,
                show_switching_actions: false,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("Test Device")
                    .weight(100)
                    .exact_match()
                    .build(),
            ],
            input_devices: vec![],
        };

        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        let device = AudioDeviceBuilder::new()
            .name("Test Device")
            .output()
            .build();

        // Priority manager should still work
        let devices = vec![device.clone()];
        let best = priority_manager.find_best_output_device(&devices);
        assert!(best.is_some());

        // Notification manager should skip all notifications (return Ok immediately)
        assert!(notification_manager.device_connected(&device).is_ok());
        assert!(notification_manager.device_disconnected(&device).is_ok());
        assert!(
            notification_manager
                .device_switched(&device, SwitchReason::Manual)
                .is_ok()
        );
    }

    #[test]
    fn test_empty_device_rules_affect_priority_selection() {
        let config_no_rules = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig::default(),
            output_devices: vec![], // No rules
            input_devices: vec![],
        };

        let priority_manager = DevicePriorityManager::new(&config_no_rules);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("Any Device")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Another Device")
                .output()
                .build(),
        ];

        // Should return None when no rules match
        let best = priority_manager.find_best_output_device(&devices);
        assert!(best.is_none());
    }

    #[test]
    fn test_match_type_consistency_across_components() {
        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: true,
                show_switching_actions: true,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("Air")
                    .weight(200)
                    .starts_with_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("Pro")
                    .weight(150)
                    .ends_with_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("Exact Match Device")
                    .weight(100)
                    .exact_match()
                    .build(),
            ],
            input_devices: vec![],
        };

        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .build(), // Matches both "Air" (starts_with) and "Pro" (ends_with)
            AudioDeviceBuilder::new()
                .name("Exact Match Device")
                .output()
                .build(), // Matches exact
            AudioDeviceBuilder::new()
                .name("No Match Device")
                .output()
                .build(), // Matches nothing
        ];

        // Should pick AirPods Pro because "Air" rule has highest weight (200)
        let best = priority_manager.find_best_output_device(&devices);
        assert!(best.is_some());
        assert_eq!(best.as_ref().unwrap().name, "AirPods Pro");

        // Notification should work with the selected device
        assert!(
            notification_manager
                .device_switched(&best.unwrap(), SwitchReason::HigherPriority)
                .is_ok()
        );
    }
}

/// Test realistic user scenarios with multiple components
#[cfg(test)]
mod realistic_scenarios {
    use super::*;

    #[test]
    fn test_home_office_setup_scenario() {
        // Realistic home office setup
        let config = Config {
            general: GeneralConfig {
                check_interval_ms: 1000,
                log_level: "info".to_string(),
                daemon_mode: true,
            },
            notifications: NotificationConfig {
                show_device_availability: true,
                show_switching_actions: true,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("AirPods")
                    .weight(300)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("Studio Display")
                    .weight(200)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("MacBook Pro Speakers")
                    .weight(10)
                    .exact_match()
                    .build(),
            ],
            input_devices: vec![
                DeviceRuleBuilder::new()
                    .name("Blue Yeti")
                    .weight(500)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("AirPods")
                    .weight(200)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("MacBook Pro Microphone")
                    .weight(10)
                    .exact_match()
                    .build(),
            ],
        };

        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        // Scenario 1: Working from home - Studio Display and Blue Yeti connected
        let work_devices = vec![
            AudioDeviceBuilder::new()
                .name("Studio Display Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Blue Yeti Microphone")
                .input()
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Microphone")
                .input()
                .build(),
        ];

        let work_output = priority_manager.find_best_output_device(&work_devices);
        let work_input = priority_manager.find_best_input_device(&work_devices);

        assert_eq!(
            work_output.as_ref().unwrap().name,
            "Studio Display Speakers"
        );
        assert_eq!(work_input.as_ref().unwrap().name, "Blue Yeti Microphone");

        // Scenario 2: AirPods connect (highest priority for output)
        let airpods_devices = vec![
            AudioDeviceBuilder::new()
                .name("Studio Display Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Blue Yeti Microphone")
                .input()
                .build(),
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("AirPods Pro Microphone")
                .input()
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Microphone")
                .input()
                .build(),
        ];

        let airpods_output = priority_manager.find_best_output_device(&airpods_devices);
        let airpods_input = priority_manager.find_best_input_device(&airpods_devices);

        assert_eq!(airpods_output.as_ref().unwrap().name, "AirPods Pro");
        assert_eq!(airpods_input.as_ref().unwrap().name, "Blue Yeti Microphone"); // Blue Yeti still wins for input

        // Test notifications for the switches
        assert!(
            notification_manager
                .device_connected(&airpods_output.clone().unwrap())
                .is_ok()
        );
        assert!(
            notification_manager
                .device_switched(&airpods_output.unwrap(), SwitchReason::HigherPriority)
                .is_ok()
        );

        // Scenario 3: Blue Yeti disconnects, fallback to AirPods mic
        let no_yeti_devices = vec![
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("AirPods Pro Microphone")
                .input()
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Microphone")
                .input()
                .build(),
        ];

        let fallback_input = priority_manager.find_best_input_device(&no_yeti_devices);
        assert_eq!(
            fallback_input.as_ref().unwrap().name,
            "AirPods Pro Microphone"
        );

        assert!(
            notification_manager
                .device_switched(&fallback_input.unwrap(), SwitchReason::PreviousUnavailable)
                .is_ok()
        );
    }

    #[test]
    fn test_gaming_setup_scenario() {
        let gaming_config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: false, // Gaming setup - no connection notifications
                show_switching_actions: true,    // But want switching notifications
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("Gaming Headset")
                    .weight(400)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("Gaming Speakers")
                    .weight(300)
                    .contains_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("Monitor")
                    .weight(200)
                    .contains_match()
                    .build(),
            ],
            input_devices: vec![
                DeviceRuleBuilder::new()
                    .name("Gaming Headset")
                    .weight(400)
                    .contains_match()
                    .build(),
            ],
        };

        let priority_manager = DevicePriorityManager::new(&gaming_config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&gaming_config, sender);

        let gaming_devices = vec![
            AudioDeviceBuilder::new()
                .name("SteelSeries Gaming Headset")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("SteelSeries Gaming Headset")
                .input()
                .build(),
            AudioDeviceBuilder::new()
                .name("Logitech Gaming Speakers")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Dell Gaming Monitor")
                .output()
                .build(),
        ];

        let best_output = priority_manager.find_best_output_device(&gaming_devices);
        let best_input = priority_manager.find_best_input_device(&gaming_devices);

        assert_eq!(
            best_output.as_ref().unwrap().name,
            "SteelSeries Gaming Headset"
        );
        assert_eq!(
            best_input.as_ref().unwrap().name,
            "SteelSeries Gaming Headset"
        );

        // Connection notifications should be skipped (config disabled)
        assert!(
            notification_manager
                .device_connected(&best_output.clone().unwrap())
                .is_ok()
        );

        // Switching notifications should work
        assert!(
            notification_manager
                .device_switched(&best_output.unwrap(), SwitchReason::Manual)
                .is_ok()
        );
    }
}

/// Test error handling and edge cases across components
#[cfg(test)]
mod cross_component_edge_cases {
    use super::*;

    #[test]
    fn test_unicode_device_names_across_components() {
        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: true,
                show_switching_actions: true,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("🎵")
                    .weight(100)
                    .contains_match()
                    .build(),
            ],
            input_devices: vec![],
        };

        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        let unicode_device = AudioDeviceBuilder::new()
            .name("🎵 音频设备 🎵")
            .output()
            .build();

        let devices = vec![unicode_device.clone()];

        // Priority manager should handle Unicode
        let best = priority_manager.find_best_output_device(&devices);
        assert!(best.is_some());
        assert_eq!(best.as_ref().unwrap().name, "🎵 音频设备 🎵");

        // Notification manager should handle Unicode
        assert!(
            notification_manager
                .device_connected(&unicode_device)
                .is_ok()
        );
        assert!(
            notification_manager
                .device_switched(&unicode_device, SwitchReason::HigherPriority)
                .is_ok()
        );
    }

    #[test]
    fn test_component_behavior_with_disabled_rules() {
        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: true,
                show_switching_actions: true,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("Enabled Device")
                    .weight(200)
                    .exact_match()
                    .build(),
                DeviceRuleBuilder::new()
                    .name("Disabled Device")
                    .weight(300) // Higher weight but disabled
                    .exact_match()
                    .disabled()
                    .build(),
            ],
            input_devices: vec![],
        };

        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        let devices = vec![
            AudioDeviceBuilder::new()
                .name("Enabled Device")
                .output()
                .build(),
            AudioDeviceBuilder::new()
                .name("Disabled Device")
                .output()
                .build(),
        ];

        // Should pick enabled device despite lower weight
        let best = priority_manager.find_best_output_device(&devices);
        assert!(best.is_some());
        assert_eq!(best.as_ref().unwrap().name, "Enabled Device");

        // Notification should work normally
        assert!(
            notification_manager
                .device_switched(&best.unwrap(), SwitchReason::HigherPriority)
                .is_ok()
        );
    }

    #[test]
    fn test_performance_with_many_devices_and_rules() {
        // Create config with many rules
        let mut output_rules = Vec::new();
        for i in 0..50 {
            output_rules.push(
                DeviceRuleBuilder::new()
                    .name(&format!("Device Rule {}", i))
                    .weight(i as u32)
                    .contains_match()
                    .build(),
            );
        }

        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig::default(),
            output_devices: output_rules,
            input_devices: vec![],
        };

        let priority_manager = DevicePriorityManager::new(&config);
        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        // Create many devices
        let mut devices = Vec::new();
        for i in 0..100 {
            devices.push(
                AudioDeviceBuilder::new()
                    .name(&format!("Test Device Rule {}", i))
                    .output()
                    .build(),
            );
        }

        // Performance test
        let start = std::time::Instant::now();
        let best = priority_manager.find_best_output_device(&devices);
        let selection_duration = start.elapsed();

        // Should complete quickly (< 1ms as per success criteria)
        assert!(
            selection_duration.as_millis() < 1,
            "Device selection took too long: {:?}",
            selection_duration
        );

        // Should find the highest weight device
        assert!(best.is_some());
        assert_eq!(best.as_ref().unwrap().name, "Test Device Rule 49"); // Rule 49 has weight 49 (highest)

        // Notification should work quickly too
        let start = std::time::Instant::now();
        assert!(
            notification_manager
                .device_switched(&best.unwrap(), SwitchReason::HigherPriority)
                .is_ok()
        );
        let notification_duration = start.elapsed();

        assert!(
            notification_duration.as_millis() < 1,
            "Notification took too long: {:?}",
            notification_duration
        );
    }

    #[test]
    fn test_switching_reasons_consistency() {
        let config = Config {
            general: GeneralConfig::default(),
            notifications: NotificationConfig {
                show_device_availability: false,
                show_switching_actions: true,
                show_device_changes: None,
            },
            output_devices: vec![
                DeviceRuleBuilder::new()
                    .name("Test")
                    .weight(100)
                    .contains_match()
                    .build(),
            ],
            input_devices: vec![],
        };

        let sender = TestNotificationSender::new();
        let notification_manager = NotificationManager::with_sender(&config, sender);

        let device = AudioDeviceBuilder::new()
            .name("Test Device")
            .output()
            .build();

        // All switch reasons should work consistently
        let switch_reasons = vec![
            SwitchReason::HigherPriority,
            SwitchReason::PreviousUnavailable,
            SwitchReason::Manual,
        ];

        for reason in switch_reasons {
            assert!(
                notification_manager
                    .device_switched(&device, reason)
                    .is_ok()
            );
        }
    }
}
