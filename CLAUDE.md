# CLAUDE.md - macOS Audio Device Monitor

## Project Overview

This is a Rust project that creates a macOS audio device monitor with priority-based automatic switching. The system monitors for audio device changes (connections, disconnections, default device changes) and automatically switches to the highest-priority available device based on user-configured preferences.

## Architecture

The project uses a hybrid approach combining:
- **CoreAudio**: For audio-specific device monitoring and control
- **Core Foundation**: For macOS system integration and event loops
- **cpal**: For cross-platform device enumeration and compatibility

## Key Components

1. **Audio Device Monitor**: Listens for audio device changes using CoreAudio property listeners
2. **Device Priority Manager**: Manages weighted priority lists for input/output devices
3. **Device Controller**: Handles device switching and enumeration
4. **System Integration**: Manages background service and system notifications

## Dependencies

```toml
[dependencies]
coreaudio-sys = "0.2"      # CoreAudio bindings
cpal = "0.15"              # Cross-platform audio I/O
core-foundation = "0.9"    # macOS system integration
serde = "1.0"              # Configuration serialization
toml = "0.8"               # Configuration format
tokio = "1.0"              # Async runtime
tracing = "0.1"            # Structured logging
clap = "4.0"               # CLI interface
anyhow = "1.0"             # Error handling
libc = "0.2"               # System calls
```

## Configuration

The application uses TOML configuration files located at `~/.config/audio-device-monitor/config.toml`:

```toml
[general]
check_interval_ms = 1000
log_level = "info"
daemon_mode = true

[[output_devices]]
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

## Common Commands

### Development
```bash
# Build the project
cargo build

# Run with logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Run specific example
cargo run --example list_devices
```

### Testing
```bash
# List all audio devices
cargo run -- --list-devices

# Test device monitoring
cargo run -- --test-monitor

# Run in daemon mode
cargo run -- --daemon

# Validate configuration
cargo run -- --check-config
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy

# Run all checks
cargo fmt && cargo clippy && cargo test
```

## Development Workflow

1. **Setup**: Use `cargo build` to compile dependencies
2. **Development**: Use `RUST_LOG=debug cargo run` for detailed logging
3. **Testing**: Use `cargo test` for unit tests, manual testing for device interactions
4. **Linting**: Run `cargo fmt && cargo clippy` before commits

## Project Structure

```
src/
├── main.rs              # Entry point and CLI
├── lib.rs               # Library exports
├── config/              # Configuration management
│   ├── mod.rs
│   └── types.rs
├── audio/               # Audio device management
│   ├── mod.rs
│   ├── monitor.rs       # Device change monitoring
│   ├── controller.rs    # Device control
│   └── device.rs        # Device types and utilities
├── system/              # System integration
│   ├── mod.rs
│   └── integration.rs   # Core Foundation integration
└── priority/            # Priority management
    ├── mod.rs
    └── manager.rs       # Priority-based device selection
```

## Key Features

- **Real-time device monitoring**: Immediate response to audio device changes
- **Priority-based switching**: Configurable weighted device preferences
- **Background service**: Runs as daemon for continuous monitoring
- **Configuration-driven**: User-friendly TOML configuration
- **Cross-platform compatibility**: Uses cpal for device enumeration
- **Safe memory management**: Uses Core Foundation safe wrappers

## Implementation Status

- [x] Research and architecture design
- [x] Implementation plan creation
- [ ] Phase 1: Foundation (device enumeration, configuration)
- [ ] Phase 2: CoreAudio integration (property listeners)
- [ ] Phase 3: Priority management (device selection logic)
- [ ] Phase 4: Device control (switching implementation)
- [ ] Phase 5: Polish and testing

## Technical Notes

### CoreAudio Integration
- Uses `AudioObjectAddPropertyListener` for device change notifications
- Monitors `kAudioHardwarePropertyDevices` for device list changes
- Monitors `kAudioHardwarePropertyDefaultOutputDevice` for default device changes

### Memory Safety
- Uses safe Core Foundation wrappers where possible
- Careful management of callback lifetimes
- Proper cleanup of listeners and resources

### Error Handling
- Uses `anyhow` for error propagation
- Comprehensive logging with `tracing`
- Graceful fallback mechanisms

## Platform Requirements

- macOS 10.7+ (targets macOS 10.7 by default)
- Rust 1.70+ (for latest dependency compatibility)
- Development: Xcode command line tools for system framework linking

## Security Considerations

- No privilege escalation required
- Uses user-space APIs only
- Configuration files stored in user directory
- No network access required

## Future Enhancements

- GUI configuration interface
- Advanced device matching rules
- Integration with other audio applications
- Statistics and usage reporting
- System tray integration
- Bluetooth device handling improvements

## Troubleshooting

### Common Issues

1. **Device detection not working**:
   - Check that CoreAudio property listeners are registered
   - Verify device permissions in System Preferences
   - Check logs for CoreAudio errors

2. **Device switching fails**:
   - Ensure device is available and not in use
   - Check for system audio restrictions
   - Verify device UIDs in configuration

3. **Background service not starting**:
   - Check daemon mode configuration
   - Verify LaunchAgent setup
   - Check system logs for service errors

### Debug Commands

```bash
# Enable verbose logging
RUST_LOG=trace cargo run

# List all devices with details
cargo run -- --list-devices --verbose

# Test configuration parsing
cargo run -- --check-config --verbose
```

## Contributing

1. Follow Rust formatting conventions (`cargo fmt`)
2. Ensure all tests pass (`cargo test`)
3. Run clippy for linting (`cargo clippy`)
4. Add tests for new functionality
5. Update documentation as needed

## License

This project is open source. Please respect macOS system APIs and user privacy when using or extending this code.