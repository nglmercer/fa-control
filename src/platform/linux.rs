use crate::AppInfo;
use libpulse_binding as pulse;
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::{Context, FlagSet as ContextFlagSet, State as ContextState};
use libpulse_binding::def::Retval;
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use libpulse_binding::operation::State as OperationState;
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

    // Fallback strategy: Get ANY sink if default server info hangs
    let operation = introspector.get_sink_info_list(move |result| {
      if let ListResult::Item(sink) = result {
        if let Some(name) = sink.name.as_ref() {
          let _ = sink_name_tx.send(name.to_string());
        }
      }
    });

    // Iterate until operation is done
    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(name) = sink_name_rx.try_recv() {
        return Ok(name);
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }
    }

    if let Ok(name) = sink_name_rx.try_recv() {
      return Ok(name);
    }
    Err(Error::new(Status::GenericFailure, "No sink found"))
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

    let operation = introspector.get_sink_info_by_name(&sink_name, move |result| {
      if let ListResult::Item(sink) = result {
        let avg_volume = sink.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;
        let _ = volume_tx.send(avg_volume);
      }
    });

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(vol) = volume_rx.try_recv() {
        return Ok(vol);
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }
    }

    if let Ok(vol) = volume_rx.try_recv() {
      return Ok(vol);
    }

    Err(Error::new(
      Status::GenericFailure,
      "Timeout getting sink volume (No result)",
    ))
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

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        OperationState::Running => {}
      }
    }

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

    let operation = introspector.get_sink_info_by_name(&sink_name, move |result| {
      if let ListResult::Item(sink) = result {
        let _ = mute_tx.send(sink.mute);
      }
    });

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(muted) = mute_rx.try_recv() {
        return Ok(muted);
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }
    }

    if let Ok(muted) = mute_rx.try_recv() {
      return Ok(muted);
    }

    Err(Error::new(
      Status::GenericFailure,
      "Timeout getting sink mute state",
    ))
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

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        OperationState::Running => {}
      }
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

  pub fn set_app_volume(pid: u32, volume: f64) -> Result<bool> {
    if !(0.0..=1.0).contains(&volume) {
      return Err(Error::new(
        Status::InvalidArg,
        "Volume must be between 0.0 and 1.0",
      ));
    }

    Self::set_sink_input_volume(pid, volume as f32)?;
    Ok(true)
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
    let apps_tx_clone = apps_tx.clone();

    let operation = introspector.get_sink_input_info_list(move |result| {
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

        // If no PID, use the sink input index as a fallback identifier
        // This allows us to control streams that don't have a PID set
        let final_pid = if pid_val == 0 {
          sink_input.index
        } else {
          pid_val
        };

        let avg_volume = sink_input.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;

        let app_info = AppInfo {
          pid: final_pid,
          name: app_name,
          volume: avg_volume as f64,
          muted: sink_input.mute,
        };

        // Send all sink inputs, not just those with PIDs
        let _ = apps_tx_clone.send(app_info);
      }
    });

    drop(apps_tx);

    // Wait for the operation to complete
    let start_time = std::time::Instant::now();
    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }

      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }

      // Timeout after 2 seconds
      if start_time.elapsed() > std::time::Duration::from_secs(2) {
        break;
      }

      std::thread::sleep(std::time::Duration::from_millis(10));
    }

    mainloop.quit(Retval(0));

    let mut apps = Vec::new();

    // Drain all messages from channel with longer timeout
    while let Ok(app) = apps_rx.recv_timeout(std::time::Duration::from_millis(500)) {
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

    let operation = introspector.get_sink_input_info(index, move |result| {
      if let ListResult::Item(sink_input) = result {
        let avg_volume = sink_input.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;
        let _ = volume_tx.send(avg_volume);
      }
    });

    // Wait for the operation to complete
    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(vol) = volume_rx.try_recv() {
        return Ok(vol);
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }
    }

    if let Ok(vol) = volume_rx.try_recv() {
      return Ok(vol);
    }

    Err(Error::new(
      Status::GenericFailure,
      "Timeout getting sink input volume (No result)",
    ))
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

    let mut cv = ChannelVolumes::default();
    let vol_val = (volume * pulse::volume::Volume::NORMAL.0 as f32) as u32;
    cv.set(2, pulse::volume::Volume(vol_val));

    let operation = context.introspect().set_sink_input_volume(index, &cv, None);

    // Wait for the operation to complete
    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        OperationState::Running => {}
      }
    }

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

    let operation = introspector.get_sink_input_info(index, move |result| {
      if let ListResult::Item(sink_input) = result {
        let _ = mute_tx.send(sink_input.mute);
      }
    });

    // Wait for the operation to complete
    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(muted) = mute_rx.try_recv() {
        return Ok(muted);
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }
    }

    if let Ok(muted) = mute_rx.try_recv() {
      return Ok(muted);
    }

    Err(Error::new(
      Status::GenericFailure,
      "Timeout getting sink input mute state (No result)",
    ))
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

    // Wait for the operation to complete
    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        OperationState::Running => {}
      }
    }

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

    let operation = introspector.get_sink_input_info_list(move |result| {
      if let ListResult::Item(sink_input) = result {
        let pid_val = sink_input
          .proplist
          .get_str(APPLICATION_PROCESS_ID)
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(0);

        // Match by PID if available, otherwise match by sink input index
        // This aligns with the logic in get_active_audio_apps()
        if pid_val == pid || (pid_val == 0 && sink_input.index == pid) {
          let _ = index_tx.send(sink_input.index);
        }
      }
    });

    let start_time = std::time::Instant::now();
    loop {
      match mainloop.iterate(false) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(idx) = index_rx.try_recv() {
        return Ok(idx);
      }

      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }

      if start_time.elapsed() > std::time::Duration::from_millis(500) {
        return Err(Error::new(
          Status::GenericFailure,
          format!("No sink input found for PID: {} (Timeout)", pid),
        ));
      }
      std::thread::sleep(std::time::Duration::from_millis(10));
    }

    if let Ok(idx) = index_rx.try_recv() {
      return Ok(idx);
    }

    Err(Error::new(
      Status::GenericFailure,
      format!("No sink input found for PID: {} (Operation finished)", pid),
    ))
  }
}

/// Controller for input devices (microphones)
pub struct InputController;

impl InputController {
  /// Get microphone volume level (0.0 to 1.0)
  pub fn get_microphone_volume() -> Result<f64> {
    let volume = Self::get_source_volume()?;
    Ok(volume as f64)
  }

  /// Set microphone volume level (0.0 to 1.0)
  pub fn set_microphone_volume(volume: f64) -> Result<()> {
    if !(0.0..=1.0).contains(&volume) {
      return Err(Error::new(
        Status::InvalidArg,
        "Volume must be between 0.0 and 1.0",
      ));
    }

    Self::set_source_volume(volume as f32)?;
    Ok(())
  }

  /// Get whether microphone is muted
  pub fn is_microphone_muted() -> Result<bool> {
    let muted = Self::get_source_mute()?;
    Ok(muted)
  }

  /// Toggle microphone mute state
  pub fn toggle_microphone_mute() -> Result<bool> {
    let current_muted = Self::is_microphone_muted()?;
    Self::set_source_mute(!current_muted)?;
    Ok(!current_muted)
  }

  /// Set microphone mute state
  pub fn set_microphone_mute(muted: bool) -> Result<()> {
    Self::set_source_mute(muted)?;
    Ok(())
  }

  fn get_default_source_name() -> Result<String> {
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-input")
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
    let (source_name_tx, source_name_rx) = std::sync::mpsc::channel();

    let operation = introspector.get_source_info_list(move |result| {
      if let ListResult::Item(source) = result {
        if let Some(name) = source.name.as_ref() {
          let _ = source_name_tx.send(name.to_string());
        }
      }
    });

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(name) = source_name_rx.try_recv() {
        return Ok(name);
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }
    }

    if let Ok(name) = source_name_rx.try_recv() {
      return Ok(name);
    }
    Err(Error::new(Status::GenericFailure, "No source found"))
  }

  fn get_source_volume() -> Result<f32> {
    let source_name = Self::get_default_source_name()?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-get-source-volume")
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

    let operation = introspector.get_source_info_by_name(&source_name, move |result| {
      if let ListResult::Item(source) = result {
        let avg_volume = source.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;
        let _ = volume_tx.send(avg_volume);
      }
    });

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(vol) = volume_rx.try_recv() {
        return Ok(vol);
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }
    }

    if let Ok(vol) = volume_rx.try_recv() {
      return Ok(vol);
    }

    Err(Error::new(
      Status::GenericFailure,
      "Timeout getting source volume (No result)",
    ))
  }

  fn set_source_volume(volume: f32) -> Result<()> {
    let source_name = Self::get_default_source_name()?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-set-source-volume")
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
        .set_source_volume_by_name(&source_name, &cv, None)
    };

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        OperationState::Running => {}
      }
    }

    Ok(())
  }

  fn get_source_mute() -> Result<bool> {
    let source_name = Self::get_default_source_name()?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-get-source-mute")
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

    let operation = introspector.get_source_info_by_name(&source_name, move |result| {
      if let ListResult::Item(source) = result {
        let _ = mute_tx.send(source.mute);
      }
    });

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      if let Ok(muted) = mute_rx.try_recv() {
        return Ok(muted);
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        _ => {}
      }
    }

    if let Ok(muted) = mute_rx.try_recv() {
      return Ok(muted);
    }

    Err(Error::new(
      Status::GenericFailure,
      "Timeout getting source mute state",
    ))
  }

  fn set_source_mute(muted: bool) -> Result<()> {
    let source_name = Self::get_default_source_name()?;
    let mut mainloop = Mainloop::new()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to create mainloop"))?;
    let mut context = Context::new(&mainloop, "fa-control-set-source-mute")
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
      .set_source_mute_by_name(&source_name, muted, None);

    loop {
      match mainloop.iterate(true) {
        IterateResult::Quit(_) | IterateResult::Err(_) => {
          return Err(Error::new(Status::GenericFailure, "Mainloop error"))
        }
        _ => {}
      }
      match operation.get_state() {
        OperationState::Done | OperationState::Cancelled => break,
        OperationState::Running => {}
      }
    }

    Ok(())
  }
}

/// Controller for audio level metering
pub struct AudioLevelMeter;

impl AudioLevelMeter {
  /// Get master output audio level (0.0 to 1.0)
  pub fn get_master_audio_level() -> Result<f64> {
    // PulseAudio doesn't provide direct peak values for sinks
    // This is a limitation of the current implementation
    // For accurate level metering, you would need to monitor the audio stream
    Err(Error::new(
      Status::GenericFailure,
      "Audio level metering for sinks is not directly supported in PulseAudio. Consider using a monitoring sink or PipeWire for this functionality.",
    ))
  }

  /// Get microphone input audio level (0.0 to 1.0)
  pub fn get_microphone_audio_level() -> Result<f64> {
    // PulseAudio doesn't provide direct peak values for sources
    // This is a limitation of the current implementation
    // For accurate level metering, you would need to monitor the audio stream
    Err(Error::new(
      Status::GenericFailure,
      "Audio level metering for sources is not directly supported in PulseAudio. Consider using a monitoring source or PipeWire for this functionality.",
    ))
  }

  /// Get audio level for a specific application by PID (0.0 to 1.0)
  pub fn get_app_audio_level(pid: u32) -> Result<f64> {
    // PulseAudio doesn't provide direct peak values for sink inputs
    // This is a limitation of the current implementation
    Err(Error::new(
      Status::GenericFailure,
      "Audio level metering for applications is not directly supported in PulseAudio. Consider using PipeWire for this functionality.",
    ))
  }
}
