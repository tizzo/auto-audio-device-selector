use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Input,
    Output,
    InputOutput,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub is_default: bool,
    pub is_available: bool,
    pub uid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub uid: String,
    pub device_type: DeviceType,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub is_default: bool,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Input => write!(f, "Input"),
            DeviceType::Output => write!(f, "Output"),
            DeviceType::InputOutput => write!(f, "Input/Output"),
        }
    }
}

impl fmt::Display for AudioDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {} [{}]",
            self.name,
            self.device_type,
            if self.is_default {
                "Default"
            } else {
                "Available"
            },
            if self.is_available {
                "Online"
            } else {
                "Offline"
            }
        )
    }
}

impl AudioDevice {
    pub fn new(id: String, name: String, device_type: DeviceType) -> Self {
        Self {
            id,
            name,
            device_type,
            is_default: false,
            is_available: true,
            uid: None,
        }
    }

    pub fn with_uid(mut self, uid: String) -> Self {
        self.uid = Some(uid);
        self
    }

    pub fn set_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    pub fn set_available(mut self, is_available: bool) -> Self {
        self.is_available = is_available;
        self
    }
}
