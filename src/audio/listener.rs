use anyhow::Result;
use core_foundation::runloop::CFRunLoop;
use coreaudio_sys::*;
use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use super::AudioDevice;
use super::controller::DeviceController;
use crate::config::Config;
use crate::notifications::{DefaultNotificationManager, SwitchReason};
use crate::priority::DevicePriorityManager;

/// Time a device must be present before we consider it stable for switching
const DEVICE_STABILITY_THRESHOLD_MS: u64 = 750;

/// Extended stability threshold for Bluetooth devices (input/output may appear separately)
const BLUETOOTH_DEVICE_STABILITY_THRESHOLD_MS: u64 = 1500;

pub struct CoreAudioListener {
    controller: DeviceController,
    priority_manager: Arc<Mutex<DevicePriorityManager>>,
    notification_manager: DefaultNotificationManager,
    device_list_address: AudioObjectPropertyAddress,
    default_output_address: AudioObjectPropertyAddress,
    default_input_address: AudioObjectPropertyAddress,
    previous_devices: Arc<Mutex<Vec<AudioDevice>>>,
    // Track when devices first appeared to implement debouncing
    device_appearance_times: Arc<Mutex<HashMap<String, Instant>>>,
}

impl CoreAudioListener {
    pub fn new(config: &Config) -> Result<Self> {
        debug!("Creating CoreAudio listener");

        let controller = DeviceController::new()?;
        let priority_manager = Arc::new(Mutex::new(DevicePriorityManager::new(config)));
        let notification_manager = DefaultNotificationManager::new(config);

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

        // Initialize appearance times for existing devices
        let mut appearance_times = HashMap::new();
        let now = Instant::now();
        for device in &initial_devices {
            appearance_times.insert(device.id.clone(), now);
        }

        Ok(Self {
            controller,
            priority_manager,
            notification_manager,
            device_list_address,
            default_output_address,
            default_input_address,
            previous_devices: Arc::new(Mutex::new(initial_devices)),
            device_appearance_times: Arc::new(Mutex::new(appearance_times)),
        })
    }

    pub fn register_listeners(&self) -> Result<()> {
        info!("Registering CoreAudio property listeners");

        unsafe {
            // Configure CFRunLoop for CoreAudio property listeners
            // This is critical for reliable event delivery, especially for rapid device changes
            // Setting to NULL tells CoreAudio to manage its own thread for notifications
            info!("Configuring CoreAudio run loop for property notifications");
            let run_loop: *const std::ffi::c_void = std::ptr::null();
            let run_loop_address = AudioObjectPropertyAddress {
                mSelector: kAudioHardwarePropertyRunLoop,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain,
            };

            let result = AudioObjectSetPropertyData(
                kAudioObjectSystemObject,
                &run_loop_address,
                0,
                std::ptr::null(),
                std::mem::size_of::<*const std::ffi::c_void>() as u32,
                &run_loop as *const _ as *const std::ffi::c_void,
            );

            if result != kAudioHardwareNoError as i32 {
                warn!("Failed to set run loop property: {}", result);
                // Continue anyway - this is an optimization, not critical
            } else {
                info!("CoreAudio run loop configured successfully");
            }

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

    /// Check if a device is likely a Bluetooth device based on its name
    fn is_likely_bluetooth_device(device_name: &str) -> bool {
        let bluetooth_keywords = [
            "airpod",
            "bluetooth",
            "beats",
            "bose",
            "sony",
            "jabra",
            "jbl",
        ];
        let name_lower = device_name.to_lowercase();
        bluetooth_keywords
            .iter()
            .any(|keyword| name_lower.contains(keyword))
    }

    /// Check if both input and output devices exist for a given device name pattern
    fn has_paired_input_output(devices: &[AudioDevice], device_name: &str) -> bool {
        let has_output = devices.iter().any(|d| {
            d.name.contains(device_name)
                && matches!(d.device_type, crate::audio::DeviceType::Output)
        });
        let has_input = devices.iter().any(|d| {
            d.name.contains(device_name) && matches!(d.device_type, crate::audio::DeviceType::Input)
        });
        has_output && has_input
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

                let now = Instant::now();

                // Check for device connections/disconnections and send notifications
                if let Ok(mut previous_devices) = self.previous_devices.lock() {
                    if let Ok(mut appearance_times) = self.device_appearance_times.lock() {
                        // Find newly connected devices
                        for device in &current_devices {
                            if !previous_devices.iter().any(|prev| prev.id == device.id) {
                                // Device was connected - record appearance time
                                appearance_times.insert(device.id.clone(), now);
                                info!(
                                    "New device detected: {} (will debounce for {}ms)",
                                    device.name, DEVICE_STABILITY_THRESHOLD_MS
                                );

                                if let Err(e) = self.notification_manager.device_connected(device) {
                                    warn!("Failed to send device connected notification: {}", e);
                                }
                            }
                        }

                        // Find disconnected devices and clean up appearance times
                        for prev_device in &*previous_devices {
                            if !current_devices.iter().any(|curr| curr.id == prev_device.id) {
                                // Device was disconnected
                                appearance_times.remove(&prev_device.id);
                                info!("Device disconnected: {}", prev_device.name);

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
                }

                // Check if we need to switch to a higher priority device
                // Only consider devices that have been stable for the threshold duration
                if let Ok(priority_manager) = self.priority_manager.lock() {
                    if let Ok(appearance_times) = self.device_appearance_times.lock() {
                        // Filter devices to only those that are stable
                        // Use extended threshold for Bluetooth devices that may have separate input/output
                        let stable_devices: Vec<_> = current_devices
                            .iter()
                            .filter(|d| {
                                appearance_times
                                    .get(&d.id)
                                    .map(|&appeared_at| {
                                        let elapsed = now.duration_since(appeared_at);
                                        let is_bluetooth =
                                            Self::is_likely_bluetooth_device(&d.name);
                                        let threshold = if is_bluetooth {
                                            BLUETOOTH_DEVICE_STABILITY_THRESHOLD_MS
                                        } else {
                                            DEVICE_STABILITY_THRESHOLD_MS
                                        };

                                        // For Bluetooth devices, also check if paired device exists
                                        if is_bluetooth && elapsed.as_millis() >= threshold as u128
                                        {
                                            // Extract common name part (e.g., "AirPods Pro" from "AirPods Pro - Output")
                                            let base_name =
                                                d.name.split('-').next().unwrap_or(&d.name).trim();
                                            Self::has_paired_input_output(
                                                &current_devices,
                                                base_name,
                                            )
                                        } else {
                                            elapsed.as_millis() >= threshold as u128
                                        }
                                    })
                                    .unwrap_or(false)
                            })
                            .cloned()
                            .collect();

                        let stable_output_devices: Vec<_> = stable_devices
                            .iter()
                            .filter(|d| matches!(d.device_type, crate::audio::DeviceType::Output))
                            .cloned()
                            .collect();

                        let stable_input_devices: Vec<_> = stable_devices
                            .iter()
                            .filter(|d| matches!(d.device_type, crate::audio::DeviceType::Input))
                            .cloned()
                            .collect();

                        let bluetooth_count = stable_devices
                            .iter()
                            .filter(|d| Self::is_likely_bluetooth_device(&d.name))
                            .count();
                        debug!(
                            "Found {} stable devices out of {} total ({} Bluetooth with {}ms threshold, {} other with {}ms threshold)",
                            stable_devices.len(),
                            current_devices.len(),
                            bluetooth_count,
                            BLUETOOTH_DEVICE_STABILITY_THRESHOLD_MS,
                            stable_devices.len() - bluetooth_count,
                            DEVICE_STABILITY_THRESHOLD_MS
                        );

                        // Find best available stable devices
                        if let Some(best_output) =
                            priority_manager.find_best_output_device(&stable_output_devices)
                        {
                            if priority_manager.should_switch_output(&best_output) {
                                info!("Switching to stable output device: {}", best_output.name);
                                match self.controller.set_default_output_device(&best_output.name) {
                                    Ok(()) => {
                                        info!(
                                            "Successfully switched to output device: {}",
                                            best_output.name
                                        );
                                        // Send notification for successful switch
                                        if let Err(e) = self.notification_manager.device_switched(
                                            &best_output,
                                            SwitchReason::HigherPriority,
                                        ) {
                                            warn!(
                                                "Failed to send device switched notification: {}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to switch output device: {}", e);
                                        // Send notification for failed switch
                                        if let Err(e) = self
                                            .notification_manager
                                            .switch_failed(&best_output.name, &e.to_string())
                                        {
                                            warn!(
                                                "Failed to send switch failed notification: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(best_input) =
                            priority_manager.find_best_input_device(&stable_input_devices)
                        {
                            if priority_manager.should_switch_input(&best_input) {
                                info!("Switching to stable input device: {}", best_input.name);
                                match self.controller.set_default_input_device(&best_input.name) {
                                    Ok(()) => {
                                        info!(
                                            "Successfully switched to input device: {}",
                                            best_input.name
                                        );
                                        // Send notification for successful switch
                                        if let Err(e) = self.notification_manager.device_switched(
                                            &best_input,
                                            SwitchReason::HigherPriority,
                                        ) {
                                            warn!(
                                                "Failed to send device switched notification: {}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to switch input device: {}", e);
                                        // Send notification for failed switch
                                        if let Err(e) = self
                                            .notification_manager
                                            .switch_failed(&best_input.name, &e.to_string())
                                        {
                                            warn!(
                                                "Failed to send switch failed notification: {}",
                                                e
                                            );
                                        }
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
