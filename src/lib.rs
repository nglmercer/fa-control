#![deny(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use napi::bindgen_prelude::*;
use napi_derive::napi;

mod platform;

use platform::{AppVolumeController, AudioController, InputController};

/// Get master volume level (0.0 to 1.0)
#[napi]
pub fn get_master_volume() -> Result<f64> {
  AudioController::get_master_volume()
}

/// Set master volume level (0.0 to 1.0)
#[napi]
pub fn set_master_volume(volume: f64) -> Result<()> {
  AudioController::set_master_volume(volume)
}

/// Get whether master audio is muted
#[napi]
pub fn is_master_muted() -> Result<bool> {
  AudioController::is_master_muted()
}

/// Toggle master mute state
#[napi]
pub fn toggle_master_mute() -> Result<bool> {
  AudioController::toggle_master_mute()
}

/// Set master mute state
#[napi]
pub fn set_master_mute(muted: bool) -> Result<()> {
  AudioController::set_master_mute(muted)
}

/// Get volume for a specific application by PID
#[napi]
pub fn get_app_volume(pid: u32) -> Result<f64> {
  AppVolumeController::get_app_volume(pid)
}

/// Set volume for a specific application by PID
#[napi]
pub fn set_app_volume(pid: u32, volume: f64) -> Result<bool> {
  AppVolumeController::set_app_volume(pid, volume)
}

/// Get mute state for a specific application by PID
#[napi]
pub fn is_app_muted(pid: u32) -> Result<bool> {
  AppVolumeController::is_app_muted(pid)
}

/// Set mute state for a specific application by PID
#[napi]
pub fn set_app_mute(pid: u32, muted: bool) -> Result<()> {
  AppVolumeController::set_app_mute(pid, muted)
}

/// Get list of all active audio applications with their PIDs and names
#[napi]
pub fn get_active_audio_apps() -> Result<Vec<AppInfo>> {
  AppVolumeController::get_active_audio_apps()
}

/// Information about an audio application
#[napi(object)]
pub struct AppInfo {
  pub pid: u32,
  pub name: String,
  pub volume: f64,
  pub muted: bool,
}

// ============== Microphone/Input Control Functions ==============

/// Get microphone volume level (0.0 to 1.0)
#[napi]
pub fn get_microphone_volume() -> Result<f64> {
  InputController::get_microphone_volume()
}

/// Set microphone volume level (0.0 to 1.0)
#[napi]
pub fn set_microphone_volume(volume: f64) -> Result<()> {
  InputController::set_microphone_volume(volume)
}

/// Get whether microphone is muted
#[napi]
pub fn is_microphone_muted() -> Result<bool> {
  InputController::is_microphone_muted()
}

/// Toggle microphone mute state
#[napi]
pub fn toggle_microphone_mute() -> Result<bool> {
  InputController::toggle_microphone_mute()
}

/// Set microphone mute state
#[napi]
pub fn set_microphone_mute(muted: bool) -> Result<()> {
  InputController::set_microphone_mute(muted)
}

// ============== Platform Info ==============

/// Get current platform
#[napi]
pub fn get_platform() -> String {
  #[cfg(windows)]
  return "windows".to_string();

  #[cfg(target_os = "linux")]
  return "linux".to_string();

  #[cfg(not(any(windows, target_os = "linux")))]
  return "unsupported".to_string();
}
