//! Test utility builders for creating test instances of audio devices and rules
//! 
//! This module provides builder patterns for easily creating test data.
//! Individual methods may not be used by all tests, so dead code warnings are suppressed.

#![allow(dead_code)]

use audio_device_monitor::audio::{AudioDevice, DeviceType};
use audio_device_monitor::config::{DeviceRule, MatchType};

/// Builder for creating test AudioDevice instances
pub struct AudioDeviceBuilder {
    id: String,
    name: String,
    device_type: DeviceType,
    is_default: bool,
    is_available: bool,
    uid: Option<String>,
}

impl AudioDeviceBuilder {
    pub fn new() -> Self {
        Self {
            id: "test_device_1".to_string(),
            name: "Test Device".to_string(),
            device_type: DeviceType::Output,
            is_default: false,
            is_available: true,
            uid: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn input(mut self) -> Self {
        self.device_type = DeviceType::Input;
        self
    }

    pub fn output(mut self) -> Self {
        self.device_type = DeviceType::Output;
        self
    }

    pub fn default_device(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn unavailable(mut self) -> Self {
        self.is_available = false;
        self
    }

    pub fn with_uid(mut self, uid: &str) -> Self {
        self.uid = Some(uid.to_string());
        self
    }

    pub fn build(self) -> AudioDevice {
        let mut device = AudioDevice::new(self.id, self.name, self.device_type);
        if let Some(uid) = self.uid {
            device = device.with_uid(uid);
        }
        device = device.set_default(self.is_default);
        device = device.set_available(self.is_available);
        device
    }
}

impl Default for AudioDeviceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating test DeviceRule instances
pub struct DeviceRuleBuilder {
    name: String,
    weight: u32,
    match_type: MatchType,
    enabled: bool,
}

impl DeviceRuleBuilder {
    pub fn new() -> Self {
        Self {
            name: "Test Rule".to_string(),
            weight: 100,
            match_type: MatchType::Exact,
            enabled: true,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    pub fn exact_match(mut self) -> Self {
        self.match_type = MatchType::Exact;
        self
    }

    pub fn contains_match(mut self) -> Self {
        self.match_type = MatchType::Contains;
        self
    }

    pub fn starts_with_match(mut self) -> Self {
        self.match_type = MatchType::StartsWith;
        self
    }

    pub fn ends_with_match(mut self) -> Self {
        self.match_type = MatchType::EndsWith;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn build(self) -> DeviceRule {
        DeviceRule {
            name: self.name,
            weight: self.weight,
            match_type: self.match_type,
            enabled: self.enabled,
        }
    }
}

impl Default for DeviceRuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating common test scenarios
pub mod scenarios {
    use super::*;

    /// Create a typical set of output devices for testing
    pub fn typical_output_devices() -> Vec<AudioDevice> {
        vec![
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .output()
                .id("airpods_pro")
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Speakers")
                .output()
                .id("mbp_speakers")
                .default_device()
                .build(),
            AudioDeviceBuilder::new()
                .name("Audioengine 2+")
                .output()
                .id("audioengine")
                .build(),
        ]
    }

    /// Create a typical set of input devices for testing
    pub fn typical_input_devices() -> Vec<AudioDevice> {
        vec![
            AudioDeviceBuilder::new()
                .name("AirPods Pro")
                .input()
                .id("airpods_pro_input")
                .build(),
            AudioDeviceBuilder::new()
                .name("MacBook Pro Microphone")
                .input()
                .id("mbp_mic")
                .default_device()
                .build(),
            AudioDeviceBuilder::new()
                .name("Shure MV7")
                .input()
                .id("shure_mv7")
                .build(),
        ]
    }

    /// Create a typical set of priority rules for testing
    pub fn typical_priority_rules() -> Vec<DeviceRule> {
        vec![
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
            DeviceRuleBuilder::new()
                .name("MacBook Pro Speakers")
                .weight(10)
                .exact_match()
                .build(),
        ]
    }

    /// Create devices with special characters for edge case testing
    pub fn special_character_devices() -> Vec<AudioDevice> {
        vec![
            AudioDeviceBuilder::new()
                .name("🎵 Music Device 🎵")
                .output()
                .id("emoji_device")
                .build(),
            AudioDeviceBuilder::new()
                .name("Device with spaces")
                .output()
                .id("spaces_device")
                .build(),
            AudioDeviceBuilder::new()
                .name("Device-with-dashes")
                .output()
                .id("dashes_device")
                .build(),
            AudioDeviceBuilder::new()
                .name("")
                .output()
                .id("empty_name_device")
                .build(),
        ]
    }
}
