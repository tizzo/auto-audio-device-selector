pub mod audio;
pub mod config;
pub mod notifications;
pub mod preference_debugging;
pub mod priority;
pub mod service;
pub mod system;

pub use audio::{AudioDevice, AudioDeviceMonitor, DeviceControllerV2, DeviceType};
pub use config::{Config, ConfigLoader};
pub use notifications::{DefaultNotificationManager, NotificationManager, SwitchReason};
pub use preference_debugging::{PreferenceChanges, PreferenceStatus};

#[cfg(any(test, feature = "test-mocks"))]
pub use notifications::TestNotificationSender;
pub use service::AudioDeviceService;

// Re-export common functionality for library users
pub use audio::controller::DeviceController;

// Export system traits and adapters
pub use system::{
    AudioSystemInterface, CoreAudioSystem, FileSystemInterface, MacOSSystemService,
    StandardFileSystem, SystemServiceInterface,
};

// Export mock implementations for testing (available for both unit and integration tests)
#[cfg(any(test, feature = "test-mocks"))]
pub use system::{MockAudioSystem, MockFileSystem, MockSystemService};
