pub mod audio;
pub mod config;
pub mod notifications;
pub mod priority;
pub mod system;

pub use audio::{AudioDeviceMonitor, DeviceControllerV2};
pub use config::Config;
pub use notifications::{NotificationManager, SwitchReason, TestNotificationSender};

// Export system traits and adapters
pub use system::{
    AudioSystemInterface, CoreAudioSystem, FileSystemInterface, MacOSSystemService,
    StandardFileSystem, SystemServiceInterface,
};

// Export mock implementations for testing
#[cfg(test)]
pub use system::{MockAudioSystem, MockFileSystem, MockSystemService};
