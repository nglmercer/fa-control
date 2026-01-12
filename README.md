# fa-control

A robust cross-platform audio control library for Node.js written in Rust. It provides master volume control and per-application volume control for Windows and Linux.

## Features

- **Master Volume Control**: Get and set the system master volume
- **Master Mute Control**: Get, set, and toggle master mute state
- **Per-Application Volume Control**: Get and set volume for specific applications by PID
- **Per-Application Mute Control**: Get and set mute state for specific applications by PID
- **List Active Audio Apps**: Get a list of all active audio applications with their PIDs, names, volumes, and mute states

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
| Per-Application Volume | ✅ Supported | ✅ Supported |
| Per-Application Mute | ✅ Supported | ✅ Supported |
| List Active Audio Apps | ✅ Supported | ✅ Supported |

## Installation

```bash
npm install fa-control
```

## Usage

```javascript
const faControl = require('fa-control');

// Master Volume Control
const currentVolume = faControl.getMasterVolume(); // Returns 0.0 to 1.0
faControl.setMasterVolume(0.5); // Set to 50%

const isMuted = faControl.isMasterMuted(); // Check if muted
faControl.setMasterMute(true); // Mute audio
faControl.toggleMasterMute(); // Toggle mute state

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

## API Reference

### Master Volume Functions

#### `getMasterVolume(): number`
Returns the master volume level as a float between 0.0 and 1.0.

#### `setMasterVolume(volume: number): void`
Sets the master volume level. Volume must be between 0.0 and 1.0.

#### `isMasterMuted(): boolean`
Returns whether the master audio is muted.

#### `setMasterMute(muted: boolean): void`
Sets the master mute state.

#### `toggleMasterMute(): boolean`
Toggles the master mute state and returns the new state.

### Per-Application Volume Functions

#### `getAppVolume(pid: number): number`
Returns the volume level for the application with the given PID.

#### `setAppVolume(pid: number, volume: number): void`
Sets the volume level for the application with the given PID. Volume must be between 0.0 and 1.0.

#### `isAppMuted(pid: number): boolean`
Returns whether the application with the given PID is muted.

#### `setAppMute(pid: number, muted: boolean): void`
Sets the mute state for the application with the given PID.

#### `getActiveAudioApps(): AppInfo[]`
Returns an array of active audio applications.

### Utility Functions

#### `getPlatform(): string`
Returns the current platform: "windows", "linux", or "unsupported".

### Types

#### `AppInfo`
```typescript
interface AppInfo {
  pid: number;      // Process ID
  name: string;     // Application name
  volume: number;   // Volume level (0.0 to 1.0)
  muted: boolean;    // Mute state
}
```

## Platform-Specific Notes

### Windows
- Uses WASAPI (Windows Audio Session API)
- Requires Windows Vista or later
- Per-application control works with most modern applications
- Some applications may not appear in the active apps list if they don't use the standard audio APIs

### Linux
- Uses PulseAudio (compatible with PipeWire)
- Requires PulseAudio server running
- Per-application control works with sink-inputs
- Application names may vary depending on how the application identifies itself

## Building from Source

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

## Development

```bash
# Install Rust toolchain
rustup install stable

# Format code
npm run format

# Lint code
npm run lint

# Run Clippy
npm run clippy
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [ ] macOS support
- [ ] Input device control
- [ ] Audio device enumeration
- [ ] Volume change events
- [ ] Per-channel volume control
