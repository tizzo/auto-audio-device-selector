use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::audio::AudioDevice;
use crate::system::traits::{AudioSystemInterface, FileSystemInterface, SystemServiceInterface};

/// Mock audio system for testing - provides controllable device behavior
pub struct MockAudioSystem {
    pub devices: Arc<Mutex<Vec<AudioDevice>>>,
    pub default_output: Arc<Mutex<Option<AudioDevice>>>,
    pub default_input: Arc<Mutex<Option<AudioDevice>>>,
    pub device_change_callbacks: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
    pub set_device_calls: Arc<Mutex<Vec<(String, String)>>>, // (device_id, call_type)
    pub should_fail_enumeration: Arc<Mutex<bool>>,
    pub should_fail_set_device: Arc<Mutex<bool>>,
}

impl MockAudioSystem {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(Vec::new())),
            default_output: Arc::new(Mutex::new(None)),
            default_input: Arc::new(Mutex::new(None)),
            device_change_callbacks: Arc::new(Mutex::new(Vec::new())),
            set_device_calls: Arc::new(Mutex::new(Vec::new())),
            should_fail_enumeration: Arc::new(Mutex::new(false)),
            should_fail_set_device: Arc::new(Mutex::new(false)),
        }
    }

    /// Add a device to the mock system
    pub fn add_device(&self, device: AudioDevice) {
        self.devices.lock().unwrap().push(device);
        self.trigger_device_change();
    }

    /// Remove a device from the mock system
    pub fn remove_device(&self, device_id: &str) {
        self.devices
            .lock()
            .unwrap()
            .retain(|d| d.id != device_id && d.name != device_id);
        self.trigger_device_change();
    }

    /// Set the default output device
    pub fn set_mock_default_output(&self, device: Option<AudioDevice>) {
        *self.default_output.lock().unwrap() = device;
        self.trigger_device_change();
    }

    /// Set the default input device
    pub fn set_mock_default_input(&self, device: Option<AudioDevice>) {
        *self.default_input.lock().unwrap() = device;
        self.trigger_device_change();
    }

    /// Trigger all registered device change callbacks
    pub fn trigger_device_change(&self) {
        let callbacks = self.device_change_callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback();
        }
    }

    /// Get all set device calls that were made
    pub fn get_set_device_calls(&self) -> Vec<(String, String)> {
        self.set_device_calls.lock().unwrap().clone()
    }

    /// Clear the history of set device calls
    pub fn clear_set_device_calls(&self) {
        self.set_device_calls.lock().unwrap().clear();
    }

    /// Configure the mock to fail enumeration
    pub fn set_enumeration_failure(&self, should_fail: bool) {
        *self.should_fail_enumeration.lock().unwrap() = should_fail;
    }

    /// Configure the mock to fail device setting
    pub fn set_device_setting_failure(&self, should_fail: bool) {
        *self.should_fail_set_device.lock().unwrap() = should_fail;
    }

    /// Get count of registered callbacks
    pub fn callback_count(&self) -> usize {
        self.device_change_callbacks.lock().unwrap().len()
    }
}

impl AudioSystemInterface for MockAudioSystem {
    fn enumerate_devices(&self) -> Result<Vec<AudioDevice>> {
        if *self.should_fail_enumeration.lock().unwrap() {
            return Err(anyhow::anyhow!("Mock enumeration failure"));
        }
        Ok(self.devices.lock().unwrap().clone())
    }

    fn get_default_output_device(&self) -> Result<Option<AudioDevice>> {
        Ok(self.default_output.lock().unwrap().clone())
    }

    fn get_default_input_device(&self) -> Result<Option<AudioDevice>> {
        Ok(self.default_input.lock().unwrap().clone())
    }

    fn set_default_output_device(&self, device_id: &str) -> Result<()> {
        if *self.should_fail_set_device.lock().unwrap() {
            return Err(anyhow::anyhow!("Mock set device failure"));
        }

        self.set_device_calls
            .lock()
            .unwrap()
            .push((device_id.to_string(), "output".to_string()));

        // Find and set the device as default if it exists
        let devices = self.devices.lock().unwrap();
        if let Some(device) = devices
            .iter()
            .find(|d| d.id == device_id || d.name == device_id)
        {
            *self.default_output.lock().unwrap() = Some(device.clone());
        }

        Ok(())
    }

    fn set_default_input_device(&self, device_id: &str) -> Result<()> {
        if *self.should_fail_set_device.lock().unwrap() {
            return Err(anyhow::anyhow!("Mock set device failure"));
        }

        self.set_device_calls
            .lock()
            .unwrap()
            .push((device_id.to_string(), "input".to_string()));

        // Find and set the device as default if it exists
        let devices = self.devices.lock().unwrap();
        if let Some(device) = devices
            .iter()
            .find(|d| d.id == device_id || d.name == device_id)
        {
            *self.default_input.lock().unwrap() = Some(device.clone());
        }

        Ok(())
    }

    fn add_device_change_listener(&self, callback: Box<dyn Fn() + Send + Sync>) -> Result<()> {
        self.device_change_callbacks.lock().unwrap().push(callback);
        Ok(())
    }

    fn is_device_available(&self, device_id: &str) -> Result<bool> {
        let devices = self.devices.lock().unwrap();
        Ok(devices
            .iter()
            .any(|d| d.id == device_id || d.name == device_id))
    }
}

impl Default for MockAudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock file system for testing - provides controllable file operations
pub struct MockFileSystem {
    pub files: Arc<Mutex<HashMap<PathBuf, String>>>,
    pub read_calls: Arc<Mutex<Vec<PathBuf>>>,
    pub write_calls: Arc<Mutex<Vec<(PathBuf, String)>>>,
    pub directory_creation_calls: Arc<Mutex<Vec<PathBuf>>>,
    pub should_fail_read: Arc<Mutex<bool>>,
    pub should_fail_write: Arc<Mutex<bool>>,
    pub should_fail_create_dir: Arc<Mutex<bool>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            read_calls: Arc::new(Mutex::new(Vec::new())),
            write_calls: Arc::new(Mutex::new(Vec::new())),
            directory_creation_calls: Arc::new(Mutex::new(Vec::new())),
            should_fail_read: Arc::new(Mutex::new(false)),
            should_fail_write: Arc::new(Mutex::new(false)),
            should_fail_create_dir: Arc::new(Mutex::new(false)),
        }
    }

    /// Add a file to the mock file system
    pub fn add_file<P: AsRef<Path>>(&self, path: P, content: String) {
        self.files
            .lock()
            .unwrap()
            .insert(path.as_ref().to_path_buf(), content);
    }

    /// Remove a file from the mock file system
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) {
        self.files
            .lock()
            .unwrap()
            .remove(&path.as_ref().to_path_buf());
    }

    /// Get all read calls that were made
    pub fn get_read_calls(&self) -> Vec<PathBuf> {
        self.read_calls.lock().unwrap().clone()
    }

    /// Get all write calls that were made
    pub fn get_write_calls(&self) -> Vec<(PathBuf, String)> {
        self.write_calls.lock().unwrap().clone()
    }

    /// Get all directory creation calls that were made
    pub fn get_directory_creation_calls(&self) -> Vec<PathBuf> {
        self.directory_creation_calls.lock().unwrap().clone()
    }

    /// Clear all call histories
    pub fn clear_call_history(&self) {
        self.read_calls.lock().unwrap().clear();
        self.write_calls.lock().unwrap().clear();
        self.directory_creation_calls.lock().unwrap().clear();
    }

    /// Configure the mock to fail read operations
    pub fn set_read_failure(&self, should_fail: bool) {
        *self.should_fail_read.lock().unwrap() = should_fail;
    }

    /// Configure the mock to fail write operations
    pub fn set_write_failure(&self, should_fail: bool) {
        *self.should_fail_write.lock().unwrap() = should_fail;
    }

    /// Configure the mock to fail directory creation
    pub fn set_create_dir_failure(&self, should_fail: bool) {
        *self.should_fail_create_dir.lock().unwrap() = should_fail;
    }

    /// Check if a file exists in the mock system
    pub fn file_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.files
            .lock()
            .unwrap()
            .contains_key(&path.as_ref().to_path_buf())
    }
}

impl FileSystemInterface for MockFileSystem {
    fn read_config_file(&self, path: &Path) -> Result<String> {
        self.read_calls.lock().unwrap().push(path.to_path_buf());

        if *self.should_fail_read.lock().unwrap() {
            return Err(anyhow::anyhow!("Mock read failure"));
        }

        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", path.display()))
    }

    fn write_config_file(&self, path: &Path, content: &str) -> Result<()> {
        self.write_calls
            .lock()
            .unwrap()
            .push((path.to_path_buf(), content.to_string()));

        if *self.should_fail_write.lock().unwrap() {
            return Err(anyhow::anyhow!("Mock write failure"));
        }

        self.files
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    fn config_file_exists(&self, path: &Path) -> bool {
        self.files.lock().unwrap().contains_key(&path.to_path_buf())
    }

    fn create_config_dir(&self, path: &Path) -> Result<()> {
        self.directory_creation_calls
            .lock()
            .unwrap()
            .push(path.to_path_buf());

        if *self.should_fail_create_dir.lock().unwrap() {
            return Err(anyhow::anyhow!("Mock create directory failure"));
        }

        Ok(())
    }

    fn get_config_modified_time(&self, path: &Path) -> Result<std::time::SystemTime> {
        if !self.config_file_exists(path) {
            return Err(anyhow::anyhow!("File not found: {}", path.display()));
        }
        // Return a fixed time for testing
        Ok(std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000))
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock system service for testing - provides controllable service behavior
pub struct MockSystemService {
    pub should_run: Arc<std::sync::atomic::AtomicBool>,
    pub signal_handler_registered: Arc<std::sync::atomic::AtomicBool>,
    pub event_loop_calls: Arc<std::sync::atomic::AtomicUsize>,
    pub sleep_calls: Arc<Mutex<Vec<u64>>>,
    pub should_fail_signal_registration: Arc<std::sync::atomic::AtomicBool>,
    pub should_fail_event_loop: Arc<std::sync::atomic::AtomicBool>,
}

impl MockSystemService {
    pub fn new() -> Self {
        Self {
            should_run: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            signal_handler_registered: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            event_loop_calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            sleep_calls: Arc::new(Mutex::new(Vec::new())),
            should_fail_signal_registration: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            should_fail_event_loop: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Stop the service (simulate signal reception)
    pub fn stop_service(&self) {
        self.should_run
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Start the service
    pub fn start_service(&self) {
        self.should_run
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if signal handlers were registered
    pub fn are_signal_handlers_registered(&self) -> bool {
        self.signal_handler_registered
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get the number of event loop calls
    pub fn get_event_loop_call_count(&self) -> usize {
        self.event_loop_calls
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get all sleep calls that were made
    pub fn get_sleep_calls(&self) -> Vec<u64> {
        self.sleep_calls.lock().unwrap().clone()
    }

    /// Clear sleep call history
    pub fn clear_sleep_calls(&self) {
        self.sleep_calls.lock().unwrap().clear();
    }

    /// Configure the mock to fail signal registration
    pub fn set_signal_registration_failure(&self, should_fail: bool) {
        self.should_fail_signal_registration
            .store(should_fail, std::sync::atomic::Ordering::Relaxed);
    }

    /// Configure the mock to fail event loop
    pub fn set_event_loop_failure(&self, should_fail: bool) {
        self.should_fail_event_loop
            .store(should_fail, std::sync::atomic::Ordering::Relaxed);
    }

    /// Reset all counters and state
    pub fn reset(&self) {
        self.should_run
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.signal_handler_registered
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.event_loop_calls
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.sleep_calls.lock().unwrap().clear();
        self.should_fail_signal_registration
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.should_fail_event_loop
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

impl SystemServiceInterface for MockSystemService {
    fn register_signal_handlers(&self) -> Result<()> {
        if self
            .should_fail_signal_registration
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(anyhow::anyhow!("Mock signal registration failure"));
        }

        self.signal_handler_registered
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn run_event_loop(&self) -> Result<()> {
        if self
            .should_fail_event_loop
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(anyhow::anyhow!("Mock event loop failure"));
        }

        self.event_loop_calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn should_continue_running(&self) -> bool {
        self.should_run.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn sleep_ms(&self, milliseconds: u64) -> Result<()> {
        self.sleep_calls.lock().unwrap().push(milliseconds);
        // Don't actually sleep in tests
        Ok(())
    }

    fn get_process_id(&self) -> u32 {
        // Return a fixed process ID for testing
        12345
    }
}

impl Default for MockSystemService {
    fn default() -> Self {
        Self::new()
    }
}
