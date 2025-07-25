use anyhow::Result;
use std::path::Path;

use crate::audio::AudioDevice;

/// Trait for audio system operations - abstracts CoreAudio and cpal interactions
pub trait AudioSystemInterface {
    /// Enumerate all available audio devices
    fn enumerate_devices(&self) -> Result<Vec<AudioDevice>>;

    /// Get the current default output device
    fn get_default_output_device(&self) -> Result<Option<AudioDevice>>;

    /// Get the current default input device
    fn get_default_input_device(&self) -> Result<Option<AudioDevice>>;

    /// Set the system default output device by device ID
    fn set_default_output_device(&self, device_id: &str) -> Result<()>;

    /// Set the system default input device by device ID
    fn set_default_input_device(&self, device_id: &str) -> Result<()>;

    /// Register a callback for device change notifications
    /// The callback will be invoked when devices are added, removed, or default devices change
    fn add_device_change_listener(&self, callback: Box<dyn Fn() + Send + Sync>) -> Result<()>;

    /// Check if a specific device is currently available
    fn is_device_available(&self, device_id: &str) -> Result<bool>;
}

/// Trait for file system operations - abstracts std::fs for testability
pub trait FileSystemInterface {
    /// Read the entire contents of a configuration file
    fn read_config_file(&self, path: &Path) -> Result<String>;

    /// Write configuration content to a file
    fn write_config_file(&self, path: &Path, content: &str) -> Result<()>;

    /// Check if a configuration file exists
    fn config_file_exists(&self, path: &Path) -> bool;

    /// Create the directory structure for config files
    fn create_config_dir(&self, path: &Path) -> Result<()>;

    /// Get the last modified time of a config file (for watching changes)
    fn get_config_modified_time(&self, path: &Path) -> Result<std::time::SystemTime>;
}

/// Trait for system service operations - abstracts daemon, signals, and event loops
pub trait SystemServiceInterface {
    /// Register signal handlers for SIGHUP, SIGTERM, etc.
    fn register_signal_handlers(&self) -> Result<()>;

    /// Run the main event loop for the specified duration
    /// Returns when the service should stop or an error occurs
    fn run_event_loop(&self) -> Result<()>;

    /// Check if the service should continue running
    /// Returns false when termination signals are received
    fn should_continue_running(&self) -> bool;

    /// Sleep for the specified number of milliseconds
    /// Can be interrupted by signals
    fn sleep_ms(&self, milliseconds: u64) -> Result<()>;

    /// Get the process ID of the current service
    fn get_process_id(&self) -> u32;
}
