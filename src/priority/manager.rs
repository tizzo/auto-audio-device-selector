use tracing::{debug, info};

use crate::audio::AudioDevice;
use crate::config::{Config, DeviceRule};

pub struct DevicePriorityManager {
    output_priorities: Vec<DeviceRule>,
    input_priorities: Vec<DeviceRule>,
    current_output: Option<String>,
    current_input: Option<String>,
}

impl DevicePriorityManager {
    pub fn new(config: &Config) -> Self {
        info!("Creating device priority manager");

        Self {
            output_priorities: config.output_devices.clone(),
            input_priorities: config.input_devices.clone(),
            current_output: None,
            current_input: None,
        }
    }

    pub fn find_best_output_device(
        &self,
        available_devices: &[AudioDevice],
    ) -> Option<AudioDevice> {
        self.find_best_device(available_devices, &self.output_priorities, "output")
    }

    pub fn find_best_input_device(&self, available_devices: &[AudioDevice]) -> Option<AudioDevice> {
        self.find_best_device(available_devices, &self.input_priorities, "input")
    }

    fn find_best_device(
        &self,
        available_devices: &[AudioDevice],
        priorities: &[DeviceRule],
        device_type: &str,
    ) -> Option<AudioDevice> {
        let mut best_device: Option<AudioDevice> = None;
        let mut best_weight = 0;

        debug!(
            "Evaluating {} devices for {} type:",
            available_devices.len(),
            device_type
        );
        for device in available_devices {
            debug!("  Checking device: '{}'", device.name);
            for rule in priorities {
                let matches = rule.matches(&device.name);
                debug!(
                    "    Rule '{}' (type: {:?}, weight: {}) -> matches: {}",
                    rule.name, rule.match_type, rule.weight, matches
                );
                if matches && rule.weight > best_weight {
                    best_device = Some(device.clone());
                    best_weight = rule.weight;
                    debug!(
                        "Found {} device match: {} (weight: {})",
                        device_type, device.name, rule.weight
                    );
                }
            }
        }

        if let Some(ref device) = best_device {
            info!(
                "Best {} device: {} (weight: {})",
                device_type, device.name, best_weight
            );
        } else {
            debug!("No matching {} device found", device_type);
        }

        best_device
    }

    pub fn should_switch_output(&self, new_device: &AudioDevice) -> bool {
        match &self.current_output {
            Some(current) => current != &new_device.name,
            None => true,
        }
    }

    pub fn should_switch_input(&self, new_device: &AudioDevice) -> bool {
        match &self.current_input {
            Some(current) => current != &new_device.name,
            None => true,
        }
    }

    pub fn update_current_output(&mut self, device_name: String) {
        self.current_output = Some(device_name);
    }

    pub fn update_current_input(&mut self, device_name: String) {
        self.current_input = Some(device_name);
    }
}
