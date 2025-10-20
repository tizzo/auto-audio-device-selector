use anyhow::Result;
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use coreaudio_sys::*;
// Removed cpal imports
use std::os::raw::c_void;
use std::ptr;
use tracing::{debug, error};

use super::device::{AudioDevice, DeviceInfo, DeviceType};

pub struct DeviceController {
    // No longer need cpal host
}

impl DeviceController {
    pub fn new() -> Result<Self> {
        debug!("Initialized audio device controller with CoreAudio");
        Ok(Self {})
    }

    pub fn enumerate_devices(&self) -> Result<Vec<AudioDevice>> {
        let mut devices = Vec::new();

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
            let mut device_ids = vec![0u32; device_count as usize];

            let result = AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                device_ids.as_mut_ptr() as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                return Err(anyhow::anyhow!("Failed to get device list"));
            }

            // Process each device
            for &device_id in &device_ids {
                if let Ok(name) = self.get_coreaudio_device_name(device_id) {
                    // Check if device supports input
                    if self.device_supports_direction(device_id, true)? {
                        let mut audio_device = AudioDevice::new(
                            device_id.to_string(),
                            name.clone(),
                            DeviceType::Input,
                        );

                        // Get device UID for more reliable identification
                        if let Ok(uid) = self.get_coreaudio_device_uid(device_id) {
                            audio_device = audio_device.with_uid(uid);
                        }

                        devices.push(audio_device);
                    }

                    // Check if device supports output
                    if self.device_supports_direction(device_id, false)? {
                        let mut audio_device = AudioDevice::new(
                            device_id.to_string(),
                            name.clone(),
                            DeviceType::Output,
                        );

                        // Get device UID for more reliable identification
                        if let Ok(uid) = self.get_coreaudio_device_uid(device_id) {
                            audio_device = audio_device.with_uid(uid);
                        }

                        devices.push(audio_device);
                    }
                }
            }
        }

        debug!("Enumerated {} audio devices", devices.len());
        Ok(devices)
    }

    pub fn get_default_input_device(&self) -> Result<Option<AudioDevice>> {
        unsafe {
            let property_address = AudioObjectPropertyAddress {
                mSelector: kAudioHardwarePropertyDefaultInputDevice,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain,
            };

            let mut device_id: AudioDeviceID = 0;
            let mut property_size = std::mem::size_of::<AudioDeviceID>() as u32;

            let result = AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut device_id as *mut _ as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 || device_id == kAudioDeviceUnknown {
                debug!("No default input device found");
                return Ok(None);
            }

            if let Ok(name) = self.get_coreaudio_device_name(device_id) {
                let mut audio_device =
                    AudioDevice::new(device_id.to_string(), name, DeviceType::Input);

                if let Ok(uid) = self.get_coreaudio_device_uid(device_id) {
                    audio_device = audio_device.with_uid(uid);
                }

                audio_device = audio_device.set_default(true);
                Ok(Some(audio_device))
            } else {
                debug!("Could not get name for default input device");
                Ok(None)
            }
        }
    }

    pub fn get_default_output_device(&self) -> Result<Option<AudioDevice>> {
        unsafe {
            let property_address = AudioObjectPropertyAddress {
                mSelector: kAudioHardwarePropertyDefaultOutputDevice,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain,
            };

            let mut device_id: AudioDeviceID = 0;
            let mut property_size = std::mem::size_of::<AudioDeviceID>() as u32;

            let result = AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut device_id as *mut _ as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 || device_id == kAudioDeviceUnknown {
                debug!("No default output device found");
                return Ok(None);
            }

            if let Ok(name) = self.get_coreaudio_device_name(device_id) {
                let mut audio_device =
                    AudioDevice::new(device_id.to_string(), name, DeviceType::Output);

                if let Ok(uid) = self.get_coreaudio_device_uid(device_id) {
                    audio_device = audio_device.with_uid(uid);
                }

                audio_device = audio_device.set_default(true);
                Ok(Some(audio_device))
            } else {
                debug!("Could not get name for default output device");
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
        debug!("Setting default output device to: {}", device_name);

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
        debug!("Setting default input device to: {}", device_name);

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

        debug!("Successfully set default output device ID: {}", device_id);
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

        debug!("Successfully set default input device ID: {}", device_id);
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

    /// Get the UID of a CoreAudio device
    fn get_coreaudio_device_uid(&self, device_id: AudioDeviceID) -> Result<String> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceUID,
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
                return Err(anyhow::anyhow!("Failed to get device UID"));
            }

            if cf_string.is_null() {
                return Err(anyhow::anyhow!("Device UID is null"));
            }

            let cf_string = CFString::wrap_under_get_rule(cf_string);
            Ok(cf_string.to_string())
        }
    }

    /// Check if device supports input or output by checking actual channel count
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

            if result != kAudioHardwareNoError as i32 || property_size == 0 {
                return Ok(false);
            }

            // Get the stream configuration to check actual channel counts
            let mut buffer = vec![0u8; property_size as usize];
            let result = AudioObjectGetPropertyData(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                buffer.as_mut_ptr() as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                return Ok(false);
            }

            // Parse AudioBufferList to check for actual channels
            let buffer_list = buffer.as_ptr() as *const AudioBufferList;
            let buffer_count = (*buffer_list).mNumberBuffers;

            if buffer_count == 0 {
                return Ok(false);
            }

            // Check if any buffer has channels
            for i in 0..buffer_count {
                let buffer = &(*buffer_list).mBuffers[i as usize];
                if buffer.mNumberChannels > 0 {
                    return Ok(true);
                }
            }

            Ok(false)
        }
    }

    // Removed old cpal-dependent device conversion method
}

impl Default for DeviceController {
    fn default() -> Self {
        Self::new().expect("Failed to create default device controller")
    }
}
