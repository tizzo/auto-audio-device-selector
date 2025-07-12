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
- **Cross-platform Architecture**: Built with cpal for future extensibility while leveraging macOS-specific CoreAudio APIs

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
show_device_changes = true

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
  - `"regex"` - Regular expression match (planned)
- **`enabled`** (required): Whether this rule is active

### Priority System

The priority system works as follows:

1. **Higher Weight Wins**: Devices with higher `weight` values take precedence
2. **Separate Input/Output**: Input and output devices are managed independently
3. **Availability Check**: Only available (connected) devices are considered
4. **Fallback Chain**: If the highest priority device is unavailable, the system falls back to the next highest priority available device

### Example Scenarios

#### Scenario 1: Basic Setup
```toml
[[output_devices]]
name = "AirPods Pro"
weight = 100
match_type = "exact"
enabled = true

[[output_devices]]
name = "MacBook Pro Speakers"
weight = 10
match_type = "exact"
enabled = true
```

**Behavior**: Always prefer "AirPods Pro" when available, fall back to "MacBook Pro Speakers"

#### Scenario 2: Multiple Similar Devices
```toml
[[output_devices]]
name = "AirPods"
weight = 100
match_type = "contains"
enabled = true

[[output_devices]]
name = "Studio"
weight = 80
match_type = "contains"
enabled = true
```

**Behavior**: Any device containing "AirPods" gets priority 100, any device containing "Studio" gets priority 80

#### Scenario 3: Work vs Personal Setup
```toml
[[output_devices]]
name = "Sony WH-1000XM4"  # Personal headphones
weight = 100
match_type = "exact"
enabled = true

[[output_devices]]
name = "Dell Monitor"     # Work monitor
weight = 90
match_type = "contains"
enabled = true

[[output_devices]]
name = "MacBook Pro Speakers"  # Fallback
weight = 10
match_type = "exact"
enabled = true
```

**Behavior**: Personal headphones first, then work monitor, then laptop speakers

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

- **`service`** - Run as background service with enhanced signal handling
  ```bash
  audio-device-monitor service
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

### Service Configuration

The installed service uses the following configuration:

- **Executable**: Current binary location (copies path at install time)
- **Run Mode**: Daemon mode with real-time monitoring
- **Auto-restart**: Enabled (service restarts if it crashes)
- **Log Output**: `/tmp/audio-device-monitor.log` and `/tmp/audio-device-monitor.err`
- **Environment**: `RUST_LOG=info` for standard logging level

### Enhanced Service Mode

For advanced users who want more control over the service:

```bash
# Run with enhanced signal handling
audio-device-monitor service

# Features:
# - Graceful shutdown on SIGTERM/SIGINT
# - Configuration hot-reload on SIGHUP
# - Better error handling and recovery
# - Structured logging with metadata
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

**The reload process:**
1. Receives SIGHUP signal
2. Stops current audio monitoring
3. Reloads configuration from disk
4. Restarts monitoring with new settings
5. Logs the successful reload

## Notification System

The application integrates with macOS Notification Center to provide real-time feedback about device changes and switching events.

### Notification Types

1. **Device Connected** - Shows when audio devices come online
   - Displays device name and type (🎤 Input, 🔊 Output, 🎧 Input/Output)
   - Helps track device availability

2. **Device Disconnected** - Shows when audio devices go offline
   - Alerts when preferred devices become unavailable
   - Useful for troubleshooting connectivity issues

3. **Device Switched** - Shows when automatic switching occurs
   - Indicates which device was selected and why
   - Includes switching reason (higher priority, previous unavailable)

4. **Switch Failed** - Shows when device switching fails
   - Displays error information for troubleshooting
   - Helps identify device conflicts or permission issues

### Notification Configuration

Configure notifications in your TOML configuration file:

```toml
[notifications]
# Show notifications when devices are added/removed
show_device_changes = true

# Show notifications when automatic switching occurs
show_switching_actions = true
```

### Testing Notifications

```bash
# Test the notification system
audio-device-monitor test-notification

# Run with notifications enabled
audio-device-monitor service
# (Try plugging/unplugging devices to see notifications)
```

### Notification Examples

- **"🎤 AirPods Pro is now available"** - Device connected
- **"🔊 USB Speakers is no longer available"** - Device disconnected  
- **"🔊 Output switched to AirPods Pro (higher priority)"** - Automatic switching
- **"Failed to switch to Blue Yeti: Device in use"** - Switch failure

## Logging System

The application features a comprehensive logging system with multiple output options and automatic management.

### Log Locations

- **Default Directory**: `~/.local/share/audio-device-monitor/logs/`
- **File Format**: `audio-device-monitor.log.YYYY-MM-DD` (daily rotation)
- **Service Logs**: `/tmp/audio-device-monitor.log` (when running as service)

### Logging Options

```bash
# Enable debug logging
audio-device-monitor --verbose list-devices

# JSON format for log aggregation
audio-device-monitor --json-logs daemon

# Console-only logging (no files)
audio-device-monitor --no-file-logs test-monitor

# Custom log directory
audio-device-monitor --log-dir /var/log/audio-monitor daemon

# Combination of options
audio-device-monitor --verbose --json-logs --log-dir ~/logs service
```

### Log Management

```bash
# Clean up old logs (keep last 30 days)
audio-device-monitor cleanup-logs --keep-days 30

# View recent logs
tail -f ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*

# View logs in JSON format
jq '.' ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*
```

### Examples

#### Complete Service Setup:
```bash
# 1. Test the configuration
audio-device-monitor check-config

# 2. Test device detection
audio-device-monitor list-devices

# 3. Test real-time monitoring
audio-device-monitor test-monitor
# (Try plugging/unplugging devices to see live updates)

# 4. Install as service
audio-device-monitor install-service

# 5. Verify service is running
launchctl list | grep audiodevicemonitor
```

#### Manual Operation with Enhanced Logging:
```bash
# Run with detailed JSON logs
audio-device-monitor --verbose --json-logs service

# Run with console-only debug output
audio-device-monitor --verbose --no-file-logs daemon

# Run with custom configuration and logs
audio-device-monitor -c ~/my-config.toml --log-dir ~/audio-logs daemon
```

#### Device Testing and Control:
```bash
# See what devices are available
audio-device-monitor list-devices --verbose

# Test manual switching
audio-device-monitor switch --device "AirPods Pro"
audio-device-monitor switch --device "Blue Yeti" --input

# Check current defaults
audio-device-monitor show-default

# Validate configuration changes
audio-device-monitor check-config
```

#### Troubleshooting:
```bash
# Enable maximum logging
RUST_LOG=trace audio-device-monitor --verbose --json-logs test-monitor

# Check service logs
tail -f ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*

# Test configuration
audio-device-monitor -c ~/my-config.toml check-config

# Clean up old logs
audio-device-monitor cleanup-logs --keep-days 7
```

## Architecture

The application uses a hybrid architecture combining:

- **CoreAudio**: For audio-specific device monitoring and control
- **Core Foundation**: For macOS system integration and event loops
- **cpal**: For cross-platform device enumeration and compatibility

### Key Components

1. **Audio Device Monitor**: Main orchestrator that coordinates all components
2. **CoreAudio Listener**: Handles real-time device change notifications with automatic switching
3. **Device Controller**: Manages device enumeration, information, and switching operations
4. **Priority Manager**: Implements the weighted priority system for intelligent device selection
5. **Service Manager**: Handles background service lifecycle with graceful shutdown
6. **Configuration System**: Manages TOML configuration loading, validation, and hot-reload
7. **Logging System**: Enhanced logging with rotation, JSON output, and automated cleanup
8. **Signal Handler**: Advanced signal processing for service management

## Development

### Building from Source

```bash
git clone https://github.com/yourusername/audio-device-monitor
cd audio-device-monitor
cargo build
```

### Running Tests

```bash
# Run unit tests
cargo test

# Test device enumeration
cargo run --example list_devices

# Test real-time monitoring
cargo run --example test_notifications
```

### Development Commands

```bash
# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Run with debug logging
RUST_LOG=debug cargo run -- daemon

# Build optimized release
cargo build --release
```

## Project Status

### ✅ Completed Features

- **Phase 1**: Project foundation, CLI interface, device enumeration, configuration system
- **Phase 2**: CoreAudio integration, real-time device monitoring, priority-based recommendations  
- **Phase 3**: Priority management system with weighted device selection and intelligent fallbacks
- **Phase 4**: Automatic device switching with CoreAudio APIs and real-time event handling
- **Phase 5**: Background service infrastructure, enhanced logging, and service installation
- **Phase 6**: macOS Notification Center integration and hot configuration reload

### 🚧 Current Capabilities

- **Automatic Device Switching**: ✅ Fully functional with real-time CoreAudio monitoring
- **Service Management**: ✅ Install/uninstall as macOS LaunchAgent with auto-startup
- **Enhanced Logging**: ✅ Daily rotation, JSON output, file/console logging, automated cleanup
- **Manual Device Control**: ✅ Command-line switching for testing and manual control
- **Priority-based Selection**: ✅ Configurable weight system with multiple matching modes
- **Signal Handling**: ✅ Graceful shutdown and service lifecycle management
- **Configuration Hot-reload**: ✅ Dynamic config changes via SIGHUP signal without service restart
- **Notification System**: ✅ macOS Notification Center integration for device events

### 📋 Planned Features

- **System Tray Integration**: macOS menu bar integration for easy access
- **GUI Configuration Interface**: Visual configuration editor
- **Advanced Device Matching**: Full regex support and complex matching rules
- **Statistics and Usage Reporting**: Device usage analytics and switching history
- **Integration APIs**: Hooks for other audio applications and automation tools

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

## Troubleshooting

### Common Issues

1. **Device detection not working**
   - Ensure you have permission to access audio devices in System Preferences
   - Check that CoreAudio property listeners are properly registered
   - Verify logs for any CoreAudio errors
   - Test with: `audio-device-monitor test-monitor`

2. **Automatic switching not working**
   - Verify device names match exactly (use `list-devices` to check)
   - Ensure devices are available and not exclusively used by other applications
   - Check device priorities are set correctly in configuration
   - Test manual switching: `audio-device-monitor switch --device "Device Name"`
   - Verify priority manager is finding matches with verbose logging

3. **Service not starting**
   - Check service status: `launchctl list | grep audiodevicemonitor`
   - Verify service installation: `ls ~/Library/LaunchAgents/com.audiodevicemonitor.daemon.plist`
   - Check service logs: `tail -f /tmp/audio-device-monitor.log`
   - Reinstall service: `audio-device-monitor uninstall-service && audio-device-monitor install-service`

4. **Configuration file errors**
   - Run `audio-device-monitor check-config` to validate syntax
   - Check file permissions on `~/.config/audio-device-monitor/`
   - Ensure TOML format is correct
   - Test with custom config: `audio-device-monitor -c /path/to/config.toml check-config`

5. **Logging issues**
   - Check log directory permissions: `ls -la ~/.local/share/audio-device-monitor/logs/`
   - Try console-only mode: `audio-device-monitor --no-file-logs daemon`
   - Clean up old logs: `audio-device-monitor cleanup-logs --keep-days 7`
   - Test different log formats: `audio-device-monitor --json-logs test-monitor`

### Debug Mode

Enable detailed logging to troubleshoot issues:

```bash
# Enable debug logging with file output
audio-device-monitor --verbose daemon

# Enable trace logging (very verbose) with JSON format
RUST_LOG=trace audio-device-monitor --verbose --json-logs test-monitor

# Debug service mode
audio-device-monitor --verbose --json-logs service

# Debug with console only (no files)
audio-device-monitor --verbose --no-file-logs test-monitor
```

### Log Analysis

```bash
# View real-time logs
tail -f ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*

# View JSON logs with formatting
jq '.' ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.* | less

# Filter logs by level
grep "ERROR\|WARN" ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*

# Search for specific device events
grep "device" ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*
```

### Testing Workflow

1. **Basic Functionality**:
   ```bash
   # Test device enumeration
   audio-device-monitor list-devices --verbose
   
   # Verify configuration
   audio-device-monitor check-config
   
   # Test manual switching
   audio-device-monitor switch --device "Your Device Name"
   ```

2. **Real-time Monitoring**:
   ```bash
   # Test live monitoring
   audio-device-monitor --verbose test-monitor
   # (Try plugging/unplugging devices)
   ```

3. **Service Testing**:
   ```bash
   # Test service mode
   audio-device-monitor --verbose service
   # (Send SIGTERM with Ctrl+C to test graceful shutdown)
   ```

4. **Full Integration**:
   ```bash
   # Install and test service
   audio-device-monitor install-service
   launchctl list | grep audiodevicemonitor
   tail -f ~/.local/share/audio-device-monitor/logs/audio-device-monitor.log.*
   ```

### Getting Help

1. Check the logs for error messages (both console and file logs)
2. Verify your configuration with `check-config`
3. Test with `list-devices` to see available devices
4. Use `test-monitor` to verify real-time detection
5. Test manual switching to isolate automatic switching issues
6. Try different logging modes to capture detailed information
7. Clean up logs if disk space is an issue: `cleanup-logs --keep-days 7`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Follow Rust formatting conventions (`cargo fmt`)
4. Ensure all tests pass (`cargo test`)
5. Run clippy for linting (`cargo clippy`)
6. Submit a pull request

## License

This project is open source and available under the MIT License.

## Acknowledgments

- Built with the excellent [cpal](https://github.com/RustAudio/cpal) cross-platform audio library
- Uses [coreaudio-sys](https://github.com/RustAudio/coreaudio-sys) for CoreAudio bindings
- Leverages [core-foundation](https://github.com/servo/core-foundation-rs) for macOS system integration
- Inspired by macOS audio management tools like SwitchAudioSource and Recadio