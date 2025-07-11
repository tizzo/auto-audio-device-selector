# Implementation Plan: macOS Audio Device Monitor with Priority-Based Switching

## Project Overview

This project creates a Rust-based macOS audio device monitor that automatically switches to preferred audio devices based on a weighted priority list. The system monitors for audio device changes and intelligently selects the highest-priority available device.

## Architecture: Hybrid Approach

After evaluating both approaches, I recommend a **hybrid solution** that combines the strengths of both:

```
┌─────────────────────────────────────────────────────────────┐
│                Audio Device Monitor                         │
├─────────────────────────────────────────────────────────────┤
│  Audio Monitoring: CoreAudio (AudioObjectAddPropertyListener)│
│  System Integration: core-foundation (CFRunLoop, CFNotificationCenter)│
│  Device Enumeration: cpal (cross-platform compatibility)   │
│  Device Control: CoreAudio (AudioObjectSetPropertyData)    │
└─────────────────────────────────────────────────────────────┘
```

## Dependencies

```toml
[dependencies]
# Audio-specific functionality
coreaudio-sys = "0.2"
cpal = "0.15"

# System integration
core-foundation = "0.9"
core-foundation-sys = "0.2"

# Configuration and utilities
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"

# macOS-specific
libc = "0.2"
```

## Core Components

### 1. Audio Device Monitor (CoreAudio)
```rust
struct AudioDeviceMonitor {
    property_listener: AudioObjectPropertyListenerProc,
    device_list_address: AudioObjectPropertyAddress,
    default_output_address: AudioObjectPropertyAddress,
    default_input_address: AudioObjectPropertyAddress,
}

impl AudioDeviceMonitor {
    fn register_listeners(&self) -> Result<(), Error>;
    fn handle_device_change(&self, property: AudioObjectPropertyAddress);
    fn get_current_devices(&self) -> Result<Vec<AudioDevice>, Error>;
}
```

### 2. System Integration (Core Foundation)
```rust
struct SystemIntegration {
    run_loop: CFRunLoop,
    notification_center: CFNotificationCenter,
    timer_source: CFRunLoopTimerRef,
}

impl SystemIntegration {
    fn start_event_loop(&self) -> Result<(), Error>;
    fn register_system_notifications(&self) -> Result<(), Error>;
    fn schedule_periodic_checks(&self, interval: Duration) -> Result<(), Error>;
}
```

### 3. Device Priority Manager
```rust
struct DevicePriorityManager {
    output_priorities: Vec<DeviceRule>,
    input_priorities: Vec<DeviceRule>,
    current_output: Option<AudioDeviceID>,
    current_input: Option<AudioDeviceID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceRule {
    name_pattern: String,
    match_type: MatchType,
    weight: u32,
    enabled: bool,
}
```

### 4. Device Controller (Hybrid)
```rust
struct DeviceController {
    cpal_host: cpal::Host,
    coreaudio_system: AudioObjectID,
}

impl DeviceController {
    fn enumerate_devices(&self) -> Result<Vec<AudioDevice>, Error>;
    fn set_default_output(&self, device_id: AudioDeviceID) -> Result<(), Error>;
    fn set_default_input(&self, device_id: AudioDeviceID) -> Result<(), Error>;
    fn get_device_info(&self, device_id: AudioDeviceID) -> Result<DeviceInfo, Error>;
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Set up Rust project structure
- [ ] Implement basic device enumeration using cpal
- [ ] Create configuration system with TOML
- [ ] Set up logging and error handling
- [ ] Implement basic CLI interface

**Deliverables:**
- Working device enumeration
- Configuration file parsing
- Basic project structure

### Phase 2: CoreAudio Integration (Week 2)
- [ ] Implement CoreAudio property listeners
- [ ] Add device change detection
- [ ] Integrate with Core Foundation event loop
- [ ] Handle device addition/removal events
- [ ] Implement audio-specific property monitoring

**Deliverables:**
- Real-time device change detection
- CoreAudio property listener system
- Event loop integration

### Phase 3: Priority Management (Week 3)
- [ ] Implement priority-based device selection
- [ ] Add device matching logic (name patterns, UIDs)
- [ ] Create device switching decision engine
- [ ] Implement fallback logic for unavailable devices
- [ ] Add separate input/output priority handling

**Deliverables:**
- Priority-based device switching
- Configuration-driven device selection
- Fallback and error handling

### Phase 4: Device Control (Week 4)
- [ ] Implement device switching via CoreAudio
- [ ] Add device state management
- [ ] Create background service architecture
- [ ] Implement daemon/service mode
- [ ] Add system startup integration

**Deliverables:**
- Working device switching
- Background service capability
- System integration

### Phase 5: Polish & Testing (Week 5)
- [ ] Add comprehensive error handling
- [ ] Implement detailed logging and debugging
- [ ] Create installation/uninstall scripts
- [ ] Add unit and integration tests
- [ ] Performance optimization

**Deliverables:**
- Production-ready application
- Installation package
- Documentation

## Configuration Format

```toml
# ~/.config/audio-device-monitor/config.toml

[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = true

[notifications]
show_device_changes = true
show_switching_actions = true

[[output_devices]]
name = "AirPods Pro"
weight = 100
match_type = "contains"
enabled = true

[[output_devices]]
name = "Studio Display"
weight = 80
match_type = "contains"
enabled = true

[[output_devices]]
name = "MacBook Pro Speakers"
weight = 10
match_type = "exact"
enabled = true

[[input_devices]]
name = "AirPods Pro"
weight = 100
match_type = "contains"
enabled = true

[[input_devices]]
name = "MacBook Pro Microphone"
weight = 10
match_type = "exact"
enabled = true
```

## Key Technical Decisions

1. **Hybrid Architecture**: Combines CoreAudio's audio-specific capabilities with Core Foundation's system integration
2. **Event-Driven Design**: Uses CoreAudio property listeners for immediate response to changes
3. **Fallback Strategy**: Includes periodic polling as backup for missed events
4. **Memory Safety**: Uses safe Core Foundation wrappers where possible
5. **Cross-Platform Compatibility**: cpal provides fallback enumeration capabilities

## Research Summary

### Available Rust Crates (Higher-level preferred)

**Primary Options:**
- **cpal**: Cross-platform audio I/O library with device enumeration capabilities, but lacks device change notifications (open issue #373)
- **coreaudio-rs**: High-level wrapper around CoreAudio, but limited to audio_unit module currently
- **coreaudio-sys**: Raw bindings to CoreAudio - necessary for device change notifications
- **core-foundation**: General-purpose macOS system integration library

### Key Findings from Community Research

- macOS lacks built-in audio device priority settings
- Existing solutions use HammerSpoon + SwitchAudioSource or third-party apps like Recadio
- Device change detection requires low-level CoreAudio APIs (`AudioObjectAddPropertyListener`)
- Current Rust audio libraries don't handle automatic device switching well
- IOKit approach is more general but CoreAudio is better for audio-specific tasks

## Advantages of This Approach

1. **Audio-Specific**: Direct access to audio device properties and events
2. **System Integration**: Proper macOS system service integration
3. **Reliable**: Multiple detection mechanisms prevent missed events
4. **Configurable**: User-friendly configuration system
5. **Maintainable**: Clear separation of concerns between components

## Next Steps

This hybrid approach provides the best balance of functionality, reliability, and maintainability for your macOS audio device monitoring needs. The plan leverages the strengths of both CoreAudio (audio-specific) and Core Foundation (system integration) while maintaining cross-platform compatibility through cpal.

## File Structure

```
audio-inputs/
├── Cargo.toml
├── README.md
├── CLAUDE.md
├── IMPLEMENTATION_PLAN.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config/
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── monitor.rs
│   │   ├── controller.rs
│   │   └── device.rs
│   ├── system/
│   │   ├── mod.rs
│   │   └── integration.rs
│   └── priority/
│       ├── mod.rs
│       └── manager.rs
├── tests/
│   ├── integration_tests.rs
│   └── unit_tests.rs
└── examples/
    ├── list_devices.rs
    └── test_notifications.rs
```