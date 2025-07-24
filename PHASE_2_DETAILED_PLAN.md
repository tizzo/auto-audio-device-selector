# Phase 2 Architecture Refactoring Implementation Plan

## Overview
Refactor tightly-coupled components to support dependency injection and mocking, enabling comprehensive testing of system-dependent code while maintaining backward compatibility.

## Current Architecture Analysis
- **Tightly Coupled**: DeviceController (CoreAudio system calls), Device enumeration (cpal/CoreAudio), File system operations
- **Mixed Dependencies**: Main service loop, Configuration loading, System signal handling
- **Pure Logic**: Already tested in Phase 1

## Target Architecture
Move from direct system dependencies to trait-based dependency injection, allowing mock implementations for testing while preserving production functionality.

## Implementation Steps

### Step 1: Extract Core System Interface Traits
Create foundational traits that abstract system dependencies:

**File: `src/system/traits.rs`**
```rust
// Audio system operations trait
pub trait AudioSystemInterface {
    fn enumerate_devices(&self) -> Result<Vec<AudioDevice>>;
    fn get_default_output_device(&self) -> Result<Option<AudioDevice>>;
    fn get_default_input_device(&self) -> Result<Option<AudioDevice>>;
    fn set_default_output_device(&self, device_id: &str) -> Result<()>;
    fn set_default_input_device(&self, device_id: &str) -> Result<()>;
    fn add_device_change_listener(&self, callback: Box<dyn Fn() + Send + Sync>) -> Result<()>;
}

// File system operations trait
pub trait FileSystemInterface {
    fn read_config_file(&self, path: &Path) -> Result<String>;
    fn write_config_file(&self, path: &Path, content: &str) -> Result<()>;
    fn config_file_exists(&self, path: &Path) -> bool;
    fn create_config_dir(&self, path: &Path) -> Result<()>;
}

// System service operations trait  
pub trait SystemServiceInterface {
    fn register_signal_handlers(&self) -> Result<()>;
    fn run_event_loop(&self) -> Result<()>;
    fn should_continue_running(&self) -> bool;
}
```

**Commit**: "Extract system interface traits for dependency injection"

### Step 2: Implement Production System Adapters
Create concrete implementations that wrap existing system calls:

**File: `src/system/adapters.rs`**
```rust
// Production CoreAudio implementation
pub struct CoreAudioSystem {
    // Existing CoreAudio integration code
}

impl AudioSystemInterface for CoreAudioSystem {
    // Move existing DeviceController methods here
    // Wrap cpal and CoreAudio calls
}

// Production file system implementation
pub struct StandardFileSystem;

impl FileSystemInterface for StandardFileSystem {
    // Move config loading/saving logic here
    // Wrap std::fs operations
}

// Production system service implementation
pub struct MacOSSystemService {
    // Signal handling and event loop code
}

impl SystemServiceInterface for MacOSSystemService {
    // Move daemon/service logic here
}
```

**Commit**: "Implement production system adapters with trait implementations"

### Step 3: Create Mock Implementations for Testing
Develop controllable mock implementations for comprehensive testing:

**File: `src/system/mocks.rs`** (or `tests/mocks/` for test-only)
```rust
// Mock audio system for testing
pub struct MockAudioSystem {
    pub devices: Arc<Mutex<Vec<AudioDevice>>>,
    pub default_output: Arc<Mutex<Option<AudioDevice>>>,
    pub default_input: Arc<Mutex<Option<AudioDevice>>>,
    pub device_change_callbacks: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
    pub set_device_calls: Arc<Mutex<Vec<(String, DeviceType)>>>,
}

impl MockAudioSystem {
    pub fn new() -> Self { /* ... */ }
    pub fn add_device(&self, device: AudioDevice) { /* ... */ }
    pub fn remove_device(&self, device_id: &str) { /* ... */ }
    pub fn trigger_device_change(&self) { /* ... */ }
    pub fn get_set_device_calls(&self) -> Vec<(String, DeviceType)> { /* ... */ }
}

impl AudioSystemInterface for MockAudioSystem {
    // Controllable implementations that return mock data
}

// Mock file system for testing
pub struct MockFileSystem {
    pub files: Arc<Mutex<HashMap<PathBuf, String>>>,
    pub read_calls: Arc<Mutex<Vec<PathBuf>>>,
    pub write_calls: Arc<Mutex<Vec<(PathBuf, String)>>>,
}

// Mock system service for testing
pub struct MockSystemService {
    pub should_run: Arc<AtomicBool>,
    pub signal_handler_registered: Arc<AtomicBool>,
    pub event_loop_calls: Arc<AtomicUsize>,
}
```

**Commit**: "Add mock implementations for system interfaces"

### Step 4: Refactor DeviceController with Dependency Injection
Modify DeviceController to accept trait objects instead of making direct system calls:

**File: `src/audio/controller.rs`**
```rust
pub struct DeviceController<A: AudioSystemInterface> {
    audio_system: A,
    priority_manager: DevicePriorityManager,
    notification_manager: NotificationManager,
    current_output: Option<AudioDevice>,
    current_input: Option<AudioDevice>,
}

impl<A: AudioSystemInterface> DeviceController<A> {
    pub fn new(
        audio_system: A,
        config: &Config,
    ) -> Self {
        Self {
            audio_system,
            priority_manager: DevicePriorityManager::new(config),
            notification_manager: NotificationManager::new(config),
            current_output: None,
            current_input: None,
        }
    }

    pub fn start_monitoring(&mut self) -> Result<()> {
        // Use self.audio_system instead of direct CoreAudio calls
        let callback = /* ... */;
        self.audio_system.add_device_change_listener(callback)?;
        Ok(())
    }

    pub fn handle_device_change(&mut self) -> Result<()> {
        let devices = self.audio_system.enumerate_devices()?;
        // Rest of the logic remains the same
    }

    // Other methods updated to use self.audio_system
}

// Convenience constructor for production use
impl DeviceController<CoreAudioSystem> {
    pub fn new_production(config: &Config) -> Self {
        Self::new(CoreAudioSystem::new(), config)
    }
}
```

**Commit**: "Refactor DeviceController with dependency injection"

### Step 5: Refactor Configuration System with File System Abstraction
Abstract file system operations for configuration management:

**File: `src/config/loader.rs`**
```rust
pub struct ConfigLoader<F: FileSystemInterface> {
    file_system: F,
    config_path: PathBuf,
}

impl<F: FileSystemInterface> ConfigLoader<F> {
    pub fn new(file_system: F, config_path: PathBuf) -> Self {
        Self { file_system, config_path }
    }

    pub fn load_config(&self) -> Result<Config> {
        if !self.file_system.config_file_exists(&self.config_path) {
            return Ok(Config::default());
        }
        
        let content = self.file_system.read_config_file(&self.config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        let content = toml::to_string_pretty(config)?;
        if let Some(parent) = self.config_path.parent() {
            self.file_system.create_config_dir(parent)?;
        }
        self.file_system.write_config_file(&self.config_path, &content)?;
        Ok(())
    }

    pub fn reload_on_change(&self) -> Result<Config> {
        // File watching logic using file_system trait
        self.load_config()
    }
}

// Production constructor
impl ConfigLoader<StandardFileSystem> {
    pub fn new_production(config_path: PathBuf) -> Self {
        Self::new(StandardFileSystem, config_path)
    }
}
```

**Commit**: "Abstract file system operations in configuration system"

### Step 6: Refactor Main Service with System Service Abstraction
Create a testable main service that doesn't directly interact with system APIs:

**File: `src/service/mod.rs`**
```rust
pub struct AudioDeviceService<A: AudioSystemInterface, F: FileSystemInterface, S: SystemServiceInterface> {
    device_controller: DeviceController<A>,
    config_loader: ConfigLoader<F>,
    system_service: S,
    config: Config,
}

impl<A: AudioSystemInterface, F: FileSystemInterface, S: SystemServiceInterface> 
    AudioDeviceService<A, F, S> {
    
    pub fn new(
        audio_system: A,
        file_system: F,
        system_service: S,
        config_path: PathBuf,
    ) -> Result<Self> {
        let config_loader = ConfigLoader::new(file_system, config_path);
        let config = config_loader.load_config()?;
        let device_controller = DeviceController::new(audio_system, &config);
        
        Ok(Self {
            device_controller,
            config_loader,
            system_service,
            config,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.system_service.register_signal_handlers()?;
        self.device_controller.start_monitoring()?;
        
        while self.system_service.should_continue_running() {
            self.system_service.run_event_loop()?;
        }
        
        Ok(())
    }

    pub fn reload_config(&mut self) -> Result<()> {
        self.config = self.config_loader.load_config()?;
        // Reinitialize components with new config
        Ok(())
    }
}

// Production constructor
impl AudioDeviceService<CoreAudioSystem, StandardFileSystem, MacOSSystemService> {
    pub fn new_production(config_path: PathBuf) -> Result<Self> {
        Self::new(
            CoreAudioSystem::new(),
            StandardFileSystem,
            MacOSSystemService::new(),
            config_path,
        )
    }
}
```

**Commit**: "Create testable main service with dependency injection"

### Step 7: Update Main Binary and Library Exports
Ensure backward compatibility while exposing new testable interfaces:

**File: `src/main.rs`**
```rust
// Update to use new production constructor
fn main() -> Result<()> {
    let config_path = get_config_path();
    let mut service = AudioDeviceService::new_production(config_path)?;
    service.start()
}
```

**File: `src/lib.rs`**
```rust
// Export traits and mock implementations for testing
pub mod system {
    pub mod traits;
    pub mod adapters;
    
    #[cfg(test)]
    pub mod mocks;
}

// Re-export for backward compatibility
pub use audio::DeviceController;
pub use config::{Config, ConfigLoader};
pub use service::AudioDeviceService;

// Export new testable interfaces
pub use system::traits::*;

#[cfg(test)]
pub use system::mocks::*;
```

**Commit**: "Update main binary and library exports for new architecture"

### Step 8: Create Integration Tests with Mock Systems
Develop comprehensive integration tests using the new mock systems:

**File: `tests/integration_with_mocks.rs`**
```rust
// Test complete device monitoring workflow
#[test]
fn test_device_monitoring_full_workflow() {
    let mock_audio = MockAudioSystem::new();
    let mock_fs = MockFileSystem::new();
    let mock_service = MockSystemService::new();
    
    // Set up initial state
    mock_audio.add_device(AudioDeviceBuilder::new()
        .name("Initial Device").output().build());
    
    let config = Config::default();
    mock_fs.add_file(config_path(), toml::to_string(&config).unwrap());
    
    let mut service = AudioDeviceService::new(
        mock_audio.clone(), mock_fs, mock_service.clone(), config_path()
    ).unwrap();
    
    // Simulate service startup
    service.start_monitoring().unwrap();
    
    // Simulate device changes
    mock_audio.add_device(AudioDeviceBuilder::new()
        .name("High Priority Device").output().build());
    mock_audio.trigger_device_change();
    
    // Verify system interactions
    let set_calls = mock_audio.get_set_device_calls();
    assert_eq!(set_calls.len(), 1);
    assert_eq!(set_calls[0].0, "High Priority Device");
}

// Test configuration reloading
#[test]
fn test_config_reload_integration() { /* ... */ }

// Test error handling across system boundaries
#[test]
fn test_system_error_handling() { /* ... */ }
```

**Commit**: "Add integration tests using mock system implementations"

## Success Criteria
- [ ] All system dependencies abstracted behind traits
- [ ] Mock implementations allow complete testing isolation
- [ ] Production code maintains existing functionality
- [ ] Integration tests cover system interaction workflows
- [ ] Backward compatibility preserved for existing API
- [ ] Performance impact minimal (<5% overhead)

## Testing Strategy
1. **Unit Tests**: Each trait implementation tested in isolation
2. **Integration Tests**: Mock systems used to test component interactions
3. **Contract Tests**: Verify production and mock implementations behave consistently
4. **Performance Tests**: Ensure dependency injection doesn't impact performance

## Risk Mitigation
- Create traits incrementally, one system at a time
- Maintain parallel implementations during transition
- Extensive testing at each step to prevent regressions
- Feature flags to switch between old/new implementations if needed

## Deliverables
8 commits implementing architecture refactoring, establishing foundation for comprehensive system testing and improved maintainability.

## Next Steps (Phase 3 Preview)
After Phase 2 completion:
- End-to-end integration tests with real system calls
- Performance benchmarking and optimization
- CI/CD pipeline integration
- Property-based testing for edge cases