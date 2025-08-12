# CLAUDE.md - macOS Audio Device Monitor

## Project Overview

This is a Rust project that creates a macOS audio device monitor with priority-based automatic switching. The system monitors for audio device changes (connections, disconnections, default device changes) and automatically switches to the highest-priority available device based on user-configured preferences.

## Architecture

The project uses a dependency injection architecture with clean interfaces:
- **CoreAudio**: For audio-specific device monitoring and control
- **Core Foundation**: For macOS system integration and event loops  
- **Dependency Injection**: Clean abstractions with trait-based interfaces
- **Mocking Support**: Comprehensive test doubles for all system interactions

## Key Components

1. **DeviceControllerV2**: Handles device switching and enumeration with dependency injection
2. **Device Priority Manager**: Manages weighted priority lists for input/output devices
3. **Audio System Interface**: Abstracts CoreAudio operations for testability
4. **Configuration Loader**: File system abstracted configuration management
5. **Notification Manager**: System notifications for device events
6. **Service Layer**: Background service orchestration with dependency injection
7. **Mock System**: Comprehensive test doubles for all external dependencies

## Dependencies

```toml
[dependencies]
coreaudio-sys = "0.2"      # CoreAudio bindings
core-foundation = "0.9"    # macOS system integration
serde = "1.0"              # Configuration serialization
toml = "0.8"               # Configuration format
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

[notifications]
show_device_availability = true
show_switching_actions = true

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
3. **Testing**: Use `cargo test` for comprehensive unit and integration tests
4. **Linting**: Run `cargo fmt && cargo clippy` before commits
5. **Documentation**: Update `README.md` and inline comments for clarity
6. **Version Control**: Use Git for source control, with feature branches for new functionality
7. **Regular Commits**: Commit frequently with clear messages, always say why we made the change and not just what the change is. Resolve all lints and tests before committing. Do not include claude as an author or in the commit message.
8. **Pull Requests**: Submit PRs for review, ensuring all tests pass and code is formatted

## Project Structure

```
src/
├── main.rs              # Entry point and CLI
├── lib.rs               # Library exports
├── config/              # Configuration management
│   ├── mod.rs
│   ├── types.rs         # Configuration data structures
│   └── loader.rs        # File system abstracted configuration loading
├── audio/               # Audio device management
│   ├── mod.rs
│   ├── device.rs        # Device types and utilities
│   ├── controller.rs    # Legacy CoreAudio controller
│   ├── controller_v2.rs # Dependency injected controller
│   ├── monitor.rs       # Device change monitoring
│   └── listener.rs      # CoreAudio property listeners
├── system/              # System integration abstractions
│   ├── mod.rs
│   ├── traits.rs        # Interface definitions
│   ├── adapters.rs      # Production implementations
│   ├── mocks.rs         # Test doubles
│   └── integration.rs   # Core Foundation integration
├── priority/            # Priority management
│   ├── mod.rs
│   └── manager.rs       # Priority-based device selection
├── notifications/       # System notifications
│   └── mod.rs           # Notification management
├── service/             # Service layer
│   ├── mod.rs
│   ├── service_v2.rs    # Dependency injected service
│   ├── daemon.rs        # Background service management
│   └── signals.rs       # Signal handling
└── logging/             # Structured logging
    └── mod.rs           # Logging configuration
```

## Test Structure

```
tests/
├── config_tests.rs                      # Configuration parsing tests
├── config_loader_integration_tests.rs   # File system integration tests
├── device_controller_integration_tests.rs # Device controller tests
├── device_matching_tests.rs             # Device matching logic tests
├── integration_dependency_injection_tests.rs # DI architecture tests
├── integration_pure_logic_tests.rs      # Pure logic integration tests
├── notification_manager_tests.rs        # Notification system tests
├── priority_manager_tests.rs            # Priority management tests
└── test_utils/                          # Test utilities
    ├── mod.rs
    └── builders.rs                      # Test data builders
```

## Key Features

- **Real-time device monitoring**: Immediate response to audio device changes
- **Priority-based switching**: Configurable weighted device preferences
- **Background service**: Runs as daemon for continuous monitoring
- **Configuration-driven**: User-friendly TOML configuration
- **Dependency injection**: Clean architecture with testable interfaces
- **Comprehensive testing**: Unit, integration, and mock-based tests
- **Safe memory management**: Uses Core Foundation safe wrappers
- **Structured logging**: Comprehensive tracing support

## Implementation Status

- [x] **Phase 1: Foundation** - Device enumeration, configuration, priority management
- [x] **Phase 2: Dependency Injection Architecture** - Clean interfaces, comprehensive testing
- [x] **CoreAudio Integration** - Device monitoring and control
- [x] **Configuration System** - File system abstracted TOML configuration
- [x] **Priority Management** - Weighted device selection with matching rules
- [x] **Device Control** - Automatic switching implementation
- [x] **Notification System** - User feedback for device events
- [x] **Service Layer** - Background service with dependency injection
- [x] **Comprehensive Testing** - Unit, integration, and mock-based test suites
- [ ] **Production Deployment** - LaunchAgent setup and production hardening
- [ ] **GUI Configuration** - User-friendly configuration interface

## Technical Notes

### Architecture Design
- Uses trait-based dependency injection for clean separation of concerns
- All external dependencies (file system, audio system, system services) are abstracted
- Comprehensive mock implementations enable fast, reliable testing
- State management follows explicit patterns with clear ownership

### Device Management
- `DeviceControllerV2` handles device enumeration, switching, and state management
- Supports priority-based automatic switching when devices connect/disconnect
- Maintains internal state synchronized with system defaults
- Handles both individual device types (Input/Output) and combination devices

### CoreAudio Integration
- Uses `AudioObjectAddPropertyListener` for device change notifications
- Monitors `kAudioHardwarePropertyDevices` for device list changes
- Monitors `kAudioHardwarePropertyDefaultOutputDevice` for default device changes
- Safe wrappers around CoreAudio APIs with proper error handling

### Configuration Management
- File system abstracted configuration loading with `ConfigLoader<F: FileSystemInterface>`
- Supports hot reloading and validation
- Device matching with multiple match types: exact, contains, starts_with, ends_with
- Weighted priority system with enable/disable per device

### Testing Strategy
- **Unit Tests**: Pure logic components tested in isolation
- **Integration Tests**: Cross-component functionality with mocks
- **Mock System**: Complete test doubles for audio system, file system, and system services
- **Builder Pattern**: Fluent test data construction utilities

### Memory Safety
- Uses safe Core Foundation wrappers where possible
- Careful management of callback lifetimes
- Proper cleanup of listeners and resources
- No unsafe code in application logic

### Error Handling
- Uses `anyhow` for error propagation with context
- Comprehensive logging with `tracing` structured logging
- Graceful fallback mechanisms for system failures
- Non-blocking error recovery for device operations

## Platform Requirements

- macOS 10.7+ (targets macOS 10.7 by default)
- Rust 1.70+ (for latest dependency compatibility)
- Development: Xcode command line tools for system framework linking

## Security Considerations

- No privilege escalation required
- Uses user-space APIs only
- Configuration files stored in user directory with proper permissions
- No network access required
- Input validation for all configuration parsing

## Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test device_controller_tests
cargo test config_tests
cargo test priority_manager_tests

# Run with output for debugging
cargo test -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test
```

### Test Coverage
- **Configuration**: Parsing, validation, file system operations
- **Device Management**: Enumeration, switching, state management
- **Priority System**: Device matching, weight-based selection
- **Notifications**: System notification delivery
- **Service Layer**: Background service orchestration
- **Integration**: Cross-component workflows with dependency injection

## Troubleshooting

### Common Issues

1. **Device detection not working**:
   - Check that CoreAudio property listeners are registered
   - Verify device permissions in System Preferences → Security & Privacy → Microphone
   - Check logs: `RUST_LOG=debug cargo run`

2. **Device switching fails**:
   - Ensure device is available and not in use by another application
   - Check for system audio restrictions
   - Verify device names match configuration exactly (case-sensitive)

3. **Configuration not loading**:
   - Check file exists at `~/.config/audio-device-monitor/config.toml`
   - Validate TOML syntax: `cargo run -- --check-config`
   - Check file permissions and directory creation

4. **Tests failing**:
   - Ensure no other audio applications are interfering
   - Run tests individually to isolate issues: `cargo test test_name`
   - Check for dependency version conflicts: `cargo update`

### Debug Commands

```bash
# Enable verbose logging
RUST_LOG=trace cargo run

# List all devices with details
cargo run -- --list-devices

# Test configuration parsing
cargo run -- --check-config

# Run specific test with logging
RUST_LOG=debug cargo test device_controller_tests::test_device_switching -- --nocapture
```

## Contributing

1. Follow Rust formatting conventions (`cargo fmt`)
2. Ensure all tests pass (`cargo test`)
3. Run clippy for linting (`cargo clippy`)
4. Add tests for new functionality (unit + integration)
5. Update documentation as needed
6. Maintain dependency injection patterns for testability

## License

This project is open source. Please respect macOS system APIs and user privacy when using or extending this code.
