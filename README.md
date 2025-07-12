# Audio Device Monitor

A Rust-based macOS audio device monitor that automatically switches to preferred audio devices based on a weighted priority list. The system monitors for audio device changes (connections, disconnections, default device changes) and intelligently selects the highest-priority available device.

## Features

- **Real-time Device Monitoring**: Instant detection of audio device changes using CoreAudio property listeners
- **Priority-based Device Selection**: Configurable weighted priority system for automatic device switching
- **Comprehensive Device Support**: Monitors both input and output devices (microphones, speakers, headphones, etc.)
- **Background Operation**: Runs as a daemon service for continuous monitoring
- **Flexible Configuration**: TOML-based configuration with multiple device matching options
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

3. **Run in daemon mode:**
   ```bash
   audio-device-monitor daemon
   ```

4. **Check configuration:**
   ```bash
   audio-device-monitor check-config
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
- `-h, --help` - Show help information
- `-V, --version` - Show version information

#### Commands

- **`list-devices`** - List all available audio devices
  ```bash
  audio-device-monitor list-devices [--verbose]
  ```

- **`test-monitor`** - Test device monitoring (shows real-time changes)
  ```bash
  audio-device-monitor test-monitor
  ```

- **`daemon`** - Run in daemon mode (continuous monitoring)
  ```bash
  audio-device-monitor daemon
  ```

- **`check-config`** - Validate configuration file
  ```bash
  audio-device-monitor check-config
  ```

- **`show-default`** - Show current default devices
  ```bash
  audio-device-monitor show-default
  ```

### Examples

#### Monitor devices and test priority system:
```bash
# See what devices are available
audio-device-monitor list-devices

# Test real-time monitoring
audio-device-monitor test-monitor
# (Try plugging/unplugging devices to see live updates)
```

#### Run as background service:
```bash
# Start monitoring in background
audio-device-monitor daemon &

# Or run in foreground with detailed logging
RUST_LOG=debug audio-device-monitor daemon
```

#### Validate and debug configuration:
```bash
# Check if configuration is valid
audio-device-monitor check-config

# Test with custom config file
audio-device-monitor -c ~/my-audio-config.toml check-config
```

## Architecture

The application uses a hybrid architecture combining:

- **CoreAudio**: For audio-specific device monitoring and control
- **Core Foundation**: For macOS system integration and event loops
- **cpal**: For cross-platform device enumeration and compatibility

### Key Components

1. **Audio Device Monitor**: Main orchestrator that coordinates all components
2. **CoreAudio Listener**: Handles real-time device change notifications
3. **Device Controller**: Manages device enumeration and information
4. **Priority Manager**: Implements the weighted priority system
5. **Configuration System**: Manages TOML configuration loading and validation

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

### 🚧 In Progress

- **Phase 3**: Priority management refinements and device matching improvements
- **Phase 4**: Automatic device switching implementation

### 📋 Planned Features

- **Phase 5**: Background service installation, system tray integration, enhanced logging
- GUI configuration interface
- Advanced device matching rules (regex support)
- Statistics and usage reporting
- Integration with other audio applications

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

2. **Configuration file errors**
   - Run `audio-device-monitor check-config` to validate syntax
   - Check file permissions on `~/.config/audio-device-monitor/`
   - Ensure TOML format is correct

3. **Device switching not working**
   - Verify device names match exactly (use `list-devices` to check)
   - Ensure devices are available and not exclusively used by other applications
   - Check device priorities are set correctly

### Debug Mode

Enable detailed logging to troubleshoot issues:

```bash
# Enable debug logging
RUST_LOG=debug audio-device-monitor daemon

# Enable trace logging (very verbose)
RUST_LOG=trace audio-device-monitor test-monitor
```

### Getting Help

1. Check the logs for error messages
2. Verify your configuration with `check-config`
3. Test with `list-devices` to see available devices
4. Use `test-monitor` to verify real-time detection

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