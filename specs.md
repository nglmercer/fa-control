## 1. Control Maestro Multiplataforma: `cpvc`

Mencionaste **CPVC** (Cross-Platform Volume Control). Es una opción sólida y ligera para el volumen maestro.

* **Ventaja:** Abstrae bien los backends (CoreAudio en macOS, WinMM/WASAPI en Windows, y ALSA en Linux).
* **Limitación:** Se queda corto cuando necesitas granularidad por aplicación o interactuar con servidores modernos como PipeWire.

---

## 2. Control por Aplicación (Per-App Volume)

Para controlar aplicaciones de forma individual, no existe una única librería "mágica" que lo haga todo en Rust, por lo que te recomiendo usar estos crates específicos según el sistema operativo:

### Para Windows: `windows-rs`

En Windows, el control por aplicación se gestiona a través de **WASAPI** (Windows Audio Session API). No necesitas una librería de terceros compleja; lo ideal es usar los bindings oficiales de Microsoft para Rust.

* **Librería:** [windows-rs](https://github.com/microsoft/windows-rs)
* **Cómo funciona:** Debes acceder al `IAudioSessionManager2` para enumerar las sesiones de audio activas. Cada sesión corresponde a un proceso (como Spotify o Chrome) y permite ajustar su volumen de forma independiente.

### Para Linux: `libpulse-binding` o `pipewire`

En Linux, la mayoría de las distros modernas usan **PipeWire** (compatible con PulseAudio).

* **Recomendación:** [libpulse-binding](https://github.com/jnqnfe/pulse-binding-rust)
* **Razón:** Aunque PipeWire es el futuro, la API de PulseAudio sigue siendo el estándar de facto para controlar el volumen de "streams" individuales (identificados por `sink-input`). Casi cualquier sistema Linux moderno aceptará comandos de PulseAudio.
* **Alternativa moderna:** Si quieres ser "PipeWire nativo", usa el crate [pipewire](https://www.google.com/search?q=https://github.com/PipeWire/pipewire-rs), aunque es más complejo de implementar para principiantes.

---

## Resumen de Stack para N-API

Para construir tu módulo de Node.js, te sugiero la siguiente estructura de dependencias en tu `Cargo.toml`:

| Plataforma | Control Maestro | Control por Aplicación |
| --- | --- | --- |
| **Windows** | `cpvc` / `windows` | `windows` (IAudioSessionEnumerator) |
| **Linux** | `cpvc` / `libpulse` | `libpulse-binding` (Sink Inputs) |
| **N-API Bridge** | **`napi-rs`** | **`napi-rs`** |

### Ejemplo de flujo lógico para tu N-API:

1. **Frontend (Electron/Node):** Llama a `setAppVolume(pid, 0.5)`.
2. **Rust (N-API):** Recibe el comando y detecta el OS.
3. **Lógica OS:**
* Si **Windows**: Filtra las sesiones de WASAPI por el Process ID (PID) y cambia el `SimpleVolume`.
* Si **Linux**: Busca el `sink-input` asociado al nombre o PID de la aplicación mediante el contexto de PulseAudio.

## Para control de volumen MASTER multiplataforma:
- **cpal**: Es la librería más popular para audio cross-platform en Rust, pero según el issue #551, NO tiene controles de volumen integrados para dispositivos
- **phonic**: Basada en cpal, pero enfocada en playback y DSP, no en control de volumen del sistema
- **awedio**: Similar, para playback de audio

## Para Windows:
- **windows-volume-control**: Crate específico para control de volumen en Windows usando la API de Windows
- Windows tiene la **Windows Audio Session API (WASAPI)** que permite controlar volumen por aplicación

## Para Linux:
- **pulsectl**: Wrapper de alto nivel para PulseAudio
- **libpulse-binding**: Bindings de bajo nivel para PulseAudio
- **rsmixer**: Implementación de mixer en Rust para PulseAudio
- **volume-ctl**: CLI tool para controlar volumen de PulseAudio
