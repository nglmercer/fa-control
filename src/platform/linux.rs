use crate::AppInfo;
use libpulse_binding as pulse;
use libpulse_binding::context::{Context, FlagSet as ContextFlagSet, State as ContextState};
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use napi::bindgen_prelude::*;

pub struct AudioController;

impl AudioController {
  pub fn get_master_volume() -> Result<f64> {
    let volume = Self::get_sink_volume()?;
    Ok(volume as f64)
  }

  pub fn set_master_volume(volume: f64) -> Result<()> {
    if volume < 0.0 || volume > 1.0 {
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
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-master")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    introspector.get_sink_info_by_index(pulse::def::PA_INVALID_INDEX, |_, _, sink| {
      if let Some(sink) = sink {
        if sink_name_tx.send(sink.name.clone().into_string()).is_err() {
          return;
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
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-get-volume")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    introspector.get_sink_info_by_name(&sink_name, |_, _, sink| {
      if let Some(sink) = sink {
        let avg_volume = sink.volume.avg().0 as f32 / pulse::volume::VolumeNorm::MAX.0 as f32;
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
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-set-volume")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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
      let sink_index = pulse::def::PA_INVALID_INDEX;
      let cv = pulse::volume::ChannelVolumes {
        channels: 2,
        volumes: [
          pulse::volume::Volume((volume * pulse::volume::VolumeNorm::MAX.0 as f32) as u32),
          pulse::volume::Volume((volume * pulse::volume::VolumeNorm::MAX.0 as f32) as u32),
        ],
      };

      context.set_sink_volume_by_name(&sink_name, &cv, None)
    };

    if operation.is_none() {
      return Err(Error::new(
        Status::GenericFailure,
        "Failed to set sink volume",
      ));
    }

    Ok(())
  }

  fn get_sink_mute() -> Result<bool> {
    let sink_name = Self::get_default_sink_name()?;
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-get-mute")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    introspector.get_sink_info_by_name(&sink_name, |_, _, sink| {
      if let Some(sink) = sink {
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
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-set-mute")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    let operation = context.set_sink_mute_by_name(&sink_name, muted, None);

    if operation.is_none() {
      return Err(Error::new(
        Status::GenericFailure,
        "Failed to set sink mute state",
      ));
    }

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
    if volume < 0.0 || volume > 1.0 {
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
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-get-apps")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    introspector.get_sink_input_info_list(|_, _, sink_input| {
      if let Some(sink_input) = sink_input {
        let app_name = sink_input
          .name
          .as_ref()
          .map(|s| s.to_string())
          .unwrap_or_else(|| "Unknown".to_string());

        let avg_volume = sink_input.volume.avg().0 as f32 / pulse::volume::VolumeNorm::MAX.0 as f32;

        let app_info = AppInfo {
          pid: sink_input.process_id,
          name: app_name,
          volume: avg_volume as f64,
          muted: sink_input.mute,
        };

        let _ = apps_tx.send(app_info);
      }
    });

    let _ = mainloop.iterate(true);
    mainloop.quit(pulse::error::Code::OK);

    let mut apps = Vec::new();

    while let Ok(app) = apps_rx.recv_timeout(std::time::Duration::from_millis(100)) {
      apps.push(app);
    }

    Ok(apps)
  }

  fn get_sink_input_volume(pid: u32) -> Result<f32> {
    let index = Self::find_sink_input_index_by_pid(pid)?;
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-get-app-volume")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    introspector.get_sink_input_info(index, |_, _, sink_input| {
      if let Some(sink_input) = sink_input {
        let avg_volume = sink_input.volume.avg().0 as f32 / pulse::volume::VolumeNorm::MAX.0 as f32;
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
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-set-app-volume")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    let cv = pulse::volume::ChannelVolumes {
      channels: 2,
      volumes: [
        pulse::volume::Volume((volume * pulse::volume::VolumeNorm::MAX.0 as f32) as u32),
        pulse::volume::Volume((volume * pulse::volume::VolumeNorm::MAX.0 as f32) as u32),
      ],
    };

    let operation = context.set_sink_input_volume(index, &cv, None);

    if operation.is_none() {
      return Err(Error::new(
        Status::GenericFailure,
        "Failed to set sink input volume",
      ));
    }

    Ok(())
  }

  fn get_sink_input_mute(pid: u32) -> Result<bool> {
    let index = Self::find_sink_input_index_by_pid(pid)?;
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-get-app-mute")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    introspector.get_sink_input_info(index, |_, _, sink_input| {
      if let Some(sink_input) = sink_input {
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
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-set-app-mute")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    let operation = context.set_sink_input_mute(index, muted, None);

    if operation.is_none() {
      return Err(Error::new(
        Status::GenericFailure,
        "Failed to set sink input mute state",
      ));
    }

    Ok(())
  }

  fn find_sink_input_index_by_pid(pid: u32) -> Result<u32> {
    let mut mainloop = Mainloop::new()?;
    let mut context = Context::new(&mut mainloop, "fa-control-find-sink-input")?;

    context.connect(None, ContextFlagSet::NOFLAGS, None)?;

    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Failure(_) => {
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

    introspector.get_sink_input_info_list(|_, _, sink_input| {
      if let Some(sink_input) = sink_input {
        if sink_input.process_id == pid {
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
