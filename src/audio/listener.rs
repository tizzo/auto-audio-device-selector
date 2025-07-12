use anyhow::Result;
use core_foundation::runloop::CFRunLoop;
use coreaudio_sys::*;
use std::os::raw::c_void;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::AudioDevice;
use super::controller::DeviceController;
use crate::config::Config;
use crate::notifications::{NotificationManager, SwitchReason};
use crate::priority::DevicePriorityManager;

pub struct CoreAudioListener {
    controller: DeviceController,
    priority_manager: Arc<Mutex<DevicePriorityManager>>,
    notification_manager: NotificationManager,
    device_list_address: AudioObjectPropertyAddress,
    default_output_address: AudioObjectPropertyAddress,
    default_input_address: AudioObjectPropertyAddress,
    previous_devices: Arc<Mutex<Vec<AudioDevice>>>,
}

impl CoreAudioListener {
    pub fn new(config: &Config) -> Result<Self> {
        info!("Creating CoreAudio listener");

        let controller = DeviceController::new()?;
        let priority_manager = Arc::new(Mutex::new(DevicePriorityManager::new(config)));
        let notification_manager = NotificationManager::new(config);

        // Property addresses for listening to device changes
        let device_list_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let default_output_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultOutputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let default_input_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultInputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        // Initialize with current devices to avoid false notifications on startup
        let initial_devices = controller.enumerate_devices().unwrap_or_default();

        Ok(Self {
            controller,
            priority_manager,
            notification_manager,
            device_list_address,
            default_output_address,
            default_input_address,
            previous_devices: Arc::new(Mutex::new(initial_devices)),
        })
    }

    pub fn register_listeners(&self) -> Result<()> {
        info!("Registering CoreAudio property listeners");

        unsafe {
            // Register listener for device list changes
            let result = AudioObjectAddPropertyListener(
                kAudioObjectSystemObject,
                &self.device_list_address,
                Some(device_list_listener),
                self as *const _ as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                error!("Failed to register device list listener: {}", result);
                return Err(anyhow::anyhow!("Failed to register device list listener"));
            }

            // Register listener for default output device changes
            let result = AudioObjectAddPropertyListener(
                kAudioObjectSystemObject,
                &self.default_output_address,
                Some(default_output_listener),
                self as *const _ as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                error!("Failed to register default output listener: {}", result);
                return Err(anyhow::anyhow!(
                    "Failed to register default output listener"
                ));
            }

            // Register listener for default input device changes
            let result = AudioObjectAddPropertyListener(
                kAudioObjectSystemObject,
                &self.default_input_address,
                Some(default_input_listener),
                self as *const _ as *mut c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                error!("Failed to register default input listener: {}", result);
                return Err(anyhow::anyhow!("Failed to register default input listener"));
            }
        }

        info!("CoreAudio property listeners registered successfully");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn start_monitoring(&self) -> Result<()> {
        info!("Starting CoreAudio device monitoring");

        // Register all property listeners
        self.register_listeners()?;

        // Start Core Foundation run loop
        info!("Starting Core Foundation run loop");
        unsafe {
            CFRunLoop::run_in_mode(
                core_foundation::runloop::kCFRunLoopDefaultMode,
                Duration::from_secs(u64::MAX),
                false,
            );
        }

        Ok(())
    }

    pub fn stop_monitoring(&self) -> Result<()> {
        info!("Stopping CoreAudio device monitoring");

        unsafe {
            // Remove all property listeners
            AudioObjectRemovePropertyListener(
                kAudioObjectSystemObject,
                &self.device_list_address,
                Some(device_list_listener),
                self as *const _ as *mut c_void,
            );

            AudioObjectRemovePropertyListener(
                kAudioObjectSystemObject,
                &self.default_output_address,
                Some(default_output_listener),
                self as *const _ as *mut c_void,
            );

            AudioObjectRemovePropertyListener(
                kAudioObjectSystemObject,
                &self.default_input_address,
                Some(default_input_listener),
                self as *const _ as *mut c_void,
            );

            // Stop the run loop
            CFRunLoop::get_current().stop();
        }

        Ok(())
    }

    fn handle_device_list_change(&self) {
        debug!("Device list changed");

        // Get current available devices
        match self.controller.enumerate_devices() {
            Ok(current_devices) => {
                info!(
                    "Device list updated, found {} devices",
                    current_devices.len()
                );

                // Check for device connections/disconnections and send notifications
                if let Ok(mut previous_devices) = self.previous_devices.lock() {
                    // Find newly connected devices
                    for device in &current_devices {
                        if !previous_devices.iter().any(|prev| prev.uid == device.uid) {
                            // Device was connected
                            if let Err(e) = self.notification_manager.device_connected(device) {
                                warn!("Failed to send device connected notification: {}", e);
                            }
                        }
                    }

                    // Find disconnected devices
                    for prev_device in &*previous_devices {
                        if !current_devices
                            .iter()
                            .any(|curr| curr.uid == prev_device.uid)
                        {
                            // Device was disconnected
                            if let Err(e) =
                                self.notification_manager.device_disconnected(prev_device)
                            {
                                warn!("Failed to send device disconnected notification: {}", e);
                            }
                        }
                    }

                    // Update previous devices list
                    *previous_devices = current_devices.clone();
                }

                // Check if we need to switch to a higher priority device
                if let Ok(priority_manager) = self.priority_manager.lock() {
                    let output_devices: Vec<_> = current_devices
                        .iter()
                        .filter(|d| matches!(d.device_type, crate::audio::DeviceType::Output))
                        .cloned()
                        .collect();

                    let input_devices: Vec<_> = current_devices
                        .iter()
                        .filter(|d| matches!(d.device_type, crate::audio::DeviceType::Input))
                        .cloned()
                        .collect();

                    // Find best available devices
                    if let Some(best_output) =
                        priority_manager.find_best_output_device(&output_devices)
                    {
                        if priority_manager.should_switch_output(&best_output) {
                            info!("Switching to output device: {}", best_output.name);
                            match self.controller.set_default_output_device(&best_output.name) {
                                Ok(()) => {
                                    info!(
                                        "Successfully switched to output device: {}",
                                        best_output.name
                                    );
                                    // Send notification for successful switch
                                    if let Err(e) = self
                                        .notification_manager
                                        .device_switched(&best_output, SwitchReason::HigherPriority)
                                    {
                                        warn!("Failed to send device switched notification: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to switch output device: {}", e);
                                    // Send notification for failed switch
                                    if let Err(e) = self
                                        .notification_manager
                                        .switch_failed(&best_output.name, &e.to_string())
                                    {
                                        warn!("Failed to send switch failed notification: {}", e);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(best_input) =
                        priority_manager.find_best_input_device(&input_devices)
                    {
                        if priority_manager.should_switch_input(&best_input) {
                            info!("Switching to input device: {}", best_input.name);
                            match self.controller.set_default_input_device(&best_input.name) {
                                Ok(()) => {
                                    info!(
                                        "Successfully switched to input device: {}",
                                        best_input.name
                                    );
                                    // Send notification for successful switch
                                    if let Err(e) = self
                                        .notification_manager
                                        .device_switched(&best_input, SwitchReason::HigherPriority)
                                    {
                                        warn!("Failed to send device switched notification: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to switch input device: {}", e);
                                    // Send notification for failed switch
                                    if let Err(e) = self
                                        .notification_manager
                                        .switch_failed(&best_input.name, &e.to_string())
                                    {
                                        warn!("Failed to send switch failed notification: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to enumerate devices: {}", e);
            }
        }
    }

    fn handle_default_output_change(&self) {
        debug!("Default output device changed");

        match self.controller.get_default_output_device() {
            Ok(Some(device)) => {
                info!("Default output device is now: {}", device.name);

                if let Ok(mut priority_manager) = self.priority_manager.lock() {
                    priority_manager.update_current_output(device.name);
                }
            }
            Ok(None) => {
                warn!("No default output device available");
            }
            Err(e) => {
                error!("Failed to get default output device: {}", e);
            }
        }
    }

    fn handle_default_input_change(&self) {
        debug!("Default input device changed");

        match self.controller.get_default_input_device() {
            Ok(Some(device)) => {
                info!("Default input device is now: {}", device.name);

                if let Ok(mut priority_manager) = self.priority_manager.lock() {
                    priority_manager.update_current_input(device.name);
                }
            }
            Ok(None) => {
                warn!("No default input device available");
            }
            Err(e) => {
                error!("Failed to get default input device: {}", e);
            }
        }
    }
}

// CoreAudio callback functions
extern "C" fn device_list_listener(
    _in_object_id: AudioObjectID,
    _in_number_addresses: UInt32,
    _in_addresses: *const AudioObjectPropertyAddress,
    in_client_data: *mut c_void,
) -> OSStatus {
    if !in_client_data.is_null() {
        let listener = unsafe { &*(in_client_data as *const CoreAudioListener) };
        listener.handle_device_list_change();
    }
    kAudioHardwareNoError as i32
}

extern "C" fn default_output_listener(
    _in_object_id: AudioObjectID,
    _in_number_addresses: UInt32,
    _in_addresses: *const AudioObjectPropertyAddress,
    in_client_data: *mut c_void,
) -> OSStatus {
    if !in_client_data.is_null() {
        let listener = unsafe { &*(in_client_data as *const CoreAudioListener) };
        listener.handle_default_output_change();
    }
    kAudioHardwareNoError as i32
}

extern "C" fn default_input_listener(
    _in_object_id: AudioObjectID,
    _in_number_addresses: UInt32,
    _in_addresses: *const AudioObjectPropertyAddress,
    in_client_data: *mut c_void,
) -> OSStatus {
    if !in_client_data.is_null() {
        let listener = unsafe { &*(in_client_data as *const CoreAudioListener) };
        listener.handle_default_input_change();
    }
    kAudioHardwareNoError as i32
}
