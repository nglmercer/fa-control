use crate::AppInfo;
use libpulse_binding as pulse;
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::{Context, FlagSet as ContextFlagSet, State as ContextState};
use libpulse_binding::def::Retval;
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use libpulse_binding::proplist::properties::APPLICATION_PROCESS_ID;
use libpulse_binding::volume::ChannelVolumes;
use napi::bindgen_prelude::*;

pub struct AudioController;

impl AudioController {
  pub fn get_master_volume() -> Result<f64> {
    let volume = Self::get_sink_volume()?;
    Ok(volume as f64)
  }

  pub fn set_master_volume(volume: f64) -> Result<()> {
    if !(0.0..=1.0).contains(&volume) {
      return Err(Error::new(
        Status::InvalidArg,
        "Volume must be between 0.0 and 1.0",
      ));
    }

    Self::set_sink_volume(volume as f32)?;
    Ok(())
  }

  pub fn is_master_muted() -> Result<bool> {
    let muted = Self::get_sink_mute()?;
    Ok(muted)
  }

  pub fn toggle_master_mute() -> Result<bool> {
    let current_muted = Self::is_master_muted()?;
    Self::set_sink_mute(!current_muted)?;
    Ok(!current_muted)
  }

  pub fn set_master_mute(muted: bool) -> Result<()> {
    Self::set_sink_mute(muted)?;
    Ok(())
  }

  fn get_default_sink_name() -> Result<String> {
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-master")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let introspector = context.introspect();
    let (sink_name_tx, sink_name_rx) = std::sync::mpsc::channel();

    introspector.get_sink_info_by_index(pulse::def::INVALID_INDEX, move |result| {
      if let ListResult::Item(sink) = result {
        if let Some(name) = sink.name.as_ref() {
          let _ = sink_name_tx.send(name.to_string());
        }
      }
    });

    let _ = mainloop.iterate(true);

    let sink_name = sink_name_rx
      .recv_timeout(std::time::Duration::from_secs(5))
      .map_err(|_| Error::new(Status::GenericFailure, "Timeout getting default sink name"))?;

    Ok(sink_name)
  }

  fn get_sink_volume() -> Result<f32> {
    let sink_name = Self::get_default_sink_name()?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-get-volume")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let introspector = context.introspect();
    let (volume_tx, volume_rx) = std::sync::mpsc::channel();

    introspector.get_sink_info_by_name(&sink_name, move |result| {
      if let ListResult::Item(sink) = result {
        let avg_volume = sink.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;
        let _ = volume_tx.send(avg_volume);
      }
    });

    let _ = mainloop.iterate(true);

    let volume = volume_rx
      .recv_timeout(std::time::Duration::from_secs(5))
      .map_err(|_| Error::new(Status::GenericFailure, "Timeout getting sink volume"))?;

    Ok(volume)
  }

  fn set_sink_volume(volume: f32) -> Result<()> {
    let sink_name = Self::get_default_sink_name()?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-set-volume")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let operation = {
      let mut cv = ChannelVolumes::default();
      let vol_val = (volume * pulse::volume::Volume::NORMAL.0 as f32) as u32;
      cv.set(2, pulse::volume::Volume(vol_val));

      context
        .introspect()
        .set_sink_volume_by_name(&sink_name, &cv, None)
    };

    // Keep operation alive while we iterate
    // The operation return type is direct Operation, not Option.

    let _ = mainloop.iterate(true);
    // Explicitly drop operation to suppress unused var warning if needed, but _ = assignment handles it.
    drop(operation);

    Ok(())
  }

  fn get_sink_mute() -> Result<bool> {
    let sink_name = Self::get_default_sink_name()?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-get-mute")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let introspector = context.introspect();
    let (mute_tx, mute_rx) = std::sync::mpsc::channel();

    introspector.get_sink_info_by_name(&sink_name, move |result| {
      if let ListResult::Item(sink) = result {
        let _ = mute_tx.send(sink.mute);
      }
    });

    let _ = mainloop.iterate(true);

    let muted = mute_rx
      .recv_timeout(std::time::Duration::from_secs(5))
      .map_err(|_| Error::new(Status::GenericFailure, "Timeout getting sink mute state"))?;

    Ok(muted)
  }

  fn set_sink_mute(muted: bool) -> Result<()> {
    let sink_name = Self::get_default_sink_name()?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-set-mute")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let operation = context
      .introspect()
      .set_sink_mute_by_name(&sink_name, muted, None);

    let _ = mainloop.iterate(true);
    drop(operation);

    Ok(())
  }
}

pub struct AppVolumeController;

impl AppVolumeController {
  pub fn get_app_volume(pid: u32) -> Result<f64> {
    let volume = Self::get_sink_input_volume(pid)?;
    Ok(volume as f64)
  }

  pub fn set_app_volume(pid: u32, volume: f64) -> Result<()> {
    if !(0.0..=1.0).contains(&volume) {
      return Err(Error::new(
        Status::InvalidArg,
        "Volume must be between 0.0 and 1.0",
      ));
    }

    Self::set_sink_input_volume(pid, volume as f32)?;
    Ok(())
  }

  pub fn is_app_muted(pid: u32) -> Result<bool> {
    let muted = Self::get_sink_input_mute(pid)?;
    Ok(muted)
  }

  pub fn set_app_mute(pid: u32, muted: bool) -> Result<()> {
    Self::set_sink_input_mute(pid, muted)?;
    Ok(())
  }

  pub fn get_active_audio_apps() -> Result<Vec<AppInfo>> {
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-get-apps")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let introspector = context.introspect();
    let (apps_tx, apps_rx) = std::sync::mpsc::channel();

    introspector.get_sink_input_info_list(move |result| {
      if let ListResult::Item(sink_input) = result {
        let app_name = sink_input
          .name
          .as_ref()
          .map(|s| s.to_string())
          .unwrap_or_else(|| "Unknown".to_string());

        // Extract PID from proplist
        let pid_val = sink_input
          .proplist
          .get_str(APPLICATION_PROCESS_ID)
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(0);

        let avg_volume = sink_input.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;

        let app_info = AppInfo {
          pid: pid_val,
          name: app_name,
          volume: avg_volume as f64,
          muted: sink_input.mute,
        };

        // Filter out 0 PIDs if necessary, or just send valid ones.
        // Usually pulse streams might not have PID if they are system sounds.
        if pid_val != 0 {
          let _ = apps_tx.send(app_info);
        }
      }
    });

    let _ = mainloop.iterate(true);
    mainloop.quit(Retval(0));

    let mut apps = Vec::new();

    while let Ok(app) = apps_rx.recv_timeout(std::time::Duration::from_millis(100)) {
      apps.push(app);
    }

    Ok(apps)
  }

  fn get_sink_input_volume(pid: u32) -> Result<f32> {
    let index = Self::find_sink_input_index_by_pid(pid)?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-get-app-volume")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let introspector = context.introspect();
    let (volume_tx, volume_rx) = std::sync::mpsc::channel();

    introspector.get_sink_input_info(index, move |result| {
      if let ListResult::Item(sink_input) = result {
        let avg_volume = sink_input.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;
        let _ = volume_tx.send(avg_volume);
      }
    });

    let _ = mainloop.iterate(true);

    let volume = volume_rx
      .recv_timeout(std::time::Duration::from_secs(5))
      .map_err(|_| Error::new(Status::GenericFailure, "Timeout getting sink input volume"))?;

    Ok(volume)
  }

  fn set_sink_input_volume(pid: u32, volume: f32) -> Result<()> {
    let index = Self::find_sink_input_index_by_pid(pid)?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-set-app-volume")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let operation = {
      let mut cv = ChannelVolumes::default();
      let vol_val = (volume * pulse::volume::Volume::NORMAL.0 as f32) as u32;
      cv.set(2, pulse::volume::Volume(vol_val));

      context.introspect().set_sink_input_volume(index, &cv, None)
    };

    let _ = mainloop.iterate(true);
    drop(operation);

    Ok(())
  }

  fn get_sink_input_mute(pid: u32) -> Result<bool> {
    let index = Self::find_sink_input_index_by_pid(pid)?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-get-app-mute")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let introspector = context.introspect();
    let (mute_tx, mute_rx) = std::sync::mpsc::channel();

    introspector.get_sink_input_info(index, move |result| {
      if let ListResult::Item(sink_input) = result {
        let _ = mute_tx.send(sink_input.mute);
      }
    });

    let _ = mainloop.iterate(true);

    let muted = mute_rx
      .recv_timeout(std::time::Duration::from_secs(5))
      .map_err(|_| {
        Error::new(
          Status::GenericFailure,
          "Timeout getting sink input mute state",
        )
      })?;

    Ok(muted)
  }

  fn set_sink_input_mute(pid: u32, muted: bool) -> Result<()> {
    let index = Self::find_sink_input_index_by_pid(pid)?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-set-app-mute")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let operation = context.introspect().set_sink_input_mute(index, muted, None);

    let _ = mainloop.iterate(true);
    drop(operation);

    Ok(())
  }

  fn find_sink_input_index_by_pid(pid: u32) -> Result<u32> {
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-find-sink-input")
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create context"))?;

    context
      .connect(None, ContextFlagSet::NOFLAGS, None)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to connect: {}", e)))?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(
            Status::GenericFailure,
            "Failed to iterate pulseaudio mainloop",
          ));
        }
        IterateResult::Success(_) => {}
      }

      if let ContextState::Ready = context.get_state() {
        break;
      }
    }

    let introspector = context.introspect();
    let (index_tx, index_rx) = std::sync::mpsc::channel();

    introspector.get_sink_input_info_list(move |result| {
      if let ListResult::Item(sink_input) = result {
        let pid_val = sink_input
          .proplist
          .get_str(APPLICATION_PROCESS_ID)
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(0);

        if pid_val == pid {
          let _ = index_tx.send(sink_input.index);
        }
      }
    });

    let _ = mainloop.iterate(true);

    let index = index_rx
      .recv_timeout(std::time::Duration::from_secs(5))
      .map_err(|_| {
        Error::new(
          Status::GenericFailure,
          format!("No sink input found for PID: {}", pid),
        )
      })?;

    Ok(index)
  }
}
