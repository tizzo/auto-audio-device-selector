use anyhow::Result;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::audio::{AudioDevice, DeviceController};
use crate::system::traits::{AudioSystemInterface, FileSystemInterface, SystemServiceInterface};

/// Production implementation of AudioSystemInterface using CoreAudio
pub struct CoreAudioSystem {
    controller: DeviceController,
    callbacks: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
}

impl CoreAudioSystem {
    pub fn new() -> Result<Self> {
        Ok(Self {
            controller: DeviceController::new()?,
            callbacks: Arc::new(Mutex::new(Vec::new())),
        })
    }
}

impl AudioSystemInterface for CoreAudioSystem {
    fn enumerate_devices(&self) -> Result<Vec<AudioDevice>> {
        self.controller.enumerate_devices()
    }

    fn get_default_output_device(&self) -> Result<Option<AudioDevice>> {
        self.controller.get_default_output_device()
    }

    fn get_default_input_device(&self) -> Result<Option<AudioDevice>> {
        self.controller.get_default_input_device()
    }

    fn set_default_output_device(&self, device_id: &str) -> Result<()> {
        // DeviceController expects device name, but we're passing device_id
        // For now, treat device_id as device name - this may need refinement
        self.controller.set_default_output_device(device_id)
    }

    fn set_default_input_device(&self, device_id: &str) -> Result<()> {
        // DeviceController expects device name, but we're passing device_id
        // For now, treat device_id as device name - this may need refinement
        self.controller.set_default_input_device(device_id)
    }

    fn add_device_change_listener(&self, callback: Box<dyn Fn() + Send + Sync>) -> Result<()> {
        // Store the callback for now
        // TODO: Implement actual CoreAudio property listeners
        self.callbacks.lock().unwrap().push(callback);
        Ok(())
    }

    fn is_device_available(&self, device_id: &str) -> Result<bool> {
        // Check if device exists in enumerated devices
        let devices = self.enumerate_devices()?;
        Ok(devices
            .iter()
            .any(|d| d.id == device_id || d.name == device_id))
    }
}

/// Production implementation of FileSystemInterface using std::fs
pub struct StandardFileSystem;

impl FileSystemInterface for StandardFileSystem {
    fn read_config_file(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))
    }

    fn write_config_file(&self, path: &Path, content: &str) -> Result<()> {
        std::fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write config file: {}", e))
    }

    fn config_file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_config_dir(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)
            .map_err(|e| anyhow::anyhow!("Failed to create config directory: {}", e))
    }

    fn get_config_modified_time(&self, path: &Path) -> Result<std::time::SystemTime> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| anyhow::anyhow!("Failed to get file metadata: {}", e))?;
        metadata
            .modified()
            .map_err(|e| anyhow::anyhow!("Failed to get modified time: {}", e))
    }
}

/// Production implementation of SystemServiceInterface for macOS
pub struct MacOSSystemService {
    should_continue: Arc<std::sync::atomic::AtomicBool>,
}

impl MacOSSystemService {
    pub fn new() -> Self {
        Self {
            should_continue: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }
}

impl SystemServiceInterface for MacOSSystemService {
    fn register_signal_handlers(&self) -> Result<()> {
        // TODO: Implement actual signal handling
        // For now, just return success
        tracing::info!("Signal handlers registered (placeholder implementation)");
        Ok(())
    }

    fn run_event_loop(&self) -> Result<()> {
        // Simple event loop that sleeps
        // TODO: Implement proper Core Foundation event loop
        self.sleep_ms(100)?;
        Ok(())
    }

    fn should_continue_running(&self) -> bool {
        self.should_continue
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn sleep_ms(&self, milliseconds: u64) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
        Ok(())
    }

    fn get_process_id(&self) -> u32 {
        std::process::id()
    }
}

// Default implementations for production use
impl Default for CoreAudioSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create CoreAudio system")
    }
}

impl Default for StandardFileSystem {
    fn default() -> Self {
        Self
    }
}

impl Default for MacOSSystemService {
    fn default() -> Self {
        Self::new()
    }
}
