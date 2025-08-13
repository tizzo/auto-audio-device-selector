# Audio Device Monitor

A Rust-based macOS audio device monitor that automatically switches to preferred audio devices based on a weighted priority list. The system monitors for audio device changes (connections, disconnections, default device changes) and intelligently selects the highest-priority available device.

## Features

- **Real-time Device Monitoring**: Instant detection of audio device changes using CoreAudio property listeners
- **Automatic Device Switching**: Intelligent switching to highest-priority available devices based on configurable rules
- **Priority-based Device Selection**: Configurable weighted priority system with multiple matching modes
- **Comprehensive Device Support**: Monitors both input and output devices (microphones, speakers, headphones, etc.)
- **Service Management**: Install as macOS LaunchAgent for automatic startup and background operation
- **Enhanced Logging**: Daily log rotation, JSON output, console and file logging with cleanup utilities
- **Manual Device Control**: Command-line interface for manual device switching and testing
- **Flexible Configuration**: TOML-based configuration with hot-reload support via SIGHUP signal
- **Notification System**: macOS Notification Center integration for device changes and switching events
- **Graceful Shutdown**: Proper signal handling for clean service lifecycle management
- **Dependency Injection Architecture**: Clean, testable architecture with comprehensive mock support
- **Comprehensive Testing**: Unit, integration, and mock-based test suites with 100% test isolation

## Requirements

- macOS 10.7 or later
- Rust 1.70 or later
- Xcode Command Line Tools (for system framework linking)

## Installation

### From Source

```bash
git clone https://github.com/yourusername/audio-device-monitor
cd audio-device-monitor
cargo build --release
```

The binary will be available at `target/release/audio-device-monitor`.

### Install System-wide

```bash
cargo install --path .
```

## Quick Start

1. **List available audio devices:**
   ```bash
   audio-device-monitor list-devices
   ```

2. **Test real-time monitoring:**
   ```bash
   audio-device-monitor test-monitor
   ```

3. **Install as system service (recommended):**
   ```bash
   audio-device-monitor install-service
   # Service will start automatically on login
   ```

4. **Or run manually in daemon mode:**
   ```bash
   audio-device-monitor daemon
   ```

5. **Check configuration:**
   ```bash
   audio-device-monitor check-config
   ```

6. **Manual device switching:**
   ```bash
   # Switch output device
   audio-device-monitor switch --device "AirPods Pro"
   
   # Switch input device
   audio-device-monitor switch --device "Blue Yeti" --input
   ```

## Configuration

The application uses a TOML configuration file located at `~/.config/audio-device-monitor/config.toml`. The configuration file is automatically created with sensible defaults on first run.

### Configuration File Structure

```toml
[general]
# Fallback polling interval in milliseconds (used alongside real-time monitoring)
check_interval_ms = 1000

# Logging level: "trace", "debug", "info", "warn", "error"
log_level = "info"

# Whether to run in daemon mode by default
daemon_mode = false

[notifications]
# Show notifications when devices are added/removed
show_device_availability = true

# Show notifications when automatic switching occurs
show_switching_actions = true

# Output device priority rules (highest weight wins)
[[output_devices]]
name = "AirPods"
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

# Input device priority rules (highest weight wins)
[[input_devices]]
name = "AirPods"
weight = 100
match_type = "contains"
enabled = true

[[input_devices]]
name = "MacBook Pro Microphone"
weight = 10
match_type = "exact"
enabled = true
```

### Device Rules

Each device rule supports the following fields:

- **`name`** (required): The device name or pattern to match
- **`weight`** (required): Priority weight (higher numbers = higher priority)
- **`match_type`** (required): How to match the device name:
  - `"exact"` - Exact string match
  - `"contains"` - Device name contains this string
  - `"starts_with"` - Device name starts with this string
  - `"ends_with"` - Device name ends with this string
- **`enabled`** (required): Whether this rule is active

### Priority System

The priority system works as follows:

1. **Higher Weight Wins**: Devices with higher `weight` values take precedence
2. **Separate Input/Output**: Input and output devices are managed independently
3. **Availability Check**: Only available (connected) devices are considered
4. **Fallback Chain**: If the highest priority device is unavailable, the system falls back to the next highest priority available device

## Usage

### Command Line Interface

```bash
audio-device-monitor [OPTIONS] [COMMAND]
```

#### Options

- `-v, --verbose` - Enable verbose logging
- `-c, --config <CONFIG>` - Specify custom configuration file path
- `--json-logs` - Enable JSON logging format (for log aggregation)
- `--no-file-logs` - Disable file logging (console only)
- `--log-dir <LOG_DIR>` - Custom log directory
- `-h, --help` - Show help information
- `-V, --version` - Show version information

#### Commands

- **`list-devices`** - List all available audio devices
  ```bash
  audio-device-monitor list-devices [--verbose]
  ```

- **`switch`** - Manually switch to a specific device
  ```bash
  audio-device-monitor switch --device "AirPods Pro"
  audio-device-monitor switch --device "Blue Yeti" --input
  ```

- **`show-default`** - Show current default devices
  ```bash
  audio-device-monitor show-default
  ```

- **`test-monitor`** - Test device monitoring (shows real-time changes)
  ```bash
  audio-device-monitor test-monitor
  ```

- **`daemon`** - Run in daemon mode (continuous monitoring)
  ```bash
  audio-device-monitor daemon
  ```

- **`install-service`** - Install as macOS LaunchAgent
  ```bash
  audio-device-monitor install-service
  ```

- **`uninstall-service`** - Uninstall the system service
  ```bash
  audio-device-monitor uninstall-service
  ```

- **`check-config`** - Validate configuration file
  ```bash
  audio-device-monitor check-config
  ```

- **`cleanup-logs`** - Clean up old log files
  ```bash
  audio-device-monitor cleanup-logs --keep-days 30
  ```

- **`test-notification`** - Test notification system
  ```bash
  audio-device-monitor test-notification
  ```

- **`device-info`** - Show detailed information about a specific device
  ```bash
  audio-device-monitor device-info --device "AirPods Pro"
  ```

- **`check-device`** - Check if a device is currently available
  ```bash
  audio-device-monitor check-device --device "Blue Yeti"
  ```

- **`status`** - Show current service status and configuration
  ```bash
  audio-device-monitor status
  ```

- **`show-current`** - Show current active/selected devices
  ```bash
  audio-device-monitor show-current
  ```

## Service Management

The application supports installation as a macOS LaunchAgent for automatic startup and background operation.

### Installing the Service

```bash
# Install as system service
audio-device-monitor install-service

# The service will:
# - Start automatically when you log in
# - Run in the background continuously
# - Restart automatically if it crashes
# - Write logs to ~/.local/share/audio-device-monitor/logs/
```

### Managing the Service

```bash
# Check service status
launchctl list | grep audiodevicemonitor

# Start the service manually
launchctl load ~/Library/LaunchAgents/com.audiodevicemonitor.daemon.plist

# Stop the service
launchctl unload ~/Library/LaunchAgents/com.audiodevicemonitor.daemon.plist

# View service logs
tail -f ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*

# Uninstall the service
audio-device-monitor uninstall-service
```

### Hot Configuration Reload

The service supports hot reloading of configuration without restart:

```bash
# Find the service process ID
ps aux | grep audio-device-monitor

# Send reload signal (replace PID with actual process ID)
kill -HUP <PID>

# Or if installed as a service
launchctl kill -HUP system/com.audiodevicemonitor.daemon
```

**What gets reloaded:**
- Device priority weights and rules
- Notification preferences
- General configuration settings
- All matching patterns and device rules

## Notification System

The application integrates with macOS Notification Center to provide real-time feedback about device changes and switching events.

### Notification Types

1. **Device Connected** - Shows when audio devices come online
2. **Device Disconnected** - Shows when audio devices go offline
3. **Device Switched** - Shows when automatic switching occurs
4. **Switch Failed** - Shows when device switching fails

### Notification Configuration

Configure notifications in your TOML configuration file:

```toml
[notifications]
# Show notifications when devices are added/removed
show_device_availability = true

# Show notifications when automatic switching occurs
show_switching_actions = true
```

### Testing Notifications

```bash
# Test the notification system
audio-device-monitor test-notification

# Run daemon with notifications enabled
audio-device-monitor daemon
# (Try plugging/unplugging devices to see notifications)
```

## Development

### Building from Source

```bash
git clone https://github.com/yourusername/audio-device-monitor
cd audio-device-monitor
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output for debugging
cargo test -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test
```

### Development Commands

```bash
# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Run all checks
cargo fmt && cargo clippy && cargo test

# Build optimized release
cargo build --release
```

## Architecture

The application uses a modern dependency injection architecture with clean interfaces:

- **CoreAudio**: For audio-specific device monitoring and control
- **Core Foundation**: For macOS system integration and event loops  
- **Dependency Injection**: Clean abstractions with trait-based interfaces
- **Mocking Support**: Comprehensive test doubles for all system interactions

### Key Components

1. **DeviceControllerV2**: Handles device switching and enumeration with dependency injection
2. **Device Priority Manager**: Manages weighted priority lists for input/output devices
3. **Audio System Interface**: Abstracts CoreAudio operations for testability
4. **Configuration Loader**: File system abstracted configuration management
5. **Notification Manager**: System notifications for device events
6. **Service Layer**: Background service orchestration with dependency injection
7. **Mock System**: Comprehensive test doubles for all external dependencies

### Testing Strategy

- **Unit Tests**: Pure logic components tested in isolation
- **Integration Tests**: Cross-component functionality with mocks
- **Mock System**: Complete test doubles for audio system, file system, and system services
- **Builder Pattern**: Fluent test data construction utilities

## Project Status

### ✅ Completed Features

- **Phase 1: Foundation** - Device enumeration, configuration, priority management
- **Phase 2: Dependency Injection Architecture** - Clean interfaces, comprehensive testing
- **CoreAudio Integration** - Device monitoring and control
- **Configuration System** - File system abstracted TOML configuration
- **Priority Management** - Weighted device selection with matching rules
- **Device Control** - Automatic switching implementation
- **Notification System** - User feedback for device events
- **Service Layer** - Background service with dependency injection
- **Comprehensive Testing** - Unit, integration, and mock-based test suites

### 🚧 Current Capabilities

- **Automatic Device Switching**: ✅ Fully functional with real-time CoreAudio monitoring
- **Service Management**: ✅ Install/uninstall as macOS LaunchAgent with auto-startup
- **Enhanced Logging**: ✅ Daily rotation, JSON output, file/console logging, automated cleanup
- **Manual Device Control**: ✅ Command-line switching for testing and manual control
- **Priority-based Selection**: ✅ Configurable weight system with multiple matching modes
- **Configuration Hot-reload**: ✅ Dynamic config changes via SIGHUP signal without service restart
- **Notification System**: ✅ macOS Notification Center integration for device events
- **Library API**: ✅ Comprehensive CLI commands for device management and inspection

### 📋 Planned Features

- **Production Deployment** - LaunchAgent setup and production hardening
- **GUI Configuration** - User-friendly configuration interface

## Technical Details

### System Requirements

- Uses CoreAudio APIs for low-level audio device management
- Requires macOS system permissions for audio device access
- No privileged access required (runs in user space)
- Minimal system resource usage when idle

### Performance

- Event-driven architecture for immediate response to device changes
- Efficient CoreAudio property listeners with minimal CPU overhead
- Background monitoring with negligible battery impact
- Fast device enumeration and switching decisions

### Security

- No network access required
- Configuration files stored in user directory
- Uses only public macOS APIs
- No system-level privileges needed
- Input validation for all configuration parsing

## Troubleshooting

### Common Issues

1. **Device detection not working**
   - Check that CoreAudio property listeners are registered
   - Verify device permissions in System Preferences → Security & Privacy → Microphone
   - Check logs: `RUST_LOG=debug cargo run`

2. **Device switching fails**
   - Ensure device is available and not in use by another application
   - Check for system audio restrictions
   - Verify device names match configuration exactly (case-sensitive)

3. **Configuration not loading**
   - Check file exists at `~/.config/audio-device-monitor/config.toml`
   - Validate TOML syntax: `cargo run -- --check-config`
   - Check file permissions and directory creation

4. **Tests failing**
   - Ensure no other audio applications are interfering
   - Run tests individually to isolate issues: `cargo test test_name`
   - Check for dependency version conflicts: `cargo update`

### Debug Commands

```bash
# Enable verbose logging
RUST_LOG=trace cargo run

# List all devices with details
cargo run -- list-devices --verbose

# Test configuration parsing
cargo run -- check-config

# Check device availability
cargo run -- check-device --device "Your Device Name"

# Show service status
cargo run -- status

# Show current devices
cargo run -- show-current
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

## Acknowledgments

- Built with [coreaudio-sys](https://github.com/RustAudio/coreaudio-sys) for CoreAudio bindings
- Uses [core-foundation](https://github.com/servo/core-foundation-rs) for macOS system integration
- Configuration management with [serde](https://serde.rs/) and [toml](https://github.com/toml-rs/toml)
- Structured logging with [tracing](https://tracing.rs/)