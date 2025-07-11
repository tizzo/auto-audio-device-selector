use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};
use tracing::{debug, info, warn};

use super::device::{AudioDevice, DeviceInfo, DeviceType};

pub struct DeviceController {
    host: Host,
}

impl DeviceController {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        info!("Initialized audio device controller with host: {}", host.id().name());
        
        Ok(Self { host })
    }
    
    pub fn enumerate_devices(&self) -> Result<Vec<AudioDevice>> {
        let mut devices = Vec::new();
        
        // Get input devices
        match self.host.input_devices() {
            Ok(input_devices) => {
                for device in input_devices {
                    if let Ok(audio_device) = self.device_to_audio_device(device, DeviceType::Input) {
                        devices.push(audio_device);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to enumerate input devices: {}", e);
            }
        }
        
        // Get output devices
        match self.host.output_devices() {
            Ok(output_devices) => {
                for device in output_devices {
                    if let Ok(audio_device) = self.device_to_audio_device(device, DeviceType::Output) {
                        devices.push(audio_device);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to enumerate output devices: {}", e);
            }
        }
        
        info!("Enumerated {} audio devices", devices.len());
        Ok(devices)
    }
    
    pub fn get_default_input_device(&self) -> Result<Option<AudioDevice>> {
        match self.host.default_input_device() {
            Some(device) => {
                let mut audio_device = self.device_to_audio_device(device, DeviceType::Input)?;
                audio_device = audio_device.set_default(true);
                Ok(Some(audio_device))
            }
            None => {
                debug!("No default input device found");
                Ok(None)
            }
        }
    }
    
    pub fn get_default_output_device(&self) -> Result<Option<AudioDevice>> {
        match self.host.default_output_device() {
            Some(device) => {
                let mut audio_device = self.device_to_audio_device(device, DeviceType::Output)?;
                audio_device = audio_device.set_default(true);
                Ok(Some(audio_device))
            }
            None => {
                debug!("No default output device found");
                Ok(None)
            }
        }
    }
    
    pub fn get_device_info(&self, device: &AudioDevice) -> Result<DeviceInfo> {
        // This will be expanded with more detailed device information
        Ok(DeviceInfo {
            name: device.name.clone(),
            uid: device.uid.clone().unwrap_or_else(|| device.id.clone()),
            device_type: device.device_type.clone(),
            sample_rate: None, // Will be filled with actual device capabilities
            channels: None,    // Will be filled with actual device capabilities
            is_default: device.is_default,
        })
    }
    
    // Convert cpal::Device to our AudioDevice
    fn device_to_audio_device(&self, device: Device, device_type: DeviceType) -> Result<AudioDevice> {
        let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
        
        // For now, use the name as the ID. Later we'll use proper device UIDs
        let id = name.clone();
        
        Ok(AudioDevice::new(id, name, device_type))
    }
}

impl Default for DeviceController {
    fn default() -> Self {
        Self::new().expect("Failed to create default device controller")
    }
}