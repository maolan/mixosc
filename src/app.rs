use crate::ProbeOutcome;
use crate::message::{AppView, ConnectionStatus, Message};
use crate::model::{ConsoleUpdate, DiscoveredMixer, FaderTarget, GainSource, MixerModel};
use crate::parameters::OscValue;
use crate::state::{MAX_SEND_BUS_COUNT, MAX_STRIP_COUNT, MixOscApp, StatusApp};
use crate::ui::format::*;
use crate::ui::nav::*;
use crate::ui::panel_channel::*;
use crate::ui::panel_fx::*;
use crate::ui::panel_routing::*;
use crate::ui::panel_scenes::*;
use crate::ui::panel_setup::*;
use crate::ui::strips::*;
use crate::workers::*;
use maolan_widgets::iced::widget::{button, column, container, row, scrollable, text};
use maolan_widgets::iced::{
    Background, Border, Color, Element, Fill, Length, Subscription, Task, Theme, time,
};
use maolan_widgets::iced_fonts::lucide::audio_lines;
use std::time::Duration;

pub fn new() -> (MixOscApp, Task<Message>) {
    let maybe_target = mixer_addr_from_args_or_env();

    let mut app = MixOscApp {
        mixers: Vec::new(),
        active_mixer: 0,
        discovered_mixers: Vec::new(),
        manual_target: maybe_target.is_some(),
        discovery_in_flight: maybe_target.is_none(),
    };

    let task = match maybe_target {
        Some(addr) => {
            app.mixers.push(StatusApp {
                mixer_addr: Some(addr),
                status: ConnectionStatus::Checking,
                probe_in_flight: true,
                ..Default::default()
            });
            spawn_probe(addr).map(|m| Message::MixerEvent(0, Box::new(m)))
        }
        None => spawn_discovery(),
    };

    (app, task)
}

pub fn update(app: &mut MixOscApp, message: Message) -> Task<Message> {
    match message {
        Message::MixerEvent(index, msg) => {
            if let Some(mixer) = app.mixers.get_mut(index) {
                let task = update_mixer(mixer, *msg);
                task.map(move |m| Message::MixerEvent(index, Box::new(m)))
            } else {
                Task::none()
            }
        }
        Message::TabSelected(index) => {
            app.active_mixer = index.min(app.mixers.len().saturating_sub(1));
            Task::none()
        }
        Message::DiscoveryFinished(result) => {
            app.discovery_in_flight = false;
            match result {
                Ok(mixers) => {
                    let mut tasks = Vec::new();
                    for mixer in mixers {
                        let already_connected =
                            app.mixers.iter().any(|m| m.mixer_addr == Some(mixer.addr));
                        if !already_connected {
                            let idx = app.mixers.len();
                            app.mixers.push(StatusApp {
                                mixer_addr: Some(mixer.addr),
                                discovered_mixer: Some(mixer.clone()),
                                mixer_model: mixer.model,
                                status: ConnectionStatus::Checking,
                                probe_in_flight: true,
                                ..Default::default()
                            });
                            tasks.push(
                                spawn_probe(mixer.addr)
                                    .map(move |m| Message::MixerEvent(idx, Box::new(m))),
                            );
                        }
                    }
                    app.discovered_mixers = app
                        .mixers
                        .iter()
                        .filter_map(|m| m.discovered_mixer.clone())
                        .collect();
                    if !app.mixers.is_empty() && app.active_mixer >= app.mixers.len() {
                        app.active_mixer = 0;
                    }
                    Task::batch(tasks)
                }
                Err(_) => {
                    app.discovered_mixers.clear();
                    Task::none()
                }
            }
        }
        Message::Tick => {
            let mut tasks = Vec::new();
            for (index, mixer) in app.mixers.iter_mut().enumerate() {
                if let Some(addr) = mixer.mixer_addr
                    && !mixer.probe_in_flight
                {
                    mixer.probe_in_flight = true;
                    tasks.push(
                        spawn_probe(addr).map(move |m| Message::MixerEvent(index, Box::new(m))),
                    );
                }
            }
            if !app.manual_target && !app.discovery_in_flight && app.mixers.is_empty() {
                app.discovery_in_flight = true;
                tasks.push(spawn_discovery());
            }
            Task::batch(tasks)
        }
        Message::Disconnect => {
            if app.active_mixer < app.mixers.len() {
                app.mixers.remove(app.active_mixer);
                if app.active_mixer >= app.mixers.len() && !app.mixers.is_empty() {
                    app.active_mixer = app.mixers.len() - 1;
                }
            }
            Task::none()
        }
        Message::MixerSelected(addr) => {
            if let Some(idx) = app.mixers.iter().position(|m| m.mixer_addr == Some(addr)) {
                app.active_mixer = idx;
                Task::none()
            } else if let Some(discovered) = app
                .discovered_mixers
                .iter()
                .find(|m| m.addr == addr)
                .cloned()
            {
                let idx = app.mixers.len();
                app.mixers.push(StatusApp {
                    mixer_addr: Some(addr),
                    discovered_mixer: Some(discovered.clone()),
                    mixer_model: discovered.model,
                    status: ConnectionStatus::Checking,
                    probe_in_flight: true,
                    ..Default::default()
                });
                app.active_mixer = idx;
                spawn_probe(addr).map(move |m| Message::MixerEvent(idx, Box::new(m)))
            } else {
                Task::none()
            }
        }
        msg => {
            if let Some(mixer) = app.mixers.get_mut(app.active_mixer) {
                let index = app.active_mixer;
                let task = update_mixer(mixer, msg);
                task.map(move |m| Message::MixerEvent(index, Box::new(m)))
            } else {
                Task::none()
            }
        }
    }
}

pub fn update_mixer(app: &mut StatusApp, message: Message) -> Task<Message> {
    match message {
        Message::Tick => Task::none(),
        Message::MixerSelected(addr) => {
            app.mixer_addr = Some(addr);
            app.probe_in_flight = true;
            app.status = ConnectionStatus::Checking;
            app.last_error = None;
            spawn_probe(addr)
        }
        Message::Disconnect => {
            app.mixer_addr = None;
            app.discovered_mixer = None;
            app.status = ConnectionStatus::Disconnected;
            app.last_error = None;
            app.mixer_model = MixerModel::X32;
            app.names = std::array::from_fn(|_| None);
            app.colors = [None; MAX_STRIP_COUNT];
            app.gains = [None; MAX_STRIP_COUNT];
            app.gain_sources = [GainSource::Trim; MAX_STRIP_COUNT];
            app.sends = [[None; MAX_SEND_BUS_COUNT]; MAX_STRIP_COUNT];
            app.pans = [None; MAX_STRIP_COUNT];
            app.faders = [None; MAX_STRIP_COUNT];
            app.meters_db = [-90.0; MAX_STRIP_COUNT];
            app.master_meters_db = [-90.0, -90.0];
            app.rta_meters_db = [-128.0; 100];
            app.muted = [None; MAX_STRIP_COUNT];
            app.soloed = [None; MAX_STRIP_COUNT];
            app.master_fader = None;
            app.master_muted = None;
            app.master_soloed = None;
            app.master_color = None;
            app.parameter_values.clear();
            app.editing_scene = None;
            app.copy_buffer = None;
            app.dca_spill = None;
            app.mute_spill = None;
            app.show_file_name.clear();
            app.editing_scene_safes = None;
            app.editing_snippet_filters = None;
            Task::none()
        }
        Message::ParameterChanged(path, value) => {
            app.parameter_values.insert(path.clone(), value.clone());
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_set_parameter(mixer_addr, path, value)
        }
        Message::ParameterSetFinished(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            }
            Task::none()
        }
        Message::ParametersLoaded(result) => {
            match result {
                Ok(values) => {
                    for (path, value) in values {
                        app.parameter_values.insert(path, value);
                    }
                }
                Err(error) => app.last_error = Some(error),
            }
            Task::none()
        }
        Message::NameEditStarted(index) => {
            let current = app.names[index].clone().unwrap_or_default();
            app.editing_name = Some((index, current));
            Task::none()
        }
        Message::NameEditChanged(index, text) => {
            if let Some((edit_index, _)) = app.editing_name
                && edit_index == index
            {
                app.editing_name = Some((index, text));
            }
            Task::none()
        }
        Message::NameEditSubmitted(index) => {
            let Some((edit_index, name)) = app.editing_name.take() else {
                return Task::none();
            };
            if edit_index != index {
                return Task::none();
            }
            app.names[index] = if name.trim().is_empty() {
                None
            } else {
                Some(name.clone())
            };
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let target = app.visible_strips()[index];
            let base = strip_base_path(target, app.mixer_model);
            let path = format!("{base}/config/name");
            spawn_set_parameter(mixer_addr, path, OscValue::String(name))
        }
        Message::ConsoleUpdateReceived(result) => {
            match result {
                Ok(ConsoleUpdate::Gain(strip)) => {
                    if let Some(index) = app
                        .visible_strips()
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        let keep_headamp_source = matches!(
                            (
                                app.visible_strips()[index],
                                app.gain_sources[index],
                                strip.source
                            ),
                            (
                                FaderTarget::Channel(1..=16),
                                GainSource::Headamp(_),
                                GainSource::Trim
                            )
                        );

                        if !keep_headamp_source {
                            app.gains[index] = Some(strip.value);
                            app.gain_sources[index] = strip.source;
                        }
                    }
                }
                Ok(ConsoleUpdate::HeadampGain {
                    index: headamp_index,
                    value,
                }) => {
                    for strip_index in 0..MAX_STRIP_COUNT {
                        if app.gain_sources[strip_index] == GainSource::Headamp(headamp_index) {
                            app.gains[strip_index] = Some(value);
                        }
                    }
                }
                Ok(ConsoleUpdate::Fader(strip)) => {
                    if strip.target == FaderTarget::Main {
                        app.master_fader = Some(strip.value);
                        return Task::none();
                    }
                    if let Some(index) = app
                        .visible_strips()
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.faders[index] = Some(strip.value);
                    }
                }
                Ok(ConsoleUpdate::Pan(strip)) => {
                    if let Some(index) = app
                        .visible_strips()
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.pans[index] = Some(strip.value);
                    }
                }
                Ok(ConsoleUpdate::Send(strip)) => {
                    if let Some(strip_index) = app
                        .visible_strips()
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        let bus_index = usize::from(strip.bus.saturating_sub(1));
                        if let Some(send) = app.sends[strip_index].get_mut(bus_index) {
                            *send = Some(strip.value);
                        }
                    }
                }
                Ok(ConsoleUpdate::Mute(strip)) => {
                    if strip.target == FaderTarget::Main {
                        app.master_muted = Some(!strip.on);
                        return Task::none();
                    }
                    if let Some(index) = app
                        .visible_strips()
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.muted[index] = Some(!strip.on);
                    }
                }
                Ok(ConsoleUpdate::Solo(strip)) => {
                    if let Some(index) = app
                        .visible_strips()
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.soloed[index] = Some(strip.on);
                    }
                }
                Ok(ConsoleUpdate::Name(strip)) => {
                    if let Some(index) = app
                        .visible_strips()
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.names[index] = if strip.value.trim().is_empty() {
                            None
                        } else {
                            Some(strip.value)
                        };
                    }
                }
                Ok(ConsoleUpdate::Color(strip)) => {
                    if strip.target == FaderTarget::Main {
                        app.master_color = Some(strip.value);
                        return Task::none();
                    }
                    if let Some(index) = app
                        .visible_strips()
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.colors[index] = Some(strip.value);
                    }
                }
                Ok(ConsoleUpdate::Parameter { path, value }) => {
                    app.parameter_values.insert(path, value);
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::GainChanged(index, value) => {
            let source = app.gain_sources[index];
            let value = quantize_gain_value(value, source);
            if let Some(drag_value) = app.gain_drag_values.get_mut(index) {
                *drag_value = Some(value);
            }
            if let Some(gain) = app.gains.get_mut(index) {
                *gain = Some(value);
            }

            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let target = app.visible_strips()[index];
            spawn_set_gain(mixer_addr, target, source, value, app.mixer_model)
        }
        Message::GainReleased(index) => {
            if let Some(Some(value)) = app.gain_drag_values.get(index).copied()
                && let Some(gain) = app.gains.get_mut(index)
            {
                *gain = Some(value);
            }
            if let Some(drag_value) = app.gain_drag_values.get_mut(index) {
                *drag_value = None;
            }
            Task::none()
        }
        Message::SendChanged(strip_index, bus_index, value) => {
            if let Some(send) = app.sends[strip_index].get_mut(bus_index) {
                *send = Some(value);
            }

            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let target = app.visible_strips()[strip_index];
            let bus = app.send_buses()[bus_index];
            spawn_set_send(mixer_addr, target, bus, value, app.mixer_model)
        }
        Message::PanChanged(index, value) => {
            if let Some(pan) = app.pans.get_mut(index) {
                *pan = Some(value);
            }

            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let target = app.visible_strips()[index];
            spawn_set_pan(mixer_addr, target, value, app.mixer_model)
        }
        Message::FaderChanged(index, value) => {
            if let Some(fader) = app.faders.get_mut(index) {
                *fader = Some(value);
            }

            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let target = app.visible_strips()[index];
            spawn_set_fader(mixer_addr, target, value, app.mixer_model)
        }
        Message::MasterFaderChanged(value) => {
            app.master_fader = Some(value);

            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_set_fader(mixer_addr, FaderTarget::Main, value, app.mixer_model)
        }
        Message::NavSelected(view) => {
            app.active_view = view;
            if let Some(mixer_addr) = app.mixer_addr
                && let Some(task) = spawn_load_panel_parameters(app, mixer_addr)
            {
                return task;
            }
            Task::none()
        }
        Message::StripSelected(selected) => {
            app.selected_strip = Some(selected);
            if let Some(mixer_addr) = app.mixer_addr
                && let Some(task) = spawn_load_panel_parameters(app, mixer_addr)
            {
                return task;
            }
            Task::none()
        }
        Message::NamesLoaded(result) => {
            match result {
                Ok(names) => {
                    for strip in names {
                        if let Some(index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == strip.target)
                        {
                            app.names[index] = if strip.value.is_empty() {
                                None
                            } else {
                                Some(strip.value)
                            };
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::ColorsLoaded(result) => {
            match result {
                Ok(colors) => {
                    for strip in colors {
                        if strip.target == FaderTarget::Main {
                            app.master_color = Some(strip.value);
                            continue;
                        }
                        if let Some(index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == strip.target)
                        {
                            app.colors[index] = Some(strip.value);
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::GainsLoaded(result) => {
            match result {
                Ok(gains) => {
                    for strip in gains {
                        if let Some(index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == strip.target)
                        {
                            app.gains[index] = Some(strip.value);
                            app.gain_sources[index] = strip.source;
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::SendsLoaded(result) => {
            match result {
                Ok(sends) => {
                    for strip in sends {
                        if let Some(strip_index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == strip.target)
                        {
                            let bus_index = usize::from(strip.bus.saturating_sub(1));
                            if let Some(send) = app.sends[strip_index].get_mut(bus_index) {
                                *send = Some(strip.value);
                            }
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::PansLoaded(result) => {
            match result {
                Ok(pans) => {
                    for strip in pans {
                        if let Some(index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == strip.target)
                        {
                            app.pans[index] = Some(strip.value);
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::FadersLoaded(result) => {
            match result {
                Ok(faders) => {
                    for fader in faders {
                        if fader.target == FaderTarget::Main {
                            app.master_fader = Some(fader.value);
                            continue;
                        }
                        if let Some(index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == fader.target)
                        {
                            app.faders[index] = Some(fader.value);
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::FaderSetFinished(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            }

            Task::none()
        }
        Message::PanSetFinished(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            }

            Task::none()
        }
        Message::SendSetFinished(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            }

            Task::none()
        }
        Message::GainSetFinished(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            }

            Task::none()
        }
        Message::MetersLoaded(result) => {
            match result {
                Ok(meters) => {
                    for meter in meters {
                        if let Some(index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == meter.target)
                        {
                            app.meters_db[index] = linear_meter_to_db(meter.level_linear);
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::MasterMetersLoaded(result) => {
            match *result {
                Ok(levels) => {
                    app.master_meters_db = [
                        linear_meter_to_db(levels.main_lr[0]),
                        linear_meter_to_db(levels.main_lr[1]),
                    ];
                    for (bus_index, level) in levels.mains.iter().enumerate() {
                        if let Some(strip_index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == FaderTarget::Bus((bus_index + 1) as u8))
                        {
                            app.meters_db[strip_index] = linear_meter_to_db(*level);
                        }
                    }
                    for (matrix_index, level) in levels.matrices.iter().enumerate() {
                        if let Some(strip_index) = app.visible_strips().iter().position(|target| {
                            *target == FaderTarget::Mtx((matrix_index + 1) as u8)
                        }) {
                            app.meters_db[strip_index] = linear_meter_to_db(*level);
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::RtaMetersLoaded(result) => {
            match *result {
                Ok(levels) => {
                    app.rta_meters_db = levels;
                }
                Err(error) => app.last_error = Some(error),
            }
            Task::none()
        }
        Message::MutePressed(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let target = app.visible_strips()[index];
            let currently_muted = app
                .muted
                .get(index)
                .and_then(|state| *state)
                .unwrap_or(false);
            let next_on = currently_muted;
            if let Some(muted) = app.muted.get_mut(index) {
                *muted = Some(!next_on);
            }
            spawn_set_mute(mixer_addr, target, next_on, app.mixer_model)
        }
        Message::MasterMutePressed => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let currently_muted = app.master_muted.unwrap_or(false);
            let next_on = currently_muted;
            app.master_muted = Some(!next_on);
            spawn_set_mute(mixer_addr, FaderTarget::Main, next_on, app.mixer_model)
        }
        Message::MutesLoaded(result) => {
            match result {
                Ok(mutes) => {
                    for strip in mutes {
                        if strip.target == FaderTarget::Main {
                            app.master_muted = Some(!strip.on);
                            continue;
                        }
                        if let Some(index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == strip.target)
                        {
                            app.muted[index] = Some(!strip.on);
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::MuteSetFinished(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            }

            Task::none()
        }
        Message::SoloPressed(index) => {
            let target = app.visible_strips()[index];
            let next_on = !app
                .soloed
                .get(index)
                .and_then(|state| *state)
                .unwrap_or(false);
            if let Some(soloed) = app.soloed.get_mut(index) {
                *soloed = Some(next_on);
            }
            if matches!(target, FaderTarget::Mtx(_) | FaderTarget::Dca(_)) {
                return Task::none();
            }
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_set_solo(mixer_addr, target, next_on, app.mixer_model)
        }
        Message::MasterSoloPressed => {
            let next_on = !app.master_soloed.unwrap_or(false);
            app.master_soloed = Some(next_on);
            Task::none()
        }
        Message::SolosLoaded(result) => {
            match result {
                Ok(solos) => {
                    for strip in solos {
                        if let Some(index) = app
                            .visible_strips()
                            .iter()
                            .position(|target| *target == strip.target)
                        {
                            app.soloed[index] = Some(strip.on);
                        }
                    }
                }
                Err(error) => app.last_error = Some(error),
            }

            Task::none()
        }
        Message::SoloSetFinished(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            }

            Task::none()
        }
        Message::DiscoveryFinished(_) => Task::none(),
        Message::SceneRecall(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            match app.mixer_model {
                MixerModel::X32 => spawn_scene_action(mixer_addr, "goscene", index),
                MixerModel::XR18 => spawn_snapshot_action(mixer_addr, "load", index),
            }
        }
        Message::SceneSave(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            match app.mixer_model {
                MixerModel::X32 => spawn_scene_action(mixer_addr, "savescene", index),
                MixerModel::XR18 => spawn_snapshot_action(mixer_addr, "save", index),
            }
        }
        Message::SnippetRecall(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_scene_action(mixer_addr, "gosnippet", index)
        }
        Message::SnippetSave(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_scene_action(mixer_addr, "savesnippet", index)
        }
        Message::RecorderAction(action) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_recorder_action(mixer_addr, action)
        }
        Message::CopyStrip(index) => {
            app.copy_buffer = Some(app.visible_strips()[index]);
            Task::none()
        }
        Message::PasteStrip(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let Some(source) = app.copy_buffer else {
                return Task::none();
            };
            let target = app.visible_strips()[index];
            spawn_copy_paste(mixer_addr, source, target)
        }
        Message::DcaSpill(dca) => {
            if app.dca_spill == Some(dca) {
                app.dca_spill = None;
            } else {
                app.dca_spill = Some(dca);
                app.mute_spill = None;
            }
            Task::none()
        }
        Message::MuteSpill(grp) => {
            if app.mute_spill == Some(grp) {
                app.mute_spill = None;
            } else {
                app.mute_spill = Some(grp);
                app.dca_spill = None;
            }
            Task::none()
        }
        Message::ClearSpill => {
            app.dca_spill = None;
            app.mute_spill = None;
            Task::none()
        }
        Message::ShowFileNameChanged(name) => {
            app.show_file_name = name;
            Task::none()
        }
        Message::ShowFileLoad => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let name = app.show_file_name.clone();
            Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(2000))
                        .set("/-show/showfile/load", OscValue::String(name))
                        .map_err(|error| error.to_string())
                },
                Message::ParameterSetFinished,
            )
        }
        Message::ShowFileSave => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let name = app.show_file_name.clone();
            Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(2000))
                        .set("/-show/showfile/save", OscValue::String(name))
                        .map_err(|error| error.to_string())
                },
                Message::ParameterSetFinished,
            )
        }
        Message::ClearSolo => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(500))
                        .set("/-action/clearsolo", OscValue::Int(1))
                        .map_err(|error| error.to_string())
                },
                Message::ParameterSetFinished,
            )
        }
        Message::Undo => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(500))
                        .set("/-action/doundo", OscValue::Int(1))
                        .map_err(|error| error.to_string())
                },
                Message::ParameterSetFinished,
            )
        }
        Message::EditSceneSafes(scene) => {
            if app.editing_scene_safes == Some(scene) {
                app.editing_scene_safes = None;
            } else {
                app.editing_scene_safes = Some(scene);
            }
            Task::none()
        }
        Message::EditSnippetFilters(snip) => {
            if app.editing_snippet_filters == Some(snip) {
                app.editing_snippet_filters = None;
                Task::none()
            } else {
                app.editing_snippet_filters = Some(snip);
                if let Some(mixer_addr) = app.mixer_addr {
                    return spawn_fetch_snippet_filters(mixer_addr, snip);
                }
                Task::none()
            }
        }
        Message::MixerEvent(_, _) => Task::none(),
        Message::TabSelected(_) => Task::none(),
        Message::ProbeFinished(result) => {
            app.probe_in_flight = false;
            let was_connected = matches!(app.status, ConnectionStatus::Connected(_));

            match result {
                Ok(ProbeOutcome::Connected {
                    response, model, ..
                }) => {
                    app.status = ConnectionStatus::Connected(response);
                    if let Some(detected) = model {
                        app.mixer_model = detected;
                    }
                    if !was_connected && let Some(mixer_addr) = app.mixer_addr {
                        let model = app.mixer_model;
                        return Task::batch([
                            spawn_load_names(mixer_addr, model),
                            spawn_load_colors(mixer_addr, model),
                            spawn_load_gains(mixer_addr, model),
                            spawn_load_sends(mixer_addr, model),
                            spawn_load_pans(mixer_addr, model),
                            spawn_load_faders(mixer_addr, model),
                            spawn_load_mutes(mixer_addr, model),
                            spawn_load_solos(mixer_addr, model),
                            spawn_load_mute_groups(mixer_addr, model),
                        ]);
                    }
                }
                Ok(ProbeOutcome::Disconnected) => {
                    app.status = ConnectionStatus::Disconnected;
                    app.last_error = None;
                    app.names = std::array::from_fn(|_| None);
                    app.gains = [None; MAX_STRIP_COUNT];
                    app.gain_sources = [GainSource::Trim; MAX_STRIP_COUNT];
                    app.sends = [[None; MAX_SEND_BUS_COUNT]; MAX_STRIP_COUNT];
                    app.pans = [None; MAX_STRIP_COUNT];
                    app.faders = [None; MAX_STRIP_COUNT];
                    app.meters_db = [-90.0; MAX_STRIP_COUNT];
                    app.master_meters_db = [-90.0, -90.0];
                    app.muted = [None; MAX_STRIP_COUNT];
                    app.soloed = [None; MAX_STRIP_COUNT];
                    app.master_fader = None;
                    app.master_muted = None;
                    app.master_soloed = None;
                    app.master_color = None;
                }
                Err(error) => {
                    app.status = ConnectionStatus::Disconnected;
                    app.last_error = Some(error);
                    app.names = std::array::from_fn(|_| None);
                    app.gains = [None; MAX_STRIP_COUNT];
                    app.gain_sources = [GainSource::Trim; MAX_STRIP_COUNT];
                    app.sends = [[None; MAX_SEND_BUS_COUNT]; MAX_STRIP_COUNT];
                    app.pans = [None; MAX_STRIP_COUNT];
                    app.faders = [None; MAX_STRIP_COUNT];
                    app.meters_db = [-90.0; MAX_STRIP_COUNT];
                    app.master_meters_db = [-90.0, -90.0];
                    app.muted = [None; MAX_STRIP_COUNT];
                    app.soloed = [None; MAX_STRIP_COUNT];
                    app.master_fader = None;
                    app.master_muted = None;
                    app.master_soloed = None;
                    app.master_color = None;
                }
            }

            Task::none()
        }
    }
}

pub fn subscription(app: &MixOscApp) -> Subscription<Message> {
    let ticker = time::every(Duration::from_secs(3)).map(|_| Message::Tick);
    let mut subs = vec![ticker];

    for (index, mixer) in app.mixers.iter().enumerate() {
        if let Some(addr) = mixer.mixer_addr {
            subs.push(state_subscription(addr, mixer.mixer_model, index));
            subs.push(meter_subscription(addr, mixer.mixer_model, index));
            subs.push(master_meter_subscription(addr, mixer.mixer_model, index));
            if mixer.mixer_model == MixerModel::X32 {
                subs.push(rta_meter_subscription(addr, mixer.mixer_model, index));
            }
        }
    }

    Subscription::batch(subs)
}

pub fn theme(_app: &MixOscApp) -> Theme {
    Theme::TokyoNight
}

pub fn view(app: &MixOscApp) -> Element<'_, Message> {
    if app.mixers.is_empty() {
        return mixer_selection_view(&app.discovered_mixers, app.discovery_in_flight, None);
    }

    let tabs = mixer_tabs(app);
    let active = &app.mixers[app.active_mixer];
    let mixer_content = view_mixer(active);

    column![tabs, mixer_content]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_mixer(app: &StatusApp) -> Element<'_, Message> {
    let content: Element<'_, Message> = if matches!(app.status, ConnectionStatus::Connected(_)) {
        let mixer_view: Element<'_, Message> = if let Some(panel) = top_detail_panel(app) {
            column![panel, mixer_strips(app)]
                .spacing(0)
                .height(Length::Fill)
                .into()
        } else {
            mixer_strips(app)
        };

        container(mixer_view)
            .padding([0, 16])
            .height(Length::Fill)
            .into()
    } else {
        mixer_selection_view(&[], false, app.last_error.as_deref())
    };

    let body = if matches!(app.status, ConnectionStatus::Connected(_)) {
        column![
            scrollable(
                container(top_nav_bar(app))
                    .padding([0, 16])
                    .width(Length::Shrink)
            )
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new().width(2).scroller_width(4),
            )),
            container(spill_bar(app))
                .padding([2, 16])
                .width(Length::Shrink),
            content
        ]
        .spacing(0)
        .into()
    } else {
        content
    };

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn mixer_selection_view<'a>(
    discovered_mixers: &'a [DiscoveredMixer],
    searching: bool,
    last_error: Option<&'a str>,
) -> Element<'a, Message> {
    let title = text("Discovered Mixers").size(28);

    let mixer_list: Element<'_, Message> = if discovered_mixers.is_empty() {
        if searching {
            text("Searching for mixers on the network...")
                .size(16)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3))
                .into()
        } else if let Some(error) = last_error {
            text(format!("Error: {error}"))
                .size(14)
                .color(Color::from_rgb8(0xF0, 0x7C, 0x82))
                .into()
        } else {
            text("No mixers found on the local network.")
                .size(16)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3))
                .into()
        }
    } else {
        let mixers = discovered_mixers.iter().fold(
            column!().spacing(8).width(Length::Fill),
            |col, mixer| {
                let name = mixer.name.as_deref().unwrap_or("Unknown Mixer");
                let ip = mixer.addr.ip().to_string();
                let detail = match (&mixer.model, &mixer.firmware) {
                    (model, Some(firmware)) => {
                        format!("{model} · firmware {firmware}")
                    }
                    (model, None) => model.to_string(),
                };

                let icon = audio_lines().size(24);
                let name_text = text(name).size(18);
                let ip_text = text(ip).size(14).color(Color::from_rgb8(0xC7, 0xC9, 0xD3));
                let detail_text = if detail.is_empty() {
                    None
                } else {
                    Some(
                        text(detail)
                            .size(12)
                            .color(Color::from_rgb8(0x80, 0x80, 0x80)),
                    )
                };

                let info_col = if let Some(detail) = detail_text {
                    column![name_text, ip_text, detail].spacing(2)
                } else {
                    column![name_text, ip_text].spacing(2)
                };

                let row = row![icon, info_col]
                    .spacing(12)
                    .align_y(maolan_widgets::iced::Alignment::Center);

                let btn = button(container(row).padding([8, 12]))
                    .on_press(Message::MixerSelected(mixer.addr))
                    .width(Length::Fill)
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x24, 0x26, 0x2F))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x2A, 0x2A, 0x2A),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    });

                col.push(btn)
            },
        );
        scrollable(mixers).into()
    };

    let content = column![title, mixer_list]
        .spacing(16)
        .width(Length::Fill)
        .max_width(480);

    container(content)
        .padding([24, 16])
        .center_x(Fill)
        .center_y(Fill)
        .into()
}

fn mixer_tabs(app: &MixOscApp) -> Element<'_, Message> {
    let tabs = app.mixers.iter().enumerate().fold(
        row!().spacing(4).padding([4, 8]),
        |row, (index, mixer)| {
            let label = mixer
                .discovered_mixer
                .as_ref()
                .and_then(|m| m.name.clone())
                .or_else(|| mixer.mixer_addr.map(|a| a.ip().to_string()))
                .unwrap_or_else(|| format!("Mixer {}", index + 1));
            let is_active = index == app.active_mixer;
            let bg = if is_active {
                Color::from_rgb8(0x2A, 0x2A, 0x2A)
            } else {
                Color::from_rgb8(0x1C, 0x1C, 0x1C)
            };
            let border_color = if is_active {
                Color::from_rgb8(0x4B, 0x4B, 0x4B)
            } else {
                Color::from_rgb8(0x3A, 0x3A, 0x3A)
            };
            let text_color = if is_active {
                Color::from_rgb8(0x29, 0xE6, 0xF2)
            } else {
                Color::from_rgb8(0xA9, 0xAC, 0xB3)
            };
            row.push(
                button(text(label).size(13))
                    .padding([6, 14])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(bg)),
                            text_color,
                            border: Border {
                                color: border_color,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        },
                    )
                    .on_press(Message::TabSelected(index)),
            )
        },
    );

    container(tabs)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x14, 0x14, 0x14))),
            border: Border {
                color: Color::from_rgb8(0x2A, 0x2A, 0x2A),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn top_detail_panel(app: &StatusApp) -> Option<Element<'_, Message>> {
    match app.active_view {
        AppView::Mixer => None,
        AppView::Channel => Some(channel_detail_panel(app)),
        AppView::Config => Some(config_detail_panel(app)),
        AppView::Gate => Some(gate_detail_panel(app)),
        AppView::Dyn => Some(dyn_detail_panel(app)),
        AppView::Eq => Some(eq_detail_panel(app)),
        AppView::Sends => Some(sends_detail_panel(app)),
        AppView::Main => Some(main_detail_panel(app)),
        AppView::Fx => Some(fx_detail_panel(app)),
        AppView::Scenes => Some(scenes_detail_panel(app)),
        AppView::Setup => Some(setup_detail_panel(app)),
        AppView::Routing => Some(routing_detail_panel(app)),
        AppView::Rta => Some(rta_detail_panel(app)),
    }
}
