use crate::AppInfo;
use napi::bindgen_prelude::*;
use windows::{
  core::Interface,
  Win32::Foundation::BOOL,
  Win32::Media::Audio::{
    EDataFlow, ERole, IAudioSessionControl, IAudioSessionControl2, IAudioSessionEnumerator,
    IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
  },
  Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
};

pub struct AudioController;

impl AudioController {
  pub fn get_master_volume() -> Result<f64> {
    Err(Error::new(
      Status::GenericFailure,
      "Master volume control not available in this version. Please use per-application volume control instead.",
    ))
  }

  pub fn set_master_volume(_volume: f64) -> Result<()> {
    Err(Error::new(
      Status::GenericFailure,
      "Master volume control not available in this version. Please use per-application volume control instead.",
    ))
  }

  pub fn is_master_muted() -> Result<bool> {
    Err(Error::new(
      Status::GenericFailure,
      "Master volume control not available in this version. Please use per-application volume control instead.",
    ))
  }

  pub fn toggle_master_mute() -> Result<bool> {
    Err(Error::new(
      Status::GenericFailure,
      "Master volume control not available in this version. Please use per-application volume control instead.",
    ))
  }

  pub fn set_master_mute(_muted: bool) -> Result<()> {
    Err(Error::new(
      Status::GenericFailure,
      "Master volume control not available in this version. Please use per-application volume control instead.",
    ))
  }
}

pub struct AppVolumeController;

impl AppVolumeController {
  pub fn get_app_volume(pid: u32) -> Result<f64> {
    unsafe {
      let session_manager = Self::get_session_manager()?;
      let session = Self::find_session_by_pid(&session_manager, pid)?;
      let simple_volume: ISimpleAudioVolume = session.cast().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to cast session: {}", e),
        )
      })?;

      let volume = simple_volume.GetMasterVolume().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to get app volume: {}", e),
        )
      })?;

      Ok(volume as f64)
    }
  }

  pub fn set_app_volume(pid: u32, volume: f64) -> Result<()> {
    if !(0.0..=1.0).contains(&volume) {
      return Err(Error::new(
        Status::InvalidArg,
        "Volume must be between 0.0 and 1.0",
      ));
    }

    unsafe {
      let session_manager = Self::get_session_manager()?;
      let session = Self::find_session_by_pid(&session_manager, pid)?;
      let simple_volume: ISimpleAudioVolume = session.cast().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to cast session: {}", e),
        )
      })?;

      simple_volume
        .SetMasterVolume(volume as f32, std::ptr::null())
        .map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to set app volume: {}", e),
          )
        })?;

      Ok(())
    }
  }

  pub fn is_app_muted(pid: u32) -> Result<bool> {
    unsafe {
      let session_manager = Self::get_session_manager()?;
      let session = Self::find_session_by_pid(&session_manager, pid)?;
      let simple_volume: ISimpleAudioVolume = session.cast().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to cast session: {}", e),
        )
      })?;

      let muted = simple_volume.GetMute().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to get app mute: {}", e),
        )
      })?;

      Ok(muted.as_bool())
    }
  }

  pub fn set_app_mute(pid: u32, muted: bool) -> Result<()> {
    unsafe {
      let session_manager = Self::get_session_manager()?;
      let session = Self::find_session_by_pid(&session_manager, pid)?;
      let simple_volume: ISimpleAudioVolume = session.cast().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to cast session: {}", e),
        )
      })?;

      simple_volume
        .SetMute(BOOL::from(muted), std::ptr::null())
        .map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to set app mute: {}", e),
          )
        })?;

      Ok(())
    }
  }

  pub fn get_active_audio_apps() -> Result<Vec<AppInfo>> {
    unsafe {
      let session_manager = Self::get_session_manager()?;
      let session_enum: IAudioSessionEnumerator =
        session_manager.GetSessionEnumerator().map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to get session enumerator: {}", e),
          )
        })?;

      let count = session_enum.GetCount().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to get session count: {}", e),
        )
      })?;

      let mut apps = Vec::new();

      for i in 0..count {
        if let Ok(session) = session_enum.GetSession(i) {
          if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
            if let Ok(process_id) = session2.GetProcessId() {
              if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                let volume = simple_volume.GetMasterVolume().unwrap_or(0.0);
                let muted = simple_volume
                  .GetMute()
                  .unwrap_or(BOOL::from(false))
                  .as_bool();

                let display_name = session
                  .GetDisplayName()
                  .map(|s| s.to_string())
                  .unwrap_or_else(|_| Ok("Unknown".to_string()))
                  .unwrap_or_else(|_| "Unknown".to_string());

                apps.push(AppInfo {
                  pid: process_id,
                  name: display_name,
                  volume: volume as f64,
                  muted,
                });
              }
            }
          }
        }
      }

      Ok(apps)
    }
  }

  unsafe fn get_session_manager() -> Result<IAudioSessionManager2> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED).ok();

    let device_enumerator: IMMDeviceEnumerator =
      CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to create device enumerator: {}", e),
        )
      })?;

    let device: IMMDevice = device_enumerator
      .GetDefaultAudioEndpoint(EDataFlow(0), ERole(1))
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to get default audio endpoint: {}", e),
        )
      })?;

    let session_manager: IAudioSessionManager2 =
      device.Activate(CLSCTX_ALL, None).map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to activate session manager: {}", e),
        )
      })?;

    Ok(session_manager)
  }

  unsafe fn find_session_by_pid(
    session_manager: &IAudioSessionManager2,
    pid: u32,
  ) -> Result<IAudioSessionControl> {
    let session_enum: IAudioSessionEnumerator =
      session_manager.GetSessionEnumerator().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to get session enumerator: {}", e),
        )
      })?;

    let count = session_enum.GetCount().map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to get session count: {}", e),
      )
    })?;

    for i in 0..count {
      if let Ok(session) = session_enum.GetSession(i) {
        if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
          if let Ok(process_id) = session2.GetProcessId() {
            if process_id == pid {
              return Ok(session);
            }
          }
        }
      }
    }

    Err(Error::new(
      Status::GenericFailure,
      format!("No audio session found for PID: {}", pid),
    ))
  }
}
