use audio_device_monitor::config::{DeviceRule, MatchType};

mod test_utils;
use test_utils::builders::DeviceRuleBuilder;

/// Test exact matching behavior
#[cfg(test)]
mod exact_matching {
    use super::*;

    #[test]
    fn test_exact_match_success() {
        let rule = DeviceRuleBuilder::new()
            .name("AirPods Pro")
            .exact_match()
            .build();

        assert!(rule.matches("AirPods Pro"));
    }

    #[test]
    fn test_exact_match_failure() {
        let rule = DeviceRuleBuilder::new()
            .name("AirPods Pro")
            .exact_match()
            .build();

        assert!(!rule.matches("AirPods"));
        assert!(!rule.matches("AirPods Pro Max"));
        assert!(!rule.matches("airpods pro")); // Case sensitive
        assert!(!rule.matches(" AirPods Pro ")); // Whitespace sensitive
    }

    #[test]
    fn test_exact_match_empty_strings() {
        let rule = DeviceRuleBuilder::new().name("").exact_match().build();

        assert!(rule.matches(""));
        assert!(!rule.matches("anything"));
    }

    #[test]
    fn test_exact_match_unicode() {
        let rule = DeviceRuleBuilder::new()
            .name("🎵 Music Device 🎵")
            .exact_match()
            .build();

        assert!(rule.matches("🎵 Music Device 🎵"));
        assert!(!rule.matches("🎵 Music Device"));
        assert!(!rule.matches("Music Device 🎵"));
    }
}

/// Test contains matching behavior
#[cfg(test)]
mod contains_matching {
    use super::*;

    #[test]
    fn test_contains_match_success() {
        let rule = DeviceRuleBuilder::new()
            .name("AirPods")
            .contains_match()
            .build();

        assert!(rule.matches("AirPods"));
        assert!(rule.matches("AirPods Pro"));
        assert!(rule.matches("My AirPods"));

        // Test the actual real-world case - this device contains "AirPod" not "AirPods"
        let airpod_rule = DeviceRuleBuilder::new()
            .name("AirPod") // Without the 's' to match the actual device name
            .contains_match()
            .build();
        assert!(airpod_rule.matches("🌪️☠️ AirPod's Revenge ☠️🌪️"));
    }

    #[test]
    fn test_contains_match_failure() {
        let rule = DeviceRuleBuilder::new()
            .name("AirPods")
            .contains_match()
            .build();

        assert!(!rule.matches("AirPod")); // Partial match
        assert!(!rule.matches("airpods")); // Case sensitive
        assert!(!rule.matches("Beats"));
    }

    #[test]
    fn test_contains_match_empty_rule() {
        let rule = DeviceRuleBuilder::new().name("").contains_match().build();

        // Empty string is contained in everything
        assert!(rule.matches(""));
        assert!(rule.matches("anything"));
        assert!(rule.matches("🎵"));
    }

    #[test]
    fn test_contains_match_special_characters() {
        let rule = DeviceRuleBuilder::new()
            .name("MV7")
            .contains_match()
            .build();

        assert!(rule.matches("Shure MV7"));
        assert!(rule.matches("MV7-USB"));
        assert!(rule.matches("My-MV7-Device"));
    }
}

/// Test starts_with matching behavior
#[cfg(test)]
mod starts_with_matching {
    use super::*;

    #[test]
    fn test_starts_with_match_success() {
        let rule = DeviceRuleBuilder::new()
            .name("MacBook")
            .starts_with_match()
            .build();

        assert!(rule.matches("MacBook"));
        assert!(rule.matches("MacBook Pro"));
        assert!(rule.matches("MacBook Pro Speakers"));
        assert!(rule.matches("MacBook Air Microphone"));
    }

    #[test]
    fn test_starts_with_match_failure() {
        let rule = DeviceRuleBuilder::new()
            .name("MacBook")
            .starts_with_match()
            .build();

        assert!(!rule.matches("My MacBook"));
        assert!(!rule.matches("macbook")); // Case sensitive
        assert!(!rule.matches("Book"));
        assert!(!rule.matches(""));
    }

    #[test]
    fn test_starts_with_empty_rule() {
        let rule = DeviceRuleBuilder::new()
            .name("")
            .starts_with_match()
            .build();

        // Empty string starts every string
        assert!(rule.matches(""));
        assert!(rule.matches("anything"));
    }
}

/// Test ends_with matching behavior
#[cfg(test)]
mod ends_with_matching {
    use super::*;

    #[test]
    fn test_ends_with_match_success() {
        let rule = DeviceRuleBuilder::new()
            .name("Speakers")
            .ends_with_match()
            .build();

        assert!(rule.matches("Speakers"));
        assert!(rule.matches("MacBook Pro Speakers"));
        assert!(rule.matches("External Speakers"));
        assert!(rule.matches("Built-in Speakers"));
    }

    #[test]
    fn test_ends_with_match_failure() {
        let rule = DeviceRuleBuilder::new()
            .name("Speakers")
            .ends_with_match()
            .build();

        assert!(!rule.matches("Speakers System"));
        assert!(!rule.matches("speakers")); // Case sensitive
        assert!(!rule.matches("Speaker"));
        assert!(!rule.matches(""));
    }

    #[test]
    fn test_ends_with_empty_rule() {
        let rule = DeviceRuleBuilder::new().name("").ends_with_match().build();

        // Empty string ends every string
        assert!(rule.matches(""));
        assert!(rule.matches("anything"));
    }
}

/// Test disabled rules
#[cfg(test)]
mod disabled_rules {
    use super::*;

    #[test]
    fn test_disabled_rule_never_matches() {
        let rule = DeviceRuleBuilder::new()
            .name("AirPods")
            .contains_match()
            .disabled()
            .build();

        // Should never match when disabled, regardless of match type
        assert!(!rule.matches("AirPods"));
        assert!(!rule.matches("AirPods Pro"));
        assert!(!rule.matches("My AirPods"));
    }

    #[test]
    fn test_disabled_rule_all_match_types() {
        let test_cases = vec![
            (MatchType::Exact, "Test Device"),
            (MatchType::Contains, "Test Device Contains"),
            (MatchType::StartsWith, "Test Device Starts"),
            (MatchType::EndsWith, "Device Test"),
        ];

        for (match_type, device_name) in test_cases {
            let rule = DeviceRule {
                name: "Test".to_string(),
                weight: 100,
                match_type: match_type.clone(),
                enabled: false,
            };

            assert!(
                !rule.matches(device_name),
                "Disabled rule with {:?} should not match '{}'",
                match_type,
                device_name
            );
        }
    }
}

/// Test edge cases and special scenarios
#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_whitespace_handling() {
        let rule = DeviceRuleBuilder::new()
            .name("Device")
            .exact_match()
            .build();

        assert!(!rule.matches(" Device"));
        assert!(!rule.matches("Device "));
        assert!(!rule.matches(" Device "));
        assert!(!rule.matches("De vice"));
    }

    #[test]
    fn test_case_sensitivity() {
        let rule = DeviceRuleBuilder::new()
            .name("AirPods")
            .contains_match()
            .build();

        // All match types should be case sensitive
        assert!(!rule.matches("airpods"));
        assert!(!rule.matches("AIRPODS"));
        assert!(!rule.matches("AirpODS"));
    }

    #[test]
    fn test_unicode_and_special_characters() {
        let test_cases = vec![
            ("🎵", "🎵 Music Device 🎵", true),
            ("é", "Café Audio", true),
            ("™", "Device™", true),
            (".", "Device.1", true),
            ("-", "Audio-Device", true),
            ("_", "Audio_Device", true),
            ("(", "Device (Built-in)", true),
        ];

        for (pattern, device_name, should_match) in test_cases {
            let rule = DeviceRuleBuilder::new()
                .name(pattern)
                .contains_match()
                .build();

            assert_eq!(
                rule.matches(device_name),
                should_match,
                "Pattern '{}' in '{}' should {} match",
                pattern,
                device_name,
                if should_match { "" } else { "not" }
            );
        }
    }

    #[test]
    fn test_very_long_strings() {
        let long_pattern = "a".repeat(1000);
        let long_device_name = format!("prefix_{}_{}", long_pattern, "suffix");

        let rule = DeviceRuleBuilder::new()
            .name(&long_pattern)
            .contains_match()
            .build();

        assert!(rule.matches(&long_device_name));
    }

    #[test]
    fn test_all_match_types_with_same_data() {
        let device_name = "MacBook Pro Speakers";

        let test_cases = vec![
            ("MacBook Pro Speakers", MatchType::Exact, true),
            ("MacBook", MatchType::StartsWith, true),
            ("Speakers", MatchType::EndsWith, true),
            ("Pro", MatchType::Contains, true),
            ("Wrong", MatchType::Exact, false),
            ("Speakers", MatchType::StartsWith, false),
            ("MacBook", MatchType::EndsWith, false),
            ("Wrong", MatchType::Contains, false),
        ];

        for (pattern, match_type, expected) in test_cases {
            let rule = DeviceRule {
                name: pattern.to_string(),
                weight: 100,
                match_type: match_type.clone(),
                enabled: true,
            };

            assert_eq!(
                rule.matches(device_name),
                expected,
                "Rule '{}' with {:?} should {} match '{}'",
                pattern,
                match_type,
                if expected { "" } else { "not" },
                device_name
            );
        }
    }
}

/// Property-based testing for additional coverage
#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn test_exact_match_reflexivity() {
        // Any string should exactly match itself
        let test_strings = vec![
            "simple",
            "with spaces",
            "with-dashes_and_underscores",
            "🎵emoji🎵",
            "",
            "VeryLongStringThatShouldStillMatchItself",
        ];

        for s in test_strings {
            let rule = DeviceRuleBuilder::new().name(s).exact_match().build();

            assert!(
                rule.matches(s),
                "String '{}' should match itself exactly",
                s
            );
        }
    }

    #[test]
    fn test_contains_match_includes_exact() {
        // If exact match works, contains should also work for the same string
        let test_strings = vec!["AirPods", "MacBook Pro", "🎵"];

        for s in test_strings {
            let exact_rule = DeviceRuleBuilder::new().name(s).exact_match().build();

            let contains_rule = DeviceRuleBuilder::new().name(s).contains_match().build();

            if exact_rule.matches(s) {
                assert!(
                    contains_rule.matches(s),
                    "Contains rule should match '{}' if exact rule does",
                    s
                );
            }
        }
    }

    #[test]
    fn test_empty_pattern_behavior() {
        let empty_rule_exact = DeviceRuleBuilder::new().name("").exact_match().build();

        let empty_rule_contains = DeviceRuleBuilder::new().name("").contains_match().build();

        let empty_rule_starts = DeviceRuleBuilder::new()
            .name("")
            .starts_with_match()
            .build();

        let empty_rule_ends = DeviceRuleBuilder::new().name("").ends_with_match().build();

        let test_strings = vec!["", "something", "anything"];

        for s in test_strings {
            // Empty exact should only match empty string
            if s.is_empty() {
                assert!(empty_rule_exact.matches(s));
            } else {
                assert!(!empty_rule_exact.matches(s));
            }

            // Empty contains/starts_with/ends_with should match everything
            assert!(empty_rule_contains.matches(s));
            assert!(empty_rule_starts.matches(s));
            assert!(empty_rule_ends.matches(s));
        }
    }
}
