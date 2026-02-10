# fa-control

A robust cross-platform audio control library written in Rust with **Python** bindings. It offers master volume control and per-application volume control for Windows and Linux.

## Features

- **Master Volume Control**: Get and set the system master volume (Linux only)
- **Master Mute Control**: Get, set, and toggle master mute state (Linux only)
- **Per-Application Volume Control**: Get and set volume for specific applications by PID
- **Per-Application Mute Control**: Get and set mute state for specific applications by PID
- **List Active Audio Apps**: Get a list of all active audio applications with their PIDs, names, volumes, and mute states
- **Microphone/Input Control**: Get and set microphone volume and mute state

## Platform Support

- ✅ **Windows**: Per-application volume control using WASAPI (Windows Audio Session API)
- ✅ **Linux**: Full support (master and per-application) using PulseAudio (compatible with PipeWire)
- ❌ **macOS**: Not currently supported (returns error)
- ❌ **Other platforms**: Not supported (returns error)

### Platform-Specific Features

| Feature | Windows | Linux |
|----------|---------|-------|
| Master Volume Control | ❌ Not available | ✅ Supported |
| Master Mute Control | ❌ Not available | ✅ Supported |
| Microphone Control | ✅ Supported | ✅ Supported |
| Per-Application Volume | ✅ Supported | ✅ Supported |
| Per-Application Mute | ✅ Supported | ✅ Supported |
| List Active Audio Apps | ✅ Supported | ✅ Supported |

## Installation

### Using pip

```bash
pip install fa-control
```

### Building from Source

```bash
# Install maturin
pip install maturin

# Build and install
maturin develop

# Or build a wheel
maturin build --release
pip install target/wheels/fa_control-*.whl
```

## Usage

```python
import fa_control

# Get current platform
platform = fa_control.get_platform()
print(f"Running on: {platform}")

# Master Volume Control (Linux only)
current_volume = fa_control.get_master_volume()  # Returns 0.0 to 1.0
fa_control.set_master_volume(0.5)  # Set to 50%

is_muted = fa_control.is_master_muted()  # Check if muted
fa_control.set_master_mute(True)  # Mute audio
fa_control.toggle_master_mute()  # Toggle mute state

# Microphone Control
mic_volume = fa_control.get_microphone_volume()
fa_control.set_microphone_volume(0.7)
is_mic_muted = fa_control.is_microphone_muted()
fa_control.set_microphone_mute(True)
fa_control.toggle_microphone_mute()

# Per-Application Volume Control (requires PID)
app_pid = 1234  # Process ID of the application
app_volume = fa_control.get_app_volume(app_pid)
fa_control.set_app_volume(app_pid, 0.7)  # Set to 70%

is_app_muted = fa_control.is_app_muted(app_pid)
fa_control.set_app_mute(app_pid, True)  # Mute the application

# List all active audio applications
active_apps = fa_control.get_active_audio_apps()
print(active_apps)
# Output: [
#   AppInfo(pid=1234, name='Spotify', volume=0.5, muted=False),
#   AppInfo(pid=5678, name='Chrome', volume=0.8, muted=False),
#   ...
# ]

# AppInfo object properties
if active_apps:
    app = active_apps[0]
    print(f"PID: {app.pid}")
    print(f"Name: {app.name}")
    print(f"Volume: {app.volume}")
    print(f"Muted: {app.muted}")
```

## API Reference

### Master Volume Functions (Linux only)

#### `get_master_volume() -> float`
Returns the master volume level as a float between 0.0 and 1.0.

#### `set_master_volume(volume: float) -> None`
Sets the master volume level. Volume must be between 0.0 and 1.0.

#### `is_master_muted() -> bool`
Returns whether the master audio is muted.

#### `set_master_mute(muted: bool) -> None`
Sets the master mute state.

#### `toggle_master_mute() -> bool`
Toggles the master mute state and returns the new state.

### Microphone/Input Control Functions

#### `get_microphone_volume() -> float`
Returns the microphone volume level as a float between 0.0 and 1.0.

#### `set_microphone_volume(volume: float) -> None`
Sets the microphone volume level. Volume must be between 0.0 and 1.0.

#### `is_microphone_muted() -> bool`
Returns whether the microphone is muted.

#### `set_microphone_mute(muted: bool) -> None`
Sets the microphone mute state.

#### `toggle_microphone_mute() -> bool`
Toggles the microphone mute state and returns the new state.

### Per-Application Volume Functions

#### `get_app_volume(pid: int) -> float`
Returns the volume level for the application with the given PID.

#### `set_app_volume(pid: int, volume: float) -> None`
Sets the volume level for the application with the given PID. Volume must be between 0.0 and 1.0.

#### `is_app_muted(pid: int) -> bool`
Returns whether the application with the given PID is muted.

#### `set_app_mute(pid: int, muted: bool) -> None`
Sets the mute state for the application with the given PID.

#### `get_active_audio_apps() -> List[AppInfo]`
Returns a list of active audio applications with their details.

### Utility Functions

#### `get_platform() -> str`
Returns the current platform: `"windows"`, `"linux"`, or `"unsupported"`.

### Types

```python
class AppInfo:
    pid: int        # Process ID
    name: str       # Application name
    volume: float   # Volume level (0.0 to 1.0)
    muted: bool     # Mute state
```

## Platform-Specific Notes

### Windows
- Uses WASAPI (Windows Audio Session API)
- Requires Windows Vista or later
- Per-application control works with most modern applications
- Some applications may not appear in the active apps list if they don't use the standard audio APIs
- Master volume control is not available on Windows (only per-application and microphone control)

### Linux
- Uses PulseAudio (compatible with PipeWire)
- **Requires**: `libpulse-dev` and `libasound2-dev` packages
  ```bash
  # Debian/Ubuntu
  sudo apt-get install libpulse-dev libasound2-dev
  ```
- Requires PulseAudio server running
- Per-application control works with sink-inputs
- Application names may vary depending on how the application identifies itself
- Full master volume and mute control available

## Getting PIDs for Applications

```python
# Get PIDs from the active apps list
apps = fa_control.get_active_audio_apps()
for app in apps:
    print(f"{app.pid}: {app.name}")
```

## Building from Source

### Prerequisites

- **Rust**: Install from https://rustup.rs/
- **Python 3.8+**
- **maturin**: `pip install maturin`

### Building

```bash
# Build and install in development mode
maturin develop

# Or build a wheel
maturin build --release

# Install the built wheel
pip install target/wheels/fa_control-*.whl
```

## Development

```bash
# Install Rust toolchain
rustup install stable

# Format code
cargo fmt

# Lint code
cargo check

# Run Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-targets --all-features

# Run Python tests
cd python && pytest
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [ ] macOS support
- [ ] Audio device enumeration and selection
- [ ] Volume change events/callbacks
- [ ] Per-channel volume control
- [ ] WASAPI exclusive mode support
