use libpulse_binding as pulse;
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::{Context, FlagSet as ContextFlagSets, State as ContextState};
use libpulse_binding::def::Retval;
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use libpulse_binding::operation::State as OperationState;
use libpulse_binding::proplist::properties::APPLICATION_PROCESS_ID;
use libpulse_binding::volume::ChannelVolumes;

use crate::AppInfo;

pub struct AudioController;

impl AudioController {
    pub fn get_master_volume() -> Result<f64, String> {
        let volume = Self::get_sink_volume()?;
        Ok(volume as f64)
    }

    pub fn set_master_volume(volume: f64) -> Result<(), String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }

        Self::set_sink_volume(volume as f32)?;
        Ok(())
    }

    pub fn is_master_muted() -> Result<bool, String> {
        let muted = Self::get_sink_mute()?;
        Ok(muted)
    }

    pub fn toggle_master_mute() -> Result<bool, String> {
        let current_muted = Self::is_master_muted()?;
        Self::set_sink_mute(!current_muted)?;
        Ok(!current_muted)
    }

    pub fn set_master_mute(muted: bool) -> Result<(), String> {
        Self::set_sink_mute(muted)?;
        Ok(())
    }

    fn get_default_sink_name() -> Result<String, String> {
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-master")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                    return Err("Mainloop error".to_string());
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
        Err("No sink found".to_string())
    }

    fn get_sink_volume() -> Result<f32, String> {
        let sink_name = Self::get_default_sink_name()?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-get-volume")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                let avg_volume =
                    sink.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;
                let _ = volume_tx.send(avg_volume);
            }
        });

        loop {
            match mainloop.iterate(true) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Mainloop error".to_string());
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

        Err("Timeout getting sink volume (No result)".to_string())
    }

    fn set_sink_volume(volume: f32) -> Result<(), String> {
        let sink_name = Self::get_default_sink_name()?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-set-volume")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                    return Err("Mainloop error".to_string());
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

    fn get_sink_mute() -> Result<bool, String> {
        let sink_name = Self::get_default_sink_name()?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-get-mute")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                    return Err("Mainloop error".to_string());
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

        Err("Timeout getting sink mute state".to_string())
    }

    fn set_sink_mute(muted: bool) -> Result<(), String> {
        let sink_name = Self::get_default_sink_name()?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-set-mute")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                    return Err("Mainloop error".to_string());
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
    pub fn get_app_volume(pid: u32) -> Result<f64, String> {
        let volume = Self::get_sink_input_volume(pid)?;
        Ok(volume as f64)
    }

    pub fn set_app_volume(pid: u32, volume: f64) -> Result<bool, String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }

        Self::set_sink_input_volume(pid, volume as f32)?;
        Ok(true)
    }

    pub fn is_app_muted(pid: u32) -> Result<bool, String> {
        let muted = Self::get_sink_input_mute(pid)?;
        Ok(muted)
    }

    pub fn set_app_mute(pid: u32, muted: bool) -> Result<(), String> {
        Self::set_sink_input_mute(pid, muted)?;
        Ok(())
    }

    pub fn get_active_audio_apps() -> Result<Vec<AppInfo>, String> {
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-get-apps")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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

                let avg_volume =
                    sink_input.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;

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
                    return Err("Mainloop error".to_string());
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

    fn get_sink_input_volume(pid: u32) -> Result<f32, String> {
        let index = Self::find_sink_input_index_by_pid(pid)?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-get-app-volume")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                let avg_volume =
                    sink_input.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;
                let _ = volume_tx.send(avg_volume);
            }
        });

        loop {
            match mainloop.iterate(true) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Mainloop error".to_string());
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

        Err("Timeout getting sink input volume".to_string())
    }

    fn set_sink_input_volume(pid: u32, volume: f32) -> Result<(), String> {
        let index = Self::find_sink_input_index_by_pid(pid)?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-set-app-volume")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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

        loop {
            match mainloop.iterate(true) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Mainloop error".to_string());
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

    fn get_sink_input_mute(pid: u32) -> Result<bool, String> {
        let index = Self::find_sink_input_index_by_pid(pid)?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-get-app-mute")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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

        loop {
            match mainloop.iterate(true) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Mainloop error".to_string());
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

        Err("Timeout getting sink input mute state".to_string())
    }

    fn set_sink_input_mute(pid: u32, muted: bool) -> Result<(), String> {
        let index = Self::find_sink_input_index_by_pid(pid)?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-set-app-mute")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
                }
                IterateResult::Success(_) => {}
            }

            if let ContextState::Ready = context.get_state() {
                break;
            }
        }

        let operation = context.introspect().set_sink_input_mute(index, muted, None);

        loop {
            match mainloop.iterate(true) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Mainloop error".to_string());
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

    fn find_sink_input_index_by_pid(pid: u32) -> Result<u32, String> {
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-find-sink-input")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                // Extract PID from proplist
                let pid_val = sink_input
                    .proplist
                    .get_str(APPLICATION_PROCESS_ID)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);

                // Check if this is the process we're looking for
                if pid_val == pid {
                    let _ = index_tx.send(sink_input.index);
                }
            }
        });

        let start_time = std::time::Instant::now();
        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Mainloop error".to_string());
                }
                _ => {}
            }

            if let Ok(index) = index_rx.try_recv() {
                return Ok(index);
            }

            match operation.get_state() {
                OperationState::Done | OperationState::Cancelled => break,
                _ => {}
            }

            if start_time.elapsed() > std::time::Duration::from_secs(2) {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        Err(format!("Sink input with PID {} not found", pid))
    }
}

pub struct InputController;

impl InputController {
    pub fn get_microphone_volume() -> Result<f64, String> {
        let volume = Self::get_source_volume()?;
        Ok(volume as f64)
    }

    pub fn set_microphone_volume(volume: f64) -> Result<(), String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }

        Self::set_source_volume(volume as f32)?;
        Ok(())
    }

    pub fn is_microphone_muted() -> Result<bool, String> {
        let muted = Self::get_source_mute()?;
        Ok(muted)
    }

    pub fn toggle_microphone_mute() -> Result<bool, String> {
        let current_muted = Self::is_microphone_muted()?;
        Self::set_source_mute(!current_muted)?;
        Ok(!current_muted)
    }

    pub fn set_microphone_mute(muted: bool) -> Result<(), String> {
        Self::set_source_mute(muted)?;
        Ok(())
    }

    fn get_default_source_name() -> Result<String, String> {
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-source")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                    return Err("Mainloop error".to_string());
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
        Err("No source found".to_string())
    }

    fn get_source_volume() -> Result<f32, String> {
        let source_name = Self::get_default_source_name()?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-get-source-volume")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                let avg_volume =
                    source.volume.avg().0 as f32 / pulse::volume::Volume::NORMAL.0 as f32;
                let _ = volume_tx.send(avg_volume);
            }
        });

        loop {
            match mainloop.iterate(true) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Mainloop error".to_string());
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

        Err("Timeout getting source volume".to_string())
    }

    fn set_source_volume(volume: f32) -> Result<(), String> {
        let source_name = Self::get_default_source_name()?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-set-source-volume")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                    return Err("Mainloop error".to_string());
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

    fn get_source_mute() -> Result<bool, String> {
        let source_name = Self::get_default_source_name()?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-get-source-mute")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                    return Err("Mainloop error".to_string());
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

        Err("Timeout getting source mute state".to_string())
    }

    fn set_source_mute(muted: bool) -> Result<(), String> {
        let source_name = Self::get_default_source_name()?;
        let mut mainloop =
            Mainloop::new().ok_or_else(|| "Failed to create mainloop".to_string())?;
        let mut context = Context::new(&mainloop, "fa-control-set-source-mute")
            .ok_or_else(|| "Failed to create context".to_string())?;

        context
            .connect(None, ContextFlagSets::empty(), None)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    return Err("Failed to iterate pulseaudio mainloop".to_string());
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
                    return Err("Mainloop error".to_string());
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
