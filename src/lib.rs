pub mod audio;
pub mod config;
pub mod notifications;
pub mod priority;
pub mod service;
pub mod system;

pub use audio::{AudioDevice, AudioDeviceMonitor, DeviceControllerV2, DeviceType};
pub use config::{Config, ConfigLoader};
pub use notifications::{NotificationManager, SwitchReason, TestNotificationSender};
pub use service::AudioDeviceService;

// Export system traits and adapters
pub use system::{
    AudioSystemInterface, CoreAudioSystem, FileSystemInterface, MacOSSystemService,
    StandardFileSystem, SystemServiceInterface,
};

// Export mock implementations for testing (available for both unit and integration tests)
pub use system::{MockAudioSystem, MockFileSystem, MockSystemService};
