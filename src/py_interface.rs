#![allow(clippy::useless_conversion)]

use pyo3::{
  exceptions::{PyRuntimeError, PyValueError},
  prelude::*,
};

use crate::platform::{AppVolumeController, AudioController, InputController};
use crate::AppInfo as RustAppInfo;

/// Python module for fa-control
#[pymodule]
#[pyo3(name = "_internal")]
pub fn fa_control(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_wrapped(wrap_pyfunction!(get_master_volume))?;
  m.add_wrapped(wrap_pyfunction!(set_master_volume))?;
  m.add_wrapped(wrap_pyfunction!(is_master_muted))?;
  m.add_wrapped(wrap_pyfunction!(toggle_master_mute))?;
  m.add_wrapped(wrap_pyfunction!(set_master_mute))?;

  m.add_wrapped(wrap_pyfunction!(get_app_volume))?;
  m.add_wrapped(wrap_pyfunction!(set_app_volume))?;
  m.add_wrapped(wrap_pyfunction!(is_app_muted))?;
  m.add_wrapped(wrap_pyfunction!(set_app_mute))?;
  m.add_wrapped(wrap_pyfunction!(get_active_audio_apps))?;

  m.add_wrapped(wrap_pyfunction!(get_microphone_volume))?;
  m.add_wrapped(wrap_pyfunction!(set_microphone_volume))?;
  m.add_wrapped(wrap_pyfunction!(is_microphone_muted))?;
  m.add_wrapped(wrap_pyfunction!(toggle_microphone_mute))?;
  m.add_wrapped(wrap_pyfunction!(set_microphone_mute))?;

  m.add_wrapped(wrap_pyfunction!(get_platform))?;

  m.add_class::<AppInfo>()?;

  Ok(())
}

/// Get master volume level (0.0 to 1.0)
#[pyfunction]
pub fn get_master_volume(_py: Python) -> PyResult<f64> {
  AudioController::get_master_volume()
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to get master volume: {}", e)))
}

/// Set master volume level (0.0 to 1.0)
#[pyfunction]
pub fn set_master_volume(_py: Python, volume: f64) -> PyResult<()> {
  if !(0.0..=1.0).contains(&volume) {
    return Err(PyValueError::new_err("Volume must be between 0.0 and 1.0"));
  }
  AudioController::set_master_volume(volume)
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to set master volume: {}", e)))
}

/// Get whether master audio is muted
#[pyfunction]
pub fn is_master_muted(_py: Python) -> PyResult<bool> {
  AudioController::is_master_muted()
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to get master mute state: {}", e)))
}

/// Toggle master mute state
#[pyfunction]
pub fn toggle_master_mute(_py: Python) -> PyResult<bool> {
  AudioController::toggle_master_mute()
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to toggle master mute: {}", e)))
}

/// Set master mute state
#[pyfunction]
pub fn set_master_mute(_py: Python, muted: bool) -> PyResult<()> {
  AudioController::set_master_mute(muted)
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to set master mute: {}", e)))
}

/// Get volume for a specific application by PID
#[pyfunction]
pub fn get_app_volume(_py: Python, pid: u32) -> PyResult<f64> {
  AppVolumeController::get_app_volume(pid)
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to get app volume: {}", e)))
}

/// Set volume for a specific application by PID
#[pyfunction]
pub fn set_app_volume(_py: Python, pid: u32, volume: f64) -> PyResult<bool> {
  if !(0.0..=1.0).contains(&volume) {
    return Err(PyValueError::new_err("Volume must be between 0.0 and 1.0"));
  }
  AppVolumeController::set_app_volume(pid, volume)
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to set app volume: {}", e)))
}

/// Get mute state for a specific application by PID
#[pyfunction]
pub fn is_app_muted(_py: Python, pid: u32) -> PyResult<bool> {
  AppVolumeController::is_app_muted(pid)
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to get app mute state: {}", e)))
}

/// Set mute state for a specific application by PID
#[pyfunction]
pub fn set_app_mute(_py: Python, pid: u32, muted: bool) -> PyResult<()> {
  AppVolumeController::set_app_mute(pid, muted)
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to set app mute: {}", e)))
}

/// Get list of all active audio applications with their PIDs and names
#[pyfunction]
pub fn get_active_audio_apps(_py: Python) -> PyResult<Vec<AppInfo>> {
  AppVolumeController::get_active_audio_apps()
    .map(|apps| apps.into_iter().map(AppInfo::from).collect())
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to get active audio apps: {}", e)))
}

/// Get microphone volume level (0.0 to 1.0)
#[pyfunction]
pub fn get_microphone_volume(_py: Python) -> PyResult<f64> {
  InputController::get_microphone_volume()
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to get microphone volume: {}", e)))
}

/// Set microphone volume level (0.0 to 1.0)
#[pyfunction]
pub fn set_microphone_volume(_py: Python, volume: f64) -> PyResult<()> {
  if !(0.0..=1.0).contains(&volume) {
    return Err(PyValueError::new_err("Volume must be between 0.0 and 1.0"));
  }
  InputController::set_microphone_volume(volume)
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to set microphone volume: {}", e)))
}

/// Get whether microphone is muted
#[pyfunction]
pub fn is_microphone_muted(_py: Python) -> PyResult<bool> {
  InputController::is_microphone_muted()
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to get microphone mute state: {}", e)))
}

/// Toggle microphone mute state
#[pyfunction]
pub fn toggle_microphone_mute(_py: Python) -> PyResult<bool> {
  InputController::toggle_microphone_mute()
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to toggle microphone mute: {}", e)))
}

/// Set microphone mute state
#[pyfunction]
pub fn set_microphone_mute(_py: Python, muted: bool) -> PyResult<()> {
  InputController::set_microphone_mute(muted)
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to set microphone mute: {}", e)))
}

/// Get current platform
#[pyfunction]
pub fn get_platform(_py: Python) -> String {
  #[cfg(windows)]
  return "windows".to_string();

  #[cfg(target_os = "linux")]
  return "linux".to_string();

  #[cfg(not(any(windows, target_os = "linux")))]
  return "unsupported".to_string();
}

/// Python representation of AppInfo
#[pyclass]
#[derive(Debug, Clone)]
pub struct AppInfo {
  pub pid: u32,
  pub name: String,
  pub volume: f64,
  pub muted: bool,
}

impl From<RustAppInfo> for AppInfo {
  fn from(info: RustAppInfo) -> Self {
    AppInfo {
      pid: info.pid,
      name: info.name,
      volume: info.volume,
      muted: info.muted,
    }
  }
}

#[pymethods]
impl AppInfo {
  #[getter]
  pub fn pid(&self) -> u32 {
    self.pid
  }

  #[getter]
  pub fn name(&self) -> &str {
    &self.name
  }

  #[getter]
  pub fn volume(&self) -> f64 {
    self.volume
  }

  #[getter]
  pub fn muted(&self) -> bool {
    self.muted
  }

  pub fn __repr__(&self) -> String {
    format!(
      "AppInfo(pid={}, name='{}', volume={:.2}, muted={})",
      self.pid, self.name, self.volume, self.muted
    )
  }

  pub fn __str__(&self) -> String {
    self.__repr__()
  }
}
