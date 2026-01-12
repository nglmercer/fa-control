use crate::AppInfo;
use napi::bindgen_prelude::*;
use std::path::Path;
use windows::{
  core::Interface,
  Win32::Foundation::{CloseHandle, BOOL},
  Win32::Media::Audio::{
    EDataFlow, ERole, IAudioSessionControl, IAudioSessionControl2, IAudioSessionEnumerator,
    IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
  },
  Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
  Win32::System::ProcessStatus::K32GetModuleFileNameExW,
  Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
};

pub struct AudioController;

impl AudioController {
  pub fn get_master_volume() -> Result<f64> {
    unsafe {
      let device = Self::get_default_device()?;
      let session_manager: IAudioSessionManager2 =
        device.Activate(CLSCTX_ALL, None).map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to activate session manager: {}", e),
          )
        })?;

      let session_enum = session_manager.GetSessionEnumerator().map_err(|e| {
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

      if count > 0 {
        let session = session_enum.GetSession(0).map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to get session: {}", e),
          )
        })?;

        let simple_volume: ISimpleAudioVolume = session.cast().map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to cast to ISimpleAudioVolume: {}", e),
          )
        })?;

        let volume = simple_volume.GetMasterVolume().map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to get master volume: {}", e),
          )
        })?;

        Ok(volume as f64)
      } else {
        Ok(1.0) // Default to full volume if no sessions
      }
    }
  }

  pub fn set_master_volume(volume: f64) -> Result<()> {
    if !(0.0..=1.0).contains(&volume) {
      return Err(Error::new(
        Status::InvalidArg,
        "Volume must be between 0.0 and 1.0",
      ));
    }

    unsafe {
      let device = Self::get_default_device()?;
      let session_manager: IAudioSessionManager2 =
        device.Activate(CLSCTX_ALL, None).map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to activate session manager: {}", e),
          )
        })?;

      let session_enum = session_manager.GetSessionEnumerator().map_err(|e| {
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
          if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
            let _ = simple_volume.SetMasterVolume(volume as f32, std::ptr::null());
          }
        }
      }

      Ok(())
    }
  }

  pub fn is_master_muted() -> Result<bool> {
    unsafe {
      let device = Self::get_default_device()?;
      let session_manager: IAudioSessionManager2 =
        device.Activate(CLSCTX_ALL, None).map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to activate session manager: {}", e),
          )
        })?;

      let session_enum = session_manager.GetSessionEnumerator().map_err(|e| {
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

      if count > 0 {
        let session = session_enum.GetSession(0).map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to get session: {}", e),
          )
        })?;

        let simple_volume: ISimpleAudioVolume = session.cast().map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to cast to ISimpleAudioVolume: {}", e),
          )
        })?;

        let muted = simple_volume.GetMute().map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to get master mute: {}", e),
          )
        })?;

        Ok(muted.as_bool())
      } else {
        Ok(false) // Default to not muted if no sessions
      }
    }
  }

  pub fn toggle_master_mute() -> Result<bool> {
    let current_mute = Self::is_master_muted()?;
    Self::set_master_mute(!current_mute)?;
    Ok(!current_mute)
  }

  pub fn set_master_mute(muted: bool) -> Result<()> {
    unsafe {
      let device = Self::get_default_device()?;
      let session_manager: IAudioSessionManager2 =
        device.Activate(CLSCTX_ALL, None).map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to activate session manager: {}", e),
          )
        })?;

      let session_enum = session_manager.GetSessionEnumerator().map_err(|e| {
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
          if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
            let _ = simple_volume.SetMute(BOOL::from(muted), std::ptr::null());
          }
        }
      }

      Ok(())
    }
  }

  unsafe fn get_default_device() -> Result<IMMDevice> {
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

    Ok(device)
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

  pub fn set_app_volume(pid: u32, volume: f64) -> Result<bool> {
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

      Ok(true)
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

                // Get the process name from PID
                let process_name = if process_id == 0 {
                  // System process
                  "System".to_string()
                } else {
                  // Get actual process name
                  Self::get_process_name(process_id).unwrap_or_else(|_| "Unknown".to_string())
                };

                apps.push(AppInfo {
                  pid: process_id,
                  name: process_name,
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

  /// Get the process name from a PID
  unsafe fn get_process_name(pid: u32) -> Result<String> {
    let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, BOOL::from(false), pid)
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to open process: {}", e),
        )
      })?;

    let mut buffer = [0u16; 520]; // Larger buffer for full path
    let size = K32GetModuleFileNameExW(process_handle, None, &mut buffer);

    let _ = CloseHandle(process_handle);

    if size == 0 {
      return Err(Error::new(
        Status::GenericFailure,
        "Failed to get module file name",
      ));
    }

    // Convert to string and extract just the file name
    let path_str = String::from_utf16_lossy(&buffer[..size as usize]);
    let name = Path::new(&path_str)
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("Unknown")
      .to_string();

    Ok(name)
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
