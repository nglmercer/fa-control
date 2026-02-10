# fa-control

A robust cross-platform audio control library written in Rust, providing bindings for both **Node.js** and **Python**. It offers master volume control and per-application volume control for Windows and Linux.

## Features

- **Master Volume Control**: Get and set the system master volume (Linux only)
- **Master Mute Control**: Get, set, and toggle master mute state (Linux only)
- **Per-Application Volume Control**: Get and set volume for specific applications by PID
- **Per-Application Mute Control**: Get and set mute state for specific applications by PID
- **List Active Audio Apps**: Get a list of all active audio applications with their PIDs, names, volumes, and mute states
- **Microphone/Input Control**: Get and set microphone volume and mute state
- **Python & Node.js Support**: Use from either language with native performance

## Platform Support

- ✅ **Windows**: Per-application volume control using WASAPI (Windows Audio Session API)
- ✅ **Linux**: Full support (master and per-application) using PulseAudio (compatible with PipeWire)
- ❌ **macOS**: Not currently supported (returns error)
- ❌ **Other platforms**: Not supported (returns error)

### Platform-Specific Features

| Feature | Windows | Linux |
|----------|----------|--------|
| Master Volume Control | ❌ Not available | ✅ Supported |
| Master Mute Control | ❌ Not available | ✅ Supported |
| Microphone Control | ✅ Supported | ✅ Supported |
| Per-Application Volume | ✅ Supported | ✅ Supported |
| Per-Application Mute | ✅ Supported | ✅ Supported |
| List Active Audio Apps | ✅ Supported | ✅ Supported |

## Python & Node.js Support

This library provides native bindings for both Python and Node.js:

- **Python**: Import as `fa_control` module (requires Python 3.8+)
- **Node.js**: Import as `fa-control` package

## Installation

### Node.js

```bash
npm install fa-control
```

### Python

```bash
# Using pip (once published)
pip install fa-control

# Or build from source
cd python
maturin build --release
pip install target/wheels/fa_control-*.whl
```

## Usage

### Node.js / JavaScript

```javascript
const faControl = require('fa-control');

// Master Volume Control (Linux only)
const currentVolume = faControl.getMasterVolume(); // Returns 0.0 to 1.0
faControl.setMasterVolume(0.5); // Set to 50%

const isMuted = faControl.isMasterMuted(); // Check if muted
faControl.setMasterMute(true); // Mute audio
faControl.toggleMasterMute(); // Toggle mute state

// Microphone Control
const micVolume = faControl.getMicrophoneVolume();
faControl.setMicrophoneVolume(0.7);
const isMicMuted = faControl.isMicrophoneMuted();
faControl.setMicrophoneMute(true);
faControl.toggleMicrophoneMute();

// Per-Application Volume Control (requires PID)
const appPid = 1234; // Process ID of the application
const appVolume = faControl.getAppVolume(appPid);
faControl.setAppVolume(appPid, 0.7); // Set to 70%

const isAppMuted = faControl.isAppMuted(appPid);
faControl.setAppMute(appPid, true); // Mute the application

// List all active audio applications
const activeApps = faControl.getActiveAudioApps();
console.log(activeApps);
// Output: [
//   { pid: 1234, name: "Spotify", volume: 0.5, muted: false },
//   { pid: 5678, name: "Chrome", volume: 0.8, muted: false },
//   ...
// ]

// Get current platform
const platform = faControl.getPlatform(); // Returns "windows" or "linux"
```

### Python

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

All functions return Promises in Node.js (except `getPlatform()`) and may throw errors on failure.

### Master Volume Functions (Linux only)

#### `getMasterVolume(): Promise<number>` / `get_master_volume() -> float`
Returns the master volume level as a float between 0.0 and 1.0.

#### `setMasterVolume(volume: number): Promise<void>` / `set_master_volume(volume: float) -> None`
Sets the master volume level. Volume must be between 0.0 and 1.0.

#### `isMasterMuted(): Promise<boolean>` / `is_master_muted() -> bool`
Returns whether the master audio is muted.

#### `setMasterMute(muted: boolean): Promise<void>` / `set_master_mute(muted: bool) -> None`
Sets the master mute state.

#### `toggleMasterMute(): Promise<boolean>` / `toggle_master_mute() -> bool`
Toggles the master mute state and returns the new state.

### Microphone/Input Control Functions

#### `getMicrophoneVolume(): Promise<number>` / `get_microphone_volume() -> float`
Returns the microphone volume level as a float between 0.0 and 1.0.

#### `setMicrophoneVolume(volume: number): Promise<void>` / `set_microphone_volume(volume: float) -> None`
Sets the microphone volume level. Volume must be between 0.0 and 1.0.

#### `isMicrophoneMuted(): Promise<boolean>` / `is_microphone_muted() -> bool`
Returns whether the microphone is muted.

#### `setMicrophoneMute(muted: boolean): Promise<void>` / `set_microphone_mute(muted: bool) -> None`
Sets the microphone mute state.

#### `toggleMicrophoneMute(): Promise<boolean>` / `toggle_microphone_mute() -> bool`
Toggles the microphone mute state and returns the new state.

### Per-Application Volume Functions

#### `getAppVolume(pid: number): Promise<number>` / `get_app_volume(pid: int) -> float`
Returns the volume level for the application with the given PID.

#### `setAppVolume(pid: number, volume: number): Promise<boolean>` / `set_app_volume(pid: int, volume: float) -> None`
Sets the volume level for the application with the given PID. Volume must be between 0.0 and 1.0.

#### `isAppMuted(pid: number): Promise<boolean>` / `is_app_muted(pid: int) -> bool`
Returns whether the application with the given PID is muted.

#### `setAppMute(pid: number, muted: boolean): Promise<void>` / `set_app_mute(pid: int, muted: bool) -> None`
Sets the mute state for the application with the given PID.

#### `getActiveAudioApps(): Promise<AppInfo[]>` / `get_active_audio_apps() -> List[AppInfo]`
Returns a list/array of active audio applications with their details.

### Utility Functions

#### `getPlatform(): string` / `get_platform() -> str`
Returns the current platform: `"windows"`, `"linux"`, or `"unsupported"`.

### Types

#### JavaScript / TypeScript

```typescript
interface AppInfo {
  pid: number;      // Process ID
  name: string;     // Application name
  volume: number;   // Volume level (0.0 to 1.0)
  muted: boolean;   // Mute state
}
```

#### Python

```python
@dataclass
class AppInfo:
    pid: int
    name: str
    volume: float
    muted: bool
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
- Requires PulseAudio server running
- Per-application control works with sink-inputs
- Application names may vary depending on how the application identifies itself
- Full master volume and mute control available

## Getting PIDs for Applications

### Node.js
```javascript
// You can get PIDs from the active apps list
const apps = await faControl.getActiveAudioApps();
apps.forEach(app => {
  console.log(`${app.pid}: ${app.name}`);
});
```

### Python
```python
# Get PIDs from the active apps list
apps = fa_control.get_active_audio_apps()
for app in apps:
    print(f"{app.pid}: {app.name}")
```

## Building from Source

### Prerequisites

- **Rust**: Install from https://rustup.rs/
- **Node.js**: For building N-API bindings (optional)
- **Python 3.8+**: For building Python bindings (optional)
- **maturin**: `pip install maturin` (for Python builds)

### Building Node.js Bindings

```bash
# Install dependencies
npm install

# Build for current platform
npm run build

# Build for all platforms
npm run build:all

# Run tests
npm test
```

### Building Python Bindings

```bash
# Navigate to python directory
cd python

# Build and install in development mode
maturin develop

# Or build a wheel
maturin build --release

# Install the built wheel
pip install target/wheels/fa_control-*.whl

# Run tests
pytest python/tests/
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
cargo clippy -- -D warnings

# Run tests (both Node.js and Python)
npm test
cd python && pytest

# Build both bindings
cargo build --release
cd python && maturin build --release
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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
