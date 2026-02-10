#![deny(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod platform;

// ============== PyO3 Bindings (Python) ==============
// These are only compiled when the "pyo3" feature is enabled

#[cfg(feature = "pyo3")]
pub mod py_interface;

/// Information about an audio application
#[derive(Debug, Clone)]
pub struct AppInfo {
  pub pid: u32,
  pub name: String,
  pub volume: f64,
  pub muted: bool,
}

/// Get current platform
pub fn get_platform() -> String {
  #[cfg(windows)]
  return "windows".to_string();

  #[cfg(target_os = "linux")]
  return "linux".to_string();

  #[cfg(not(any(windows, target_os = "linux")))]
  return "unsupported".to_string();
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_platform() {
    let platform = get_platform();
    assert!(platform == "windows" || platform == "linux" || platform == "unsupported");
  }
}
