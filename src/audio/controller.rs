use anyhow::Result;
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use coreaudio_sys::*;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};
use std::os::raw::c_void;
use std::ptr;
use tracing::{debug, error, info, warn};

use super::device::{AudioDevice, DeviceInfo, DeviceType};

pub struct DeviceController {
    host: Host,
}

impl DeviceController {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        info!(
            "Initialized audio device controller with host: {}",
            host.id().name()
        );

        Ok(Self { host })
    }

    pub fn enumerate_devices(&self) -> Result<Vec<AudioDevice>> {
        let mut devices = Vec::new();

        // Get input devices
        match self.host.input_devices() {
            Ok(input_devices) => {
                for device in input_devices {
                    if let Ok(audio_device) = self.device_to_audio_device(device, DeviceType::Input)
                    {
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
                    if let Ok(audio_device) =
                        self.device_to_audio_device(device, DeviceType::Output)
                    {
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

    /// Set the default output device by name
    pub fn set_default_output_device(&self, device_name: &str) -> Result<()> {
        info!("Setting default output device to: {}", device_name);

        // Find the CoreAudio device ID by name
        if let Some(device_id) = self.find_coreaudio_device_by_name(device_name, false)? {
            self.set_default_output_device_by_id(device_id)?;
        } else {
            return Err(anyhow::anyhow!("Output device '{}' not found", device_name));
        }

        Ok(())
    }

    /// Set the default input device by name  
    pub fn set_default_input_device(&self, device_name: &str) -> Result<()> {
        info!("Setting default input device to: {}", device_name);

        // Find the CoreAudio device ID by name
        if let Some(device_id) = self.find_coreaudio_device_by_name(device_name, true)? {
            self.set_default_input_device_by_id(device_id)?;
        } else {
            return Err(anyhow::anyhow!("Input device '{}' not found", device_name));
        }

        Ok(())
    }

    /// Set default output device by CoreAudio device ID
    fn set_default_output_device_by_id(&self, device_id: AudioDeviceID) -> Result<()> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultOutputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        unsafe {
            let result = AudioObjectSetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                std::mem::size_of::<AudioDeviceID>() as u32,
                &device_id as *const _ as *const c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                error!("Failed to set default output device: {}", result);
                return Err(anyhow::anyhow!("Failed to set default output device"));
            }
        }

        info!("Successfully set default output device ID: {}", device_id);
        Ok(())
    }

    /// Set default input device by CoreAudio device ID
    fn set_default_input_device_by_id(&self, device_id: AudioDeviceID) -> Result<()> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultInputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        unsafe {
            let result = AudioObjectSetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                std::mem::size_of::<AudioDeviceID>() as u32,
                &device_id as *const _ as *const c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                error!("Failed to set default input device: {}", result);
                return Err(anyhow::anyhow!("Failed to set default input device"));
            }
        }

        info!("Successfully set default input device ID: {}", device_id);
        Ok(())
    }

    /// Find CoreAudio device ID by name
    fn find_coreaudio_device_by_name(
        &self,
        device_name: &str,
        is_input: bool,
    ) -> Result<Option<AudioDeviceID>> {
        debug!(
            "Looking for {} device: {}",
            if is_input { "input" } else { "output" },
            device_name
        );

        unsafe {
            // Get list of all audio devices
            let property_address = AudioObjectPropertyAddress {
                mSelector: kAudioHardwarePropertyDevices,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain,
            };

            let mut property_size: u32 = 0;
            let result = AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
            );

            if result != kAudioHardwareNoError as i32 {
                return Err(anyhow::anyhow!("Failed to get device list size"));
            }

            let device_count = property_size / std::mem::size_of::<AudioDeviceID>() as u32;
            let mut devices = vec![0u32; device_count as usize];

            let result = AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                devices.as_mut_ptr() as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                return Err(anyhow::anyhow!("Failed to get device list"));
            }

            // Check each device
            for &device_id in &devices {
                if let Ok(name) = self.get_coreaudio_device_name(device_id) {
                    if name == device_name {
                        // Verify device supports the required direction
                        if self.device_supports_direction(device_id, is_input)? {
                            debug!("Found matching device: {} (ID: {})", name, device_id);
                            return Ok(Some(device_id));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get the name of a CoreAudio device
    fn get_coreaudio_device_name(&self, device_id: AudioDeviceID) -> Result<String> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceNameCFString,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        unsafe {
            let mut property_size = std::mem::size_of::<CFStringRef>() as u32;
            let mut cf_string: CFStringRef = ptr::null();

            let result = AudioObjectGetPropertyData(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut cf_string as *mut _ as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                return Err(anyhow::anyhow!("Failed to get device name"));
            }

            if cf_string.is_null() {
                return Err(anyhow::anyhow!("Device name is null"));
            }

            let cf_string = CFString::wrap_under_get_rule(cf_string);
            Ok(cf_string.to_string())
        }
    }

    /// Check if device supports input or output
    fn device_supports_direction(&self, device_id: AudioDeviceID, is_input: bool) -> Result<bool> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: if is_input {
                kAudioDevicePropertyScopeInput
            } else {
                kAudioDevicePropertyScopeOutput
            },
            mElement: kAudioObjectPropertyElementMain,
        };

        unsafe {
            let mut property_size: u32 = 0;
            let result = AudioObjectGetPropertyDataSize(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
            );

            if result != kAudioHardwareNoError as i32 {
                return Ok(false);
            }

            // If property_size > 0, device supports this direction
            Ok(property_size > 0)
        }
    }

    // Convert cpal::Device to our AudioDevice
    fn device_to_audio_device(
        &self,
        device: Device,
        device_type: DeviceType,
    ) -> Result<AudioDevice> {
        let name = device
            .name()
            .unwrap_or_else(|_| "Unknown Device".to_string());

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
