pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(windows)]
pub use windows::{AppVolumeController, AudioController};

#[cfg(target_os = "linux")]
pub use linux::{AppVolumeController, AudioController};

#[cfg(not(any(windows, target_os = "linux")))]
mod unsupported {
  use crate::AppInfo;
  use napi::bindgen_prelude::*;

  pub struct AudioController;

  impl AudioController {
    pub fn get_master_volume() -> Result<f64> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }

    pub fn set_master_volume(_volume: f64) -> Result<()> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }

    pub fn is_master_muted() -> Result<bool> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }

    pub fn toggle_master_mute() -> Result<bool> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }

    pub fn set_master_mute(_muted: bool) -> Result<()> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }
  }

  pub struct AppVolumeController;

  impl AppVolumeController {
    pub fn get_app_volume(_pid: u32) -> Result<f64> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }

    pub fn set_app_volume(_pid: u32, _volume: f64) -> Result<()> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }

    pub fn is_app_muted(_pid: u32) -> Result<bool> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }

    pub fn set_app_mute(_pid: u32, _muted: bool) -> Result<()> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }

    pub fn get_active_audio_apps() -> Result<Vec<AppInfo>> {
      Err(Error::new(
        Status::GenericFailure,
        "Platform not supported. Only Windows and Linux are supported.",
      ))
    }
  }
}

#[cfg(not(any(windows, target_os = "linux")))]
pub use unsupported::{AppVolumeController, AudioController};
