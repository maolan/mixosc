use crate::parameters::OscValue;
use crate::{
    ColorBankProbe, ConnectionProbe, ConsoleUpdate, DiscoveredMixer, DiscoveryProbe,
    FaderBankProbe, FaderTarget, GainBankProbe, GainSource, MainMeterLevels, MuteBankProbe,
    NameBankProbe, PanBankProbe, ProbeOutcome, ProbeResponse, SendBankProbe, SoloBankProbe,
    StripColor, StripFader, StripGain, StripMeter, StripMute, StripName, StripPan, StripSend,
    StripSolo, XREMOTE_REQUEST, batchsubscribe_meter_request, parse_console_update,
    parse_input_meter_packet, parse_main_meter_packet, parse_rta_meter_packet, parse_target,
    renew_request,
};
use iced::futures::sink::SinkExt;
use iced::futures::{StreamExt, channel::mpsc, stream::BoxStream};
use iced::stream;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Background, Border, Color, Element, Fill, Length, Subscription, Task, Theme, time};
use iced_fonts::lucide::{
    activity, audio_lines, audio_waveform, equal, file_input, git_merge, panel_left, save, send,
    settings, shield, sliders_vertical, toggle_left,
};
use maolan_widgets::horizontal_slider::horizontal_slider;
use maolan_widgets::meters::meters;
use maolan_widgets::slider::slider as vertical_slider;
use maolan_widgets::ticks::meter_ticks;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{Instant, sleep};

const STRIP_COUNT: usize = 74;
const SEND_BUS_COUNT: usize = 16;
const STRIP_METER_HEIGHT: f32 = 260.0;
const SEND_BUSES: [u8; SEND_BUS_COUNT] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const MATRIX_SENDS: [u8; 6] = [1, 2, 3, 4, 5, 6];
const VISIBLE_STRIPS: [FaderTarget; STRIP_COUNT] = [
    FaderTarget::Channel(1),
    FaderTarget::Channel(2),
    FaderTarget::Channel(3),
    FaderTarget::Channel(4),
    FaderTarget::Channel(5),
    FaderTarget::Channel(6),
    FaderTarget::Channel(7),
    FaderTarget::Channel(8),
    FaderTarget::Channel(9),
    FaderTarget::Channel(10),
    FaderTarget::Channel(11),
    FaderTarget::Channel(12),
    FaderTarget::Channel(13),
    FaderTarget::Channel(14),
    FaderTarget::Channel(15),
    FaderTarget::Channel(16),
    FaderTarget::Channel(17),
    FaderTarget::Channel(18),
    FaderTarget::Channel(19),
    FaderTarget::Channel(20),
    FaderTarget::Channel(21),
    FaderTarget::Channel(22),
    FaderTarget::Channel(23),
    FaderTarget::Channel(24),
    FaderTarget::Channel(25),
    FaderTarget::Channel(26),
    FaderTarget::Channel(27),
    FaderTarget::Channel(28),
    FaderTarget::Channel(29),
    FaderTarget::Channel(30),
    FaderTarget::Channel(31),
    FaderTarget::Channel(32),
    FaderTarget::Aux(1),
    FaderTarget::Aux(2),
    FaderTarget::Aux(3),
    FaderTarget::Aux(4),
    FaderTarget::Aux(5),
    FaderTarget::Aux(6),
    FaderTarget::Aux(7),
    FaderTarget::Aux(8),
    FaderTarget::Bus(1),
    FaderTarget::Bus(2),
    FaderTarget::Bus(3),
    FaderTarget::Bus(4),
    FaderTarget::Bus(5),
    FaderTarget::Bus(6),
    FaderTarget::Bus(7),
    FaderTarget::Bus(8),
    FaderTarget::Bus(9),
    FaderTarget::Bus(10),
    FaderTarget::Bus(11),
    FaderTarget::Bus(12),
    FaderTarget::FxRtn(1),
    FaderTarget::FxRtn(2),
    FaderTarget::FxRtn(3),
    FaderTarget::FxRtn(4),
    FaderTarget::FxRtn(5),
    FaderTarget::FxRtn(6),
    FaderTarget::FxRtn(7),
    FaderTarget::FxRtn(8),
    FaderTarget::Mtx(1),
    FaderTarget::Mtx(2),
    FaderTarget::Mtx(3),
    FaderTarget::Mtx(4),
    FaderTarget::Mtx(5),
    FaderTarget::Mtx(6),
    FaderTarget::Dca(1),
    FaderTarget::Dca(2),
    FaderTarget::Dca(3),
    FaderTarget::Dca(4),
    FaderTarget::Dca(5),
    FaderTarget::Dca(6),
    FaderTarget::Dca(7),
    FaderTarget::Dca(8),
];

#[derive(Debug)]
pub struct StatusApp {
    mixer_addr: Option<SocketAddr>,
    discovered_mixer: Option<DiscoveredMixer>,
    discovered_mixers: Vec<DiscoveredMixer>,
    manual_target: bool,
    probe_in_flight: bool,
    names: [Option<String>; STRIP_COUNT],
    colors: [Option<u8>; STRIP_COUNT],
    gains: [Option<f32>; STRIP_COUNT],
    gain_sources: [GainSource; STRIP_COUNT],
    gain_drag_values: [Option<f32>; STRIP_COUNT],
    sends: [[Option<f32>; SEND_BUS_COUNT]; STRIP_COUNT],
    pans: [Option<f32>; STRIP_COUNT],
    faders: [Option<f32>; STRIP_COUNT],
    meters_db: [f32; STRIP_COUNT],
    master_meters_db: [f32; 2],
    rta_meters_db: [f32; 100],
    muted: [Option<bool>; STRIP_COUNT],
    soloed: [Option<bool>; STRIP_COUNT],
    master_fader: Option<f32>,
    master_muted: Option<bool>,
    master_soloed: Option<bool>,
    master_color: Option<u8>,
    active_view: AppView,
    selected_strip: Option<SelectedStrip>,
    status: ConnectionStatus,
    last_error: Option<String>,
    parameter_values: std::collections::HashMap<String, OscValue>,
    editing_name: Option<(usize, String)>,
    editing_scene: Option<(usize, String)>,
    copy_buffer: Option<FaderTarget>,
    dca_spill: Option<u8>,
    mute_spill: Option<u8>,
    show_file_name: String,
    editing_scene_safes: Option<i32>,
    editing_snippet_filters: Option<i32>,
}

impl Default for StatusApp {
    fn default() -> Self {
        Self {
            mixer_addr: None,
            discovered_mixer: None,
            discovered_mixers: Vec::new(),
            manual_target: false,
            probe_in_flight: false,
            names: std::array::from_fn(|_| None),
            colors: [None; STRIP_COUNT],
            gains: [None; STRIP_COUNT],
            gain_sources: [GainSource::Trim; STRIP_COUNT],
            gain_drag_values: [None; STRIP_COUNT],
            sends: [[None; SEND_BUS_COUNT]; STRIP_COUNT],
            pans: [None; STRIP_COUNT],
            faders: [None; STRIP_COUNT],
            meters_db: [-90.0; STRIP_COUNT],
            master_meters_db: [-90.0, -90.0],
            rta_meters_db: [-128.0; 100],
            muted: [None; STRIP_COUNT],
            soloed: [None; STRIP_COUNT],
            master_fader: None,
            master_muted: None,
            master_soloed: None,
            master_color: None,
            active_view: AppView::Mixer,
            selected_strip: Some(SelectedStrip::Strip(0)),
            status: ConnectionStatus::Disconnected,
            last_error: None,
            parameter_values: std::collections::HashMap::new(),
            editing_name: None,
            editing_scene: None,
            copy_buffer: None,
            dca_spill: None,
            mute_spill: None,
            show_file_name: String::new(),
            editing_scene_safes: None,
            editing_snippet_filters: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Mixer,
    Channel,
    Config,
    Gate,
    Dyn,
    Eq,
    Sends,
    Main,
    Fx,
    Scenes,
    Setup,
    Routing,
    Rta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedStrip {
    Strip(usize),
    Master,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Checking,
    Connected(ProbeResponse),
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ConsoleUpdateReceived(Result<ConsoleUpdate, String>),
    GainChanged(usize, f32),
    GainReleased(usize),
    SendChanged(usize, usize, f32),
    PanChanged(usize, f32),
    FaderChanged(usize, f32),
    MasterFaderChanged(f32),
    NamesLoaded(Result<Vec<StripName>, String>),
    ColorsLoaded(Result<Vec<StripColor>, String>),
    GainsLoaded(Result<Vec<StripGain>, String>),
    SendsLoaded(Result<Vec<StripSend>, String>),
    PansLoaded(Result<Vec<StripPan>, String>),
    FadersLoaded(Result<Vec<StripFader>, String>),
    SendSetFinished(Result<(), String>),
    GainSetFinished(Result<(), String>),
    PanSetFinished(Result<(), String>),
    FaderSetFinished(Result<(), String>),
    MetersLoaded(Result<Vec<StripMeter>, String>),
    MasterMetersLoaded(Box<Result<MainMeterLevels, String>>),
    RtaMetersLoaded(Box<Result<[f32; 100], String>>),
    MutePressed(usize),
    MasterMutePressed,
    MasterSoloPressed,
    NavSelected(AppView),
    StripSelected(SelectedStrip),
    MutesLoaded(Result<Vec<StripMute>, String>),
    MuteSetFinished(Result<(), String>),
    SoloPressed(usize),
    SolosLoaded(Result<Vec<StripSolo>, String>),
    SoloSetFinished(Result<(), String>),
    DiscoveryFinished(Result<Vec<DiscoveredMixer>, String>),
    MixerSelected(SocketAddr),
    Disconnect,
    ParameterChanged(String, OscValue),
    ParameterSetFinished(Result<(), String>),
    ParametersLoaded(Result<Vec<(String, OscValue)>, String>),
    NameEditStarted(usize),
    NameEditChanged(usize, String),
    NameEditSubmitted(usize),
    ProbeFinished(Result<ProbeOutcome, String>),
    SceneRecall(i32),
    SceneSave(i32),
    SnippetRecall(i32),
    SnippetSave(i32),
    EditSnippetFilters(i32),
    RecorderAction(&'static str),
    CopyStrip(usize),
    PasteStrip(usize),
    DcaSpill(u8),
    MuteSpill(u8),
    ClearSpill,
    ShowFileNameChanged(String),
    ShowFileLoad,
    ShowFileSave,
    ClearSolo,
    Undo,
    EditSceneSafes(i32),
}

pub fn new() -> (StatusApp, Task<Message>) {
    let maybe_target = mixer_addr_from_args_or_env();
    let app = StatusApp {
        mixer_addr: maybe_target,
        discovered_mixer: None,
        discovered_mixers: Vec::new(),
        manual_target: maybe_target.is_some(),
        probe_in_flight: true,
        names: std::array::from_fn(|_| None),
        colors: [None; STRIP_COUNT],
        gains: [None; STRIP_COUNT],
        gain_sources: [GainSource::Trim; STRIP_COUNT],
        gain_drag_values: [None; STRIP_COUNT],
        sends: [[None; SEND_BUS_COUNT]; STRIP_COUNT],
        pans: [None; STRIP_COUNT],
        faders: [None; STRIP_COUNT],
        meters_db: [-90.0; STRIP_COUNT],
        master_meters_db: [-90.0, -90.0],
        rta_meters_db: [-128.0; 100],
        muted: [None; STRIP_COUNT],
        soloed: [None; STRIP_COUNT],
        master_fader: None,
        master_muted: None,
        master_soloed: None,
        master_color: None,
        active_view: AppView::Mixer,
        selected_strip: Some(SelectedStrip::Strip(0)),
        status: ConnectionStatus::Checking,
        last_error: None,
        parameter_values: std::collections::HashMap::new(),
        editing_name: None,
        editing_scene: None,
        copy_buffer: None,
        dca_spill: None,
        mute_spill: None,
        show_file_name: String::new(),
        editing_scene_safes: None,
        editing_snippet_filters: None,
    };

    let task = match maybe_target {
        Some(mixer_addr) => spawn_probe(mixer_addr),
        None => spawn_discovery(),
    };

    (app, task)
}

pub fn update(app: &mut StatusApp, message: Message) -> Task<Message> {
    match message {
        Message::Tick if app.probe_in_flight => Task::none(),
        Message::Tick => {
            app.probe_in_flight = true;
            match app.mixer_addr {
                Some(mixer_addr) => spawn_probe(mixer_addr),
                None => spawn_discovery(),
            }
        }
        Message::MixerSelected(addr) => {
            app.mixer_addr = Some(addr);
            app.manual_target = true;
            app.probe_in_flight = true;
            app.status = ConnectionStatus::Checking;
            app.last_error = None;
            app.discovered_mixer = app
                .discovered_mixers
                .iter()
                .find(|m| m.addr == addr)
                .cloned();
            spawn_probe(addr)
        }
        Message::Disconnect => {
            app.mixer_addr = None;
            app.manual_target = false;
            app.discovered_mixer = None;
            app.status = ConnectionStatus::Disconnected;
            app.last_error = None;
            app.names = std::array::from_fn(|_| None);
            app.colors = [None; STRIP_COUNT];
            app.gains = [None; STRIP_COUNT];
            app.gain_sources = [GainSource::Trim; STRIP_COUNT];
            app.sends = [[None; SEND_BUS_COUNT]; STRIP_COUNT];
            app.pans = [None; STRIP_COUNT];
            app.faders = [None; STRIP_COUNT];
            app.meters_db = [-90.0; STRIP_COUNT];
            app.master_meters_db = [-90.0, -90.0];
            app.rta_meters_db = [-128.0; 100];
            app.muted = [None; STRIP_COUNT];
            app.soloed = [None; STRIP_COUNT];
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
            let target = VISIBLE_STRIPS[index];
            let path = match target {
                FaderTarget::Channel(n) => format!("/ch/{n:02}/config/name"),
                FaderTarget::Aux(n) => format!("/auxin/{n:02}/config/name"),
                FaderTarget::Bus(n) => format!("/bus/{n:02}/config/name"),
                FaderTarget::FxRtn(n) => format!("/fxrtn/{n:02}/config/name"),
                FaderTarget::Mtx(n) => format!("/mtx/{n:02}/config/name"),
                FaderTarget::Dca(n) => format!("/dca/{n}/config/name"),
                FaderTarget::Main => "/main/st/config/name".to_owned(),
            };
            spawn_set_parameter(mixer_addr, path, OscValue::String(name))
        }
        Message::ConsoleUpdateReceived(result) => {
            match result {
                Ok(ConsoleUpdate::Gain(strip)) => {
                    if let Some(index) = VISIBLE_STRIPS
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        let keep_headamp_source = matches!(
                            (VISIBLE_STRIPS[index], app.gain_sources[index], strip.source),
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
                    for strip_index in 0..STRIP_COUNT {
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
                    if let Some(index) = VISIBLE_STRIPS
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.faders[index] = Some(strip.value);
                    }
                }
                Ok(ConsoleUpdate::Pan(strip)) => {
                    if let Some(index) = VISIBLE_STRIPS
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.pans[index] = Some(strip.value);
                    }
                }
                Ok(ConsoleUpdate::Send(strip)) => {
                    if let Some(strip_index) = VISIBLE_STRIPS
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
                    if let Some(index) = VISIBLE_STRIPS
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.muted[index] = Some(!strip.on);
                    }
                }
                Ok(ConsoleUpdate::Solo(strip)) => {
                    if let Some(index) = VISIBLE_STRIPS
                        .iter()
                        .position(|target| *target == strip.target)
                    {
                        app.soloed[index] = Some(strip.on);
                    }
                }
                Ok(ConsoleUpdate::Name(strip)) => {
                    if let Some(index) = VISIBLE_STRIPS
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
                    if let Some(index) = VISIBLE_STRIPS
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
            let target = VISIBLE_STRIPS[index];
            spawn_set_gain(mixer_addr, target, source, value)
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
            let target = VISIBLE_STRIPS[strip_index];
            let bus = SEND_BUSES[bus_index];
            spawn_set_send(mixer_addr, target, bus, value)
        }
        Message::PanChanged(index, value) => {
            if let Some(pan) = app.pans.get_mut(index) {
                *pan = Some(value);
            }

            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let target = VISIBLE_STRIPS[index];
            spawn_set_pan(mixer_addr, target, value)
        }
        Message::FaderChanged(index, value) => {
            if let Some(fader) = app.faders.get_mut(index) {
                *fader = Some(value);
            }

            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let target = VISIBLE_STRIPS[index];
            spawn_set_fader(mixer_addr, target, value)
        }
        Message::MasterFaderChanged(value) => {
            app.master_fader = Some(value);

            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_set_fader(mixer_addr, FaderTarget::Main, value)
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
                        if let Some(index) = VISIBLE_STRIPS
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
                        if let Some(index) = VISIBLE_STRIPS
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
                        if let Some(index) = VISIBLE_STRIPS
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
                        if let Some(strip_index) = VISIBLE_STRIPS
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
                        if let Some(index) = VISIBLE_STRIPS
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
                        if let Some(index) = VISIBLE_STRIPS
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
                        if let Some(index) = VISIBLE_STRIPS
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
                        if let Some(strip_index) = VISIBLE_STRIPS
                            .iter()
                            .position(|target| *target == FaderTarget::Bus((bus_index + 1) as u8))
                        {
                            app.meters_db[strip_index] = linear_meter_to_db(*level);
                        }
                    }
                    for (matrix_index, level) in levels.matrices.iter().enumerate() {
                        if let Some(strip_index) = VISIBLE_STRIPS.iter().position(|target| {
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
            let target = VISIBLE_STRIPS[index];
            let currently_muted = app
                .muted
                .get(index)
                .and_then(|state| *state)
                .unwrap_or(false);
            let next_on = currently_muted;
            if let Some(muted) = app.muted.get_mut(index) {
                *muted = Some(!next_on);
            }
            spawn_set_mute(mixer_addr, target, next_on)
        }
        Message::MasterMutePressed => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let currently_muted = app.master_muted.unwrap_or(false);
            let next_on = currently_muted;
            app.master_muted = Some(!next_on);
            spawn_set_mute(mixer_addr, FaderTarget::Main, next_on)
        }
        Message::MutesLoaded(result) => {
            match result {
                Ok(mutes) => {
                    for strip in mutes {
                        if strip.target == FaderTarget::Main {
                            app.master_muted = Some(!strip.on);
                            continue;
                        }
                        if let Some(index) = VISIBLE_STRIPS
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
            let target = VISIBLE_STRIPS[index];
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
            spawn_set_solo(mixer_addr, target, next_on)
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
                        if let Some(index) = VISIBLE_STRIPS
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
        Message::DiscoveryFinished(result) => {
            app.probe_in_flight = false;

            match result {
                Ok(mixers) => {
                    app.discovered_mixers = mixers;
                    if app.discovered_mixers.is_empty() {
                        app.last_error =
                            Some("no X32 mixer discovered on the local network".to_owned());
                    } else {
                        app.last_error = None;
                    }
                    Task::none()
                }
                Err(error) => {
                    app.discovered_mixers.clear();
                    app.last_error = Some(error);
                    Task::none()
                }
            }
        }
        Message::SceneRecall(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_scene_action(mixer_addr, "goscene", index)
        }
        Message::SceneSave(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            spawn_scene_action(mixer_addr, "savescene", index)
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
            app.copy_buffer = Some(VISIBLE_STRIPS[index]);
            Task::none()
        }
        Message::PasteStrip(index) => {
            let Some(mixer_addr) = app.mixer_addr else {
                return Task::none();
            };
            let Some(source) = app.copy_buffer else {
                return Task::none();
            };
            let target = VISIBLE_STRIPS[index];
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
        Message::ProbeFinished(result) => {
            app.probe_in_flight = false;
            let was_connected = matches!(app.status, ConnectionStatus::Connected(_));

            match result {
                Ok(ProbeOutcome::Connected { response, .. }) => {
                    app.status = ConnectionStatus::Connected(response);
                    if !was_connected && let Some(mixer_addr) = app.mixer_addr {
                        return Task::batch([
                            spawn_load_names(mixer_addr),
                            spawn_load_colors(mixer_addr),
                            spawn_load_gains(mixer_addr),
                            spawn_load_sends(mixer_addr),
                            spawn_load_pans(mixer_addr),
                            spawn_load_faders(mixer_addr),
                            spawn_load_mutes(mixer_addr),
                            spawn_load_solos(mixer_addr),
                            spawn_load_mute_groups(mixer_addr),
                        ]);
                    }
                }
                Ok(ProbeOutcome::Disconnected) => {
                    app.status = ConnectionStatus::Disconnected;
                    app.last_error = None;
                    app.names = std::array::from_fn(|_| None);
                    app.gains = [None; STRIP_COUNT];
                    app.gain_sources = [GainSource::Trim; STRIP_COUNT];
                    app.sends = [[None; SEND_BUS_COUNT]; STRIP_COUNT];
                    app.pans = [None; STRIP_COUNT];
                    app.faders = [None; STRIP_COUNT];
                    app.meters_db = [-90.0; STRIP_COUNT];
                    app.master_meters_db = [-90.0, -90.0];
                    app.muted = [None; STRIP_COUNT];
                    app.soloed = [None; STRIP_COUNT];
                    app.master_fader = None;
                    app.master_muted = None;
                    app.master_soloed = None;
                    app.master_color = None;
                    if !app.manual_target {
                        app.mixer_addr = None;
                        app.discovered_mixer = None;
                    }
                }
                Err(error) => {
                    app.status = ConnectionStatus::Disconnected;
                    app.last_error = Some(error);
                    app.names = std::array::from_fn(|_| None);
                    app.gains = [None; STRIP_COUNT];
                    app.gain_sources = [GainSource::Trim; STRIP_COUNT];
                    app.sends = [[None; SEND_BUS_COUNT]; STRIP_COUNT];
                    app.pans = [None; STRIP_COUNT];
                    app.faders = [None; STRIP_COUNT];
                    app.meters_db = [-90.0; STRIP_COUNT];
                    app.master_meters_db = [-90.0, -90.0];
                    app.muted = [None; STRIP_COUNT];
                    app.soloed = [None; STRIP_COUNT];
                    app.master_fader = None;
                    app.master_muted = None;
                    app.master_soloed = None;
                    app.master_color = None;
                    if !app.manual_target {
                        app.mixer_addr = None;
                        app.discovered_mixer = None;
                    }
                }
            }

            Task::none()
        }
    }
}

pub fn subscription(_app: &StatusApp) -> Subscription<Message> {
    let ticker = time::every(Duration::from_secs(3)).map(|_| Message::Tick);

    if let Some(mixer_addr) = _app.mixer_addr {
        Subscription::batch([
            ticker,
            state_subscription(mixer_addr),
            meter_subscription(mixer_addr),
            master_meter_subscription(mixer_addr),
            rta_meter_subscription(mixer_addr),
        ])
    } else {
        ticker
    }
}

pub fn theme(_app: &StatusApp) -> Theme {
    Theme::TokyoNight
}

pub fn view(app: &StatusApp) -> Element<'_, Message> {
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
        mixer_selection_view(app)
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

fn mixer_selection_view(app: &StatusApp) -> Element<'_, Message> {
    let title = text("Discovered Mixers").size(28);

    let mixer_list: Element<'_, Message> = if app.discovered_mixers.is_empty() {
        if app.probe_in_flight {
            text("Searching for mixers on the network...")
                .size(16)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3))
                .into()
        } else if let Some(ref error) = app.last_error {
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
        let mixers = app.discovered_mixers.iter().fold(
            column!().spacing(8).width(Length::Fill),
            |col, mixer| {
                let name = mixer.name.as_deref().unwrap_or("Unknown Mixer");
                let ip = mixer.addr.ip().to_string();
                let detail = match (&mixer.model, &mixer.firmware) {
                    (Some(model), Some(firmware)) => {
                        format!("{model} · firmware {firmware}")
                    }
                    (Some(model), None) => model.clone(),
                    _ => String::new(),
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
                    .align_y(iced::Alignment::Center);

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

#[derive(Clone, Copy)]
pub struct NavTab {
    icon: fn() -> iced::widget::Text<'static, Theme>,
    label: &'static str,
    view: AppView,
}

fn spill_bar(app: &StatusApp) -> Element<'static, Message> {
    // Mute group toggle buttons
    let mute_buttons: Element<'_, Message> = (1..=6)
        .fold(row!().spacing(4), |row, grp| {
            let path = format!("/config/mute/{grp}");
            let active = param_bool(app, &path);
            let (bg, border) = if active {
                (
                    Color::from_rgb8(0x8A, 0x3A, 0x3A),
                    Color::from_rgb8(0xC0, 0x5A, 0x5A),
                )
            } else {
                (
                    Color::from_rgb8(0x2A, 0x2A, 0x2C),
                    Color::from_rgb8(0x4A, 0x4A, 0x4C),
                )
            };
            row.push(
                button(text(format!("M{grp}")).size(11))
                    .on_press(Message::ParameterChanged(path, OscValue::Bool(!active)))
                    .padding([3, 8])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(bg)),
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: border,
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        },
                    ),
            )
        })
        .into();

    // Spill indicator buttons (shown when DCA spill is active)
    let spill_buttons: Element<'_, Message> = if let Some(dca) = app.dca_spill {
        (1..=8)
            .fold(row!().spacing(2), |row, n| {
                let active = dca == n;
                let (bg, border) = if active {
                    (
                        Color::from_rgb8(0x5A, 0x3A, 0x8A),
                        Color::from_rgb8(0x8A, 0x5A, 0xC0),
                    )
                } else {
                    (
                        Color::from_rgb8(0x1A, 0x1A, 0x1C),
                        Color::from_rgb8(0x2A, 0x2A, 0x2C),
                    )
                };
                row.push(
                    button(
                        text(if active {
                            format!("d{n}")
                        } else {
                            "·".to_owned()
                        })
                        .size(8),
                    )
                    .on_press(Message::DcaSpill(n))
                    .padding([1, 4])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(bg)),
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: border,
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        },
                    ),
                )
            })
            .into()
    } else {
        Space::new().width(Length::Fixed(0.0)).into()
    };

    let clear_btn: Element<'static, Message> =
        if app.dca_spill.is_some() || app.mute_spill.is_some() {
            button(text("Clear").size(10))
                .on_press(Message::ClearSpill)
                .padding([3, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x8A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x8A, 0xC0),
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            Space::new().width(Length::Fixed(0.0)).into()
        };

    // DCA spill buttons (only visible when no spill is active)
    let dca_buttons: Element<'_, Message> = if app.dca_spill.is_none() && app.mute_spill.is_none() {
        (1..=8)
            .fold(row!().spacing(4), |row, dca| {
                row.push(
                    button(text(format!("D{dca}")).size(11))
                        .on_press(Message::DcaSpill(dca))
                        .padding([3, 8])
                        .style(|_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C))),
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: Color::from_rgb8(0x4A, 0x4A, 0x4C),
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        }),
                )
            })
            .into()
    } else {
        Space::new().width(Length::Fixed(0.0)).into()
    };

    let clear_solo_btn = button(text("Clear Solo").size(10))
        .on_press(Message::ClearSolo)
        .padding([3, 8])
        .style(|_theme: &Theme, _status: button::Status| button::Style {
            background: Some(Background::Color(Color::from_rgb8(0x8A, 0x6A, 0x2A))),
            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
            border: Border {
                color: Color::from_rgb8(0xC0, 0xA0, 0x5A),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        });

    row![
        text("Mute:")
            .size(11)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
        mute_buttons,
        Space::new().width(Length::Fixed(8.0)),
        spill_buttons,
        clear_btn,
        dca_buttons,
        Space::new().width(Length::Fixed(12.0)),
        clear_solo_btn,
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .into()
}

fn top_nav_bar(app: &StatusApp) -> Element<'static, Message> {
    const TABS: [NavTab; 13] = [
        NavTab {
            icon: sliders_vertical,
            label: "Mixer",
            view: AppView::Mixer,
        },
        NavTab {
            icon: panel_left,
            label: "Channel",
            view: AppView::Channel,
        },
        NavTab {
            icon: file_input,
            label: "Config",
            view: AppView::Config,
        },
        NavTab {
            icon: toggle_left,
            label: "Gate",
            view: AppView::Gate,
        },
        NavTab {
            icon: audio_waveform,
            label: "Dyn",
            view: AppView::Dyn,
        },
        NavTab {
            icon: equal,
            label: "EQ",
            view: AppView::Eq,
        },
        NavTab {
            icon: send,
            label: "Sends",
            view: AppView::Sends,
        },
        NavTab {
            icon: audio_lines,
            label: "Main",
            view: AppView::Main,
        },
        NavTab {
            icon: shield,
            label: "FX1 - 8",
            view: AppView::Fx,
        },
        NavTab {
            icon: save,
            label: "Scenes",
            view: AppView::Scenes,
        },
        NavTab {
            icon: settings,
            label: "Setup",
            view: AppView::Setup,
        },
        NavTab {
            icon: git_merge,
            label: "Routing",
            view: AppView::Routing,
        },
        NavTab {
            icon: activity,
            label: "RTA",
            view: AppView::Rta,
        },
    ];

    let tabs = TABS.into_iter().fold(
        row!()
            .spacing(4)
            .padding([3, 3])
            .align_y(iced::Alignment::Center),
        |row, tab| row.push(nav_button(tab, app.active_view == tab.view)),
    );

    let disconnect_btn = button(text("Disconnect").size(12))
        .on_press(Message::Disconnect)
        .style(|_theme: &Theme, _status: button::Status| button::Style {
            background: Some(Background::Color(Color::from_rgb8(0xF0, 0x7C, 0x82))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb8(0xF0, 0x7C, 0x82),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    let bar = row![tabs, Space::new().width(Length::Fill), disconnect_btn]
        .spacing(8)
        .padding([4, 8])
        .align_y(iced::Alignment::Center)
        .width(Length::Fill);

    container(bar)
        .height(Length::Shrink)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x1C, 0x1C, 0x1C))),
            border: Border {
                color: Color::from_rgb8(0x2A, 0x2A, 0x2A),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn nav_button(tab: NavTab, selected: bool) -> Element<'static, Message> {
    let accent = mixer_accent_color();
    let active_text = accent;
    let inactive_text = Color::from_rgb8(0xA9, 0xAC, 0xB3);

    let icon =
        container(
            (tab.icon)()
                .size(17)
                .color(if selected { active_text } else { inactive_text }),
        )
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .padding(0)
        .center_x(Fill)
        .center_y(Fill)
        .style(move |_theme: &Theme| container::Style {
            border: Border {
                color: if selected {
                    accent
                } else {
                    Color::from_rgb8(0x6B, 0x6F, 0x76)
                },
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        });

    button(
        row![
            icon,
            text(tab.label)
                .size(14)
                .color(if selected { active_text } else { inactive_text }),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 10])
    .width(Length::Fixed(108.0))
    .height(Length::Fixed(36.0))
    .style(move |_theme: &Theme, _status| button::Style {
        background: Some(Background::Color(if selected {
            Color::from_rgb8(0x2A, 0x2A, 0x2A)
        } else {
            Color::from_rgb8(0x24, 0x24, 0x24)
        })),
        border: Border {
            color: if selected {
                Color::from_rgb8(0x4B, 0x4B, 0x4B)
            } else {
                Color::from_rgb8(0x3A, 0x3A, 0x3A)
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        text_color: if selected { active_text } else { inactive_text },
        ..Default::default()
    })
    .on_press(Message::NavSelected(tab.view))
    .into()
}

fn mixer_accent_color() -> Color {
    Color::from_rgb8(0x29, 0xE6, 0xF2)
}

fn strip_in_spill(app: &StatusApp, target: FaderTarget, _index: usize) -> bool {
    if let Some(dca) = app.dca_spill {
        let base = strip_base_path(target);
        let dca_path = format!("{base}/grp/dca");
        let dca_val = param_int(app, &dca_path);
        return (dca_val & (1 << (dca - 1))) != 0;
    }
    if let Some(grp) = app.mute_spill {
        let base = strip_base_path(target);
        let mute_path = format!("{base}/grp/mute");
        let mute_val = param_int(app, &mute_path);
        return (mute_val & (1 << (grp - 1))) != 0;
    }
    false
}

fn strip_module_item(label: &'static str) -> Element<'static, Message> {
    text(label)
        .size(12)
        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3))
        .into()
}

fn spawn_probe(mixer_addr: SocketAddr) -> Task<Message> {
    Task::perform(
        async move {
            ConnectionProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(400))
                .probe()
                .map_err(|error| error.to_string())
        },
        Message::ProbeFinished,
    )
}

fn spawn_load_faders(mixer_addr: SocketAddr) -> Task<Message> {
    Task::perform(
        async move {
            FaderBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .load(&[VISIBLE_STRIPS.as_slice(), &[FaderTarget::Main]].concat())
                .map_err(|error| error.to_string())
        },
        Message::FadersLoaded,
    )
}

fn spawn_load_names(mixer_addr: SocketAddr) -> Task<Message> {
    Task::perform(
        async move {
            NameBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .load(&VISIBLE_STRIPS)
                .map_err(|error| error.to_string())
        },
        Message::NamesLoaded,
    )
}

fn spawn_load_colors(mixer_addr: SocketAddr) -> Task<Message> {
    Task::perform(
        async move {
            ColorBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .load(&[VISIBLE_STRIPS.as_slice(), &[FaderTarget::Main]].concat())
                .map_err(|error| error.to_string())
        },
        Message::ColorsLoaded,
    )
}

fn spawn_load_gains(mixer_addr: SocketAddr) -> Task<Message> {
    let targets: Vec<FaderTarget> = VISIBLE_STRIPS
        .iter()
        .filter(|t| {
            !matches!(
                t,
                FaderTarget::Bus(_)
                    | FaderTarget::FxRtn(_)
                    | FaderTarget::Mtx(_)
                    | FaderTarget::Dca(_)
            )
        })
        .cloned()
        .collect();
    Task::perform(
        async move {
            GainBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .load(&targets)
                .map_err(|error| error.to_string())
        },
        Message::GainsLoaded,
    )
}

fn spawn_load_sends(mixer_addr: SocketAddr) -> Task<Message> {
    let channel_aux_targets: Vec<FaderTarget> = VISIBLE_STRIPS
        .iter()
        .filter(|t| {
            matches!(
                t,
                FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_)
            )
        })
        .cloned()
        .collect();
    let bus_targets: Vec<FaderTarget> = VISIBLE_STRIPS
        .iter()
        .filter(|t| matches!(t, FaderTarget::Bus(_) | FaderTarget::Main))
        .cloned()
        .collect();
    Task::batch([
        Task::perform(
            async move {
                SendBankProbe::new(mixer_addr)
                    .with_timeout(Duration::from_millis(250))
                    .load(&channel_aux_targets, &SEND_BUSES)
                    .map_err(|error| error.to_string())
            },
            Message::SendsLoaded,
        ),
        Task::perform(
            async move {
                SendBankProbe::new(mixer_addr)
                    .with_timeout(Duration::from_millis(250))
                    .load(&bus_targets, &MATRIX_SENDS)
                    .map_err(|error| error.to_string())
            },
            Message::SendsLoaded,
        ),
    ])
}

fn spawn_load_pans(mixer_addr: SocketAddr) -> Task<Message> {
    let targets: Vec<FaderTarget> = VISIBLE_STRIPS
        .iter()
        .filter(|t| !matches!(t, FaderTarget::Dca(_) | FaderTarget::Mtx(_)))
        .cloned()
        .collect();
    Task::perform(
        async move {
            PanBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .load(&targets)
                .map_err(|error| error.to_string())
        },
        Message::PansLoaded,
    )
}

fn spawn_load_mutes(mixer_addr: SocketAddr) -> Task<Message> {
    Task::perform(
        async move {
            MuteBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .load(&[VISIBLE_STRIPS.as_slice(), &[FaderTarget::Main]].concat())
                .map_err(|error| error.to_string())
        },
        Message::MutesLoaded,
    )
}

fn spawn_load_solos(mixer_addr: SocketAddr) -> Task<Message> {
    let targets: Vec<FaderTarget> = VISIBLE_STRIPS
        .iter()
        .filter(|t| !matches!(t, FaderTarget::Mtx(_) | FaderTarget::Dca(_)))
        .cloned()
        .collect();
    Task::perform(
        async move {
            SoloBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .load(&targets)
                .map_err(|error| error.to_string())
        },
        Message::SolosLoaded,
    )
}

fn spawn_set_fader(mixer_addr: SocketAddr, target: FaderTarget, value: f32) -> Task<Message> {
    Task::perform(
        async move {
            FaderBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .set(target, value)
                .map_err(|error| error.to_string())
        },
        Message::FaderSetFinished,
    )
}

fn spawn_set_pan(mixer_addr: SocketAddr, target: FaderTarget, value: f32) -> Task<Message> {
    Task::perform(
        async move {
            PanBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .set(target, value)
                .map_err(|error| error.to_string())
        },
        Message::PanSetFinished,
    )
}

fn spawn_set_send(
    mixer_addr: SocketAddr,
    target: FaderTarget,
    bus: u8,
    value: f32,
) -> Task<Message> {
    Task::perform(
        async move {
            SendBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .set(target, bus, value)
                .map_err(|error| error.to_string())
        },
        Message::SendSetFinished,
    )
}

fn spawn_set_gain(
    mixer_addr: SocketAddr,
    target: FaderTarget,
    source: GainSource,
    value: f32,
) -> Task<Message> {
    Task::perform(
        async move {
            GainBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .set(target, source, value)
                .map_err(|error| error.to_string())
        },
        Message::GainSetFinished,
    )
}

fn spawn_set_mute(mixer_addr: SocketAddr, target: FaderTarget, on: bool) -> Task<Message> {
    Task::perform(
        async move {
            MuteBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .set(target, on)
                .map_err(|error| error.to_string())
        },
        Message::MuteSetFinished,
    )
}

fn spawn_set_solo(mixer_addr: SocketAddr, target: FaderTarget, on: bool) -> Task<Message> {
    Task::perform(
        async move {
            SoloBankProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .set(target, on)
                .map_err(|error| error.to_string())
        },
        Message::SoloSetFinished,
    )
}

fn spawn_set_parameter(mixer_addr: SocketAddr, path: String, value: OscValue) -> Task<Message> {
    Task::perform(
        async move {
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .set(&path, value)
                .map_err(|error| error.to_string())
        },
        Message::ParameterSetFinished,
    )
}

fn spawn_fetch_snippet_filters(mixer_addr: SocketAddr, snip: i32) -> Task<Message> {
    let paths = vec![
        format!("/-show/showfile/snippet/{snip:03}/eventtyp"),
        format!("/-show/showfile/snippet/{snip:03}/channels"),
        format!("/-show/showfile/snippet/{snip:03}/auxbuses"),
        format!("/-show/showfile/snippet/{snip:03}/maingrps"),
    ];
    Task::perform(
        async move {
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(500))
                .load_batch(&paths)
                .map_err(|error| error.to_string())
        },
        Message::ParametersLoaded,
    )
}

fn spawn_scene_action(mixer_addr: SocketAddr, action: &'static str, index: i32) -> Task<Message> {
    Task::perform(
        async move {
            let path = format!("/-action/{action}");
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(500))
                .set(&path, OscValue::Int(index))
                .map_err(|error| error.to_string())
        },
        Message::ParameterSetFinished,
    )
}

fn spawn_recorder_action(mixer_addr: SocketAddr, action: &'static str) -> Task<Message> {
    Task::perform(
        async move {
            let path = format!("/-action/{action}");
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(500))
                .set(&path, OscValue::Int(1))
                .map_err(|error| error.to_string())
        },
        Message::ParameterSetFinished,
    )
}

fn target_to_ch_index(target: FaderTarget) -> i32 {
    match target {
        FaderTarget::Channel(n) => n as i32 - 1,
        FaderTarget::Aux(n) => 31 + n as i32,
        FaderTarget::FxRtn(n) => 39 + n as i32,
        FaderTarget::Bus(n) => 47 + n as i32,
        FaderTarget::Mtx(n) => 63 + n as i32,
        FaderTarget::Main => 70,
        FaderTarget::Dca(_) => -1,
    }
}

fn spawn_copy_paste(
    mixer_addr: SocketAddr,
    source: FaderTarget,
    target: FaderTarget,
) -> Task<Message> {
    Task::perform(
        async move {
            let src_index = target_to_ch_index(source);
            let dst_index = target_to_ch_index(target);
            if src_index < 0 || dst_index < 0 {
                return Err("Copy/paste not supported for DCAs".to_owned());
            }
            let probe =
                crate::ParameterProbe::new(mixer_addr).with_timeout(Duration::from_millis(1000));
            // Save source to temporary preset slot 99
            probe
                .set_multi(
                    "/save",
                    &[
                        OscValue::String("libchan".to_owned()),
                        OscValue::Int(99),
                        OscValue::String("mixosc_copy".to_owned()),
                        OscValue::Int(src_index),
                    ],
                )
                .map_err(|e| e.to_string())?;
            // Load preset slot 99 to destination
            probe
                .set_multi(
                    "/load",
                    &[
                        OscValue::String("libchan".to_owned()),
                        OscValue::Int(99),
                        OscValue::Int(dst_index),
                    ],
                )
                .map_err(|e| e.to_string())?;
            Ok(())
        },
        Message::ParameterSetFinished,
    )
}

fn panel_parameter_paths(app: &StatusApp) -> Option<Vec<String>> {
    let selected = app.selected_strip?;
    let (index, target, base) = match selected {
        SelectedStrip::Strip(index) => {
            let target = VISIBLE_STRIPS[index];
            (index, target, strip_base_path(target))
        }
        SelectedStrip::Master => {
            return match app.active_view {
                AppView::Eq => {
                    let mut p = vec!["/main/st/eq/on".to_owned()];
                    for band in 1..=6 {
                        p.push(format!("/main/st/eq/{band:02}/on"));
                        p.push(format!("/main/st/eq/{band:02}/f"));
                        p.push(format!("/main/st/eq/{band:02}/g"));
                        p.push(format!("/main/st/eq/{band:02}/q"));
                    }
                    Some(p)
                }
                AppView::Dyn => Some(vec![
                    "/main/st/dyn/on".to_owned(),
                    "/main/st/dyn/thr".to_owned(),
                    "/main/st/dyn/ratio".to_owned(),
                    "/main/st/dyn/knee".to_owned(),
                    "/main/st/dyn/mgain".to_owned(),
                    "/main/st/dyn/attack".to_owned(),
                    "/main/st/dyn/hold".to_owned(),
                    "/main/st/dyn/release".to_owned(),
                    "/main/st/dyn/mix".to_owned(),
                ]),
                AppView::Config => Some(vec![
                    "/main/st/insert/on".to_owned(),
                    "/main/st/insert/pos".to_owned(),
                    "/main/st/insert/sel".to_owned(),
                    "/main/m/insert/on".to_owned(),
                    "/main/m/insert/pos".to_owned(),
                    "/main/m/insert/sel".to_owned(),
                ]),
                AppView::Sends => {
                    let mut p = Vec::new();
                    for mtx in 1..=6 {
                        p.push(format!("/main/st/mix/{mtx:02}/level"));
                        p.push(format!("/main/st/mix/{mtx:02}/on"));
                    }
                    for mtx in (1..=6).step_by(2) {
                        p.push(format!("/main/st/mix/{mtx:02}/pan"));
                    }
                    p.push("/main/st/mix/on".to_owned());
                    p.push("/main/st/mix/fader".to_owned());
                    p.push("/main/st/mix/pan".to_owned());
                    p.push("/main/m/mix/on".to_owned());
                    p.push("/main/m/mix/fader".to_owned());
                    Some(p)
                }
                AppView::Main => Some(vec![
                    "/main/st/mix/on".to_owned(),
                    "/main/st/mix/fader".to_owned(),
                    "/main/st/mix/pan".to_owned(),
                    "/main/m/mix/on".to_owned(),
                    "/main/m/mix/fader".to_owned(),
                ]),
                _ => None,
            };
        }
    };

    let paths: Vec<String> = match app.active_view {
        AppView::Eq => {
            let bands = eq_band_count(target);
            if bands == 0 {
                return None;
            }
            let mut p = vec![format!("{base}/eq/on")];
            for band in 1..=bands {
                p.push(format!("{base}/eq/{band:02}/on"));
                p.push(format!("{base}/eq/{band:02}/f"));
                p.push(format!("{base}/eq/{band:02}/g"));
                p.push(format!("{base}/eq/{band:02}/q"));
            }
            p
        }
        AppView::Gate => {
            if !matches!(target, FaderTarget::Channel(_)) {
                return None;
            }
            vec![
                format!("{base}/gate/on"),
                format!("{base}/gate/mode"),
                format!("{base}/gate/auto"),
                format!("{base}/gate/thr"),
                format!("{base}/gate/range"),
                format!("{base}/gate/attack"),
                format!("{base}/gate/hold"),
                format!("{base}/gate/release"),
                format!("{base}/gate/keysrc"),
                format!("{base}/gate/filter/on"),
                format!("{base}/gate/filter/type"),
                format!("{base}/gate/filter/f"),
            ]
        }
        AppView::Dyn => {
            if matches!(
                target,
                FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Dca(_)
            ) {
                return None;
            }
            vec![
                format!("{base}/dyn/on"),
                format!("{base}/dyn/thr"),
                format!("{base}/dyn/ratio"),
                format!("{base}/dyn/knee"),
                format!("{base}/dyn/mgain"),
                format!("{base}/dyn/attack"),
                format!("{base}/dyn/hold"),
                format!("{base}/dyn/release"),
                format!("{base}/dyn/mix"),
                format!("{base}/dyn/auto"),
                format!("{base}/dyn/mode"),
                format!("{base}/dyn/det"),
                format!("{base}/dyn/env"),
                format!("{base}/dyn/pos"),
                format!("{base}/dyn/keysrc"),
                format!("{base}/dyn/filter/on"),
                format!("{base}/dyn/filter/type"),
                format!("{base}/dyn/filter/f"),
            ]
        }
        AppView::Config => {
            let mut p = Vec::new();
            match target {
                FaderTarget::Channel(_) | FaderTarget::Aux(_) => {
                    p.push(format!("{base}/preamp/trim"));
                    p.push(format!("{base}/preamp/invert"));
                    if matches!(target, FaderTarget::Channel(_)) {
                        p.push(format!("{base}/preamp/hpon"));
                        p.push(format!("{base}/preamp/hpf"));
                        p.push(format!("{base}/preamp/hpslope"));
                    }
                    p.push(format!("{base}/delay/on"));
                    p.push(format!("{base}/delay/time"));
                }
                FaderTarget::Mtx(_) => {
                    p.push(format!("{base}/preamp/invert"));
                }
                _ => {}
            }
            if matches!(
                target,
                FaderTarget::Channel(_)
                    | FaderTarget::Bus(_)
                    | FaderTarget::Mtx(_)
                    | FaderTarget::Main
            ) {
                p.push(format!("{base}/insert/on"));
                p.push(format!("{base}/insert/pos"));
                p.push(format!("{base}/insert/sel"));
            }
            if let FaderTarget::Channel(ch) = target {
                let headamp_index = match app.gain_sources[index] {
                    GainSource::Headamp(idx) => idx,
                    _ => ch - 1,
                };
                p.push(format!("/headamp/{headamp_index:03}/phantom"));
                p.push(format!("/headamp/{headamp_index:03}/gain"));
            }
            if matches!(
                target,
                FaderTarget::Channel(_)
                    | FaderTarget::Aux(_)
                    | FaderTarget::FxRtn(_)
                    | FaderTarget::Bus(_)
            ) {
                p.push(format!("{base}/grp/dca"));
                p.push(format!("{base}/grp/mute"));
            }
            if let FaderTarget::Channel(ch) = target
                && ch <= 8
            {
                p.push(format!("{base}/amix/on"));
                p.push(format!("{base}/amix/group"));
                p.push(format!("{base}/amix/weight"));
            }
            if !matches!(target, FaderTarget::Main) {
                p.push(format!("{base}/config/color"));
                if !matches!(target, FaderTarget::Dca(_)) {
                    p.push(format!("{base}/config/icon"));
                }
            }
            if p.is_empty() {
                return None;
            }
            p
        }
        AppView::Sends => {
            let mut p = Vec::new();
            match target {
                FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => {
                    for bus in 1..=16 {
                        p.push(format!("{base}/mix/{bus:02}/level"));
                        p.push(format!("{base}/mix/{bus:02}/on"));
                    }
                    for bus in (1..=16).step_by(2) {
                        p.push(format!("{base}/mix/{bus:02}/pan"));
                        p.push(format!("{base}/mix/{bus:02}/type"));
                    }
                    for bus in (1..=16).step_by(2) {
                        p.push(format!("/bus/{bus:02}/mix/st"));
                    }
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/st"));
                    p.push(format!("{base}/mix/pan"));
                    p.push(format!("{base}/mix/mono"));
                    p.push(format!("{base}/mix/mlevel"));
                }
                FaderTarget::Bus(_) => {
                    for mtx in 1..=6 {
                        p.push(format!("{base}/mix/{mtx:02}/level"));
                        p.push(format!("{base}/mix/{mtx:02}/on"));
                    }
                    for mtx in (1..=6).step_by(2) {
                        p.push(format!("{base}/mix/{mtx:02}/pan"));
                    }
                    for mtx in (1..=6).step_by(2) {
                        p.push(format!("/mtx/{mtx:02}/mix/st"));
                    }
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/st"));
                    p.push(format!("{base}/mix/pan"));
                    p.push(format!("{base}/mix/mono"));
                    p.push(format!("{base}/mix/mlevel"));
                }
                FaderTarget::Mtx(_) => {
                    p.push(format!("{base}/mix/on"));
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/pan"));
                }
                FaderTarget::Dca(_) => {
                    p.push(format!("{base}/on"));
                    p.push(format!("{base}/fader"));
                }
                FaderTarget::Main => {}
            }
            p
        }
        AppView::Main => {
            let mut p = Vec::new();
            match target {
                FaderTarget::Channel(_)
                | FaderTarget::Aux(_)
                | FaderTarget::FxRtn(_)
                | FaderTarget::Bus(_) => {
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/st"));
                    p.push(format!("{base}/mix/pan"));
                    p.push(format!("{base}/mix/mono"));
                    p.push(format!("{base}/mix/mlevel"));
                }
                FaderTarget::Mtx(_) => {
                    p.push(format!("{base}/mix/on"));
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/pan"));
                }
                FaderTarget::Dca(_) => {
                    p.push(format!("{base}/on"));
                    p.push(format!("{base}/fader"));
                }
                FaderTarget::Main => {}
            }
            p
        }
        AppView::Fx => {
            let mut p = Vec::new();
            for slot in 1..=8 {
                let fx_base = format!("/fx/{slot:02}");
                p.push(format!("{fx_base}/type"));
                if slot <= 4 {
                    p.push(format!("{fx_base}/source/l"));
                    p.push(format!("{fx_base}/source/r"));
                }
                for par in 1..=8 {
                    p.push(format!("{fx_base}/par/{par:02}"));
                }
            }
            p
        }
        _ => return None,
    };

    Some(paths)
}

fn spawn_load_mute_groups(mixer_addr: SocketAddr) -> Task<Message> {
    let paths: Vec<String> = (1..=6).map(|n| format!("/config/mute/{n}")).collect();
    Task::perform(
        async move {
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(200))
                .load_batch(&paths)
                .map_err(|error| error.to_string())
        },
        Message::ParametersLoaded,
    )
}

fn spawn_load_panel_parameters(app: &StatusApp, mixer_addr: SocketAddr) -> Option<Task<Message>> {
    match app.active_view {
        AppView::Scenes => {
            let mut scene_paths: Vec<String> = (1..=100)
                .map(|i| format!("/-show/showfile/scene/{i:03}/name"))
                .chain((1..=100).map(|i| format!("/-show/showfile/scene/{i:03}/hasData")))
                .chain((1..=100).map(|i| format!("/-show/showfile/scene/{i:03}/safes")))
                .chain((1..=100).map(|i| format!("/-show/showfile/scene/{i:03}/notes")))
                .collect();
            let mut cue_paths: Vec<String> = (0..100)
                .map(|i| format!("/-show/showfile/cue/{i:03}/name"))
                .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/scene")))
                .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/skip")))
                .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/miditype")))
                .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/midichan")))
                .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/midipara1")))
                .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/midipara2")))
                .collect();
            let mut snippet_paths: Vec<String> = (0..100)
                .map(|i| format!("/-show/showfile/snippet/{i:03}/name"))
                .chain((0..100).map(|i| format!("/-show/showfile/snippet/{i:03}/hasData")))
                .collect();
            let all_paths = scene_paths
                .drain(..)
                .chain(cue_paths.drain(..))
                .chain(snippet_paths.drain(..))
                .collect::<Vec<_>>();
            return Some(Task::batch(
                all_paths
                    .chunks(75)
                    .map(|chunk| {
                        let chunk = chunk.to_vec();
                        Task::perform(
                            async move {
                                crate::ParameterProbe::new(mixer_addr)
                                    .with_timeout(Duration::from_millis(1500))
                                    .load_batch(&chunk)
                                    .map_err(|error| error.to_string())
                            },
                            Message::ParametersLoaded,
                        )
                    })
                    .collect::<Vec<_>>(),
            ));
        }
        AppView::Setup => {
            let paths: Vec<String> = [
                "/config/talk/enable",
                "/config/talk/source",
                "/config/talk/A/level",
                "/config/talk/A/latch",
                "/config/talk/A/dim",
                "/config/talk/A/destmap",
                "/config/talk/B/level",
                "/config/talk/B/latch",
                "/config/talk/B/dim",
                "/config/talk/B/destmap",
                "/config/osc/type",
                "/config/osc/f",
                "/config/osc/fsel",
                "/config/osc/level",
                "/config/osc/dest",
                "/config/solo/level",
                "/config/solo/source",
                "/config/solo/sourcetrim",
                "/config/solo/chmode",
                "/config/solo/busmode",
                "/config/solo/dcamode",
                "/config/solo/exclusive",
                "/config/solo/followsel",
                "/config/solo/followsolo",
                "/config/solo/dimatt",
                "/config/solo/dim",
                "/config/solo/mono",
                "/config/solo/delay",
                "/config/solo/delaytime",
                "/config/solo/masterctrl",
                "/config/solo/mute",
                "/config/solo/dimpfl",
                "/-stat/urec/state",
                "/-stat/urec/rtime",
                "/-stat/urec/etime",
                "/-stat/sends on fader",
                "/-stat/geqonfdr",
                "/-stat/geqpos",
                "/config/userctrl/A/color",
                "/config/userctrl/B/color",
                "/config/userctrl/C/color",
                "/config/mono/link",
                "/config/tape/autoplay",
                "/-prefs/ip/dhcp",
                "/-prefs/clocksource",
                "/-prefs/clockrate",
                "/-prefs/clockmode",
                "/-prefs/bright",
                "/-prefs/lcdcont",
                "/-prefs/ledbright",
                "/-prefs/lamp",
                "/-prefs/lampon",
                "/-prefs/confirm_general",
                "/-prefs/confirm_overwrite",
                "/-prefs/confirm_sceneload",
                "/-prefs/remote/enable",
                "/-prefs/remote/protocol",
                "/-prefs/remote/port",
                "/-prefs/card/UFifc",
                "/-prefs/card/UFmode",
                "/-prefs/fastFaders",
                "/-prefs/hardmute",
                "/-prefs/dcamute",
                "/-prefs/invertmutes",
                "/-prefs/safe_masterlevels",
                "/-prefs/viewrtn",
                "/-prefs/scene_advance",
                "/-prefs/haflags",
                "/-prefs/show_control",
                "/-prefs/rec_control",
            ]
            .iter()
            .map(|s| s.to_string())
            .chain(["A", "B", "C"].iter().flat_map(|layer| {
                (1..=4)
                    .map(move |enc| format!("/config/userctrl/{layer}/enc/{enc}"))
                    .chain((5..=12).map(move |btn| format!("/config/userctrl/{layer}/btn/{btn}")))
            }))
            .collect();
            return Some(Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(1500))
                        .load_batch(&paths)
                        .map_err(|error| error.to_string())
                },
                Message::ParametersLoaded,
            ));
        }
        AppView::Routing => {
            let mut paths: Vec<String> = (1..=16)
                .map(|n| format!("/config/chlink/{n:02}"))
                .chain((1..=4).map(|n| format!("/config/auxlink/{n:02}")))
                .chain((1..=8).map(|n| format!("/config/buslink/{n:02}")))
                .chain((1..=4).map(|n| format!("/config/fxlink/{n:02}")))
                .chain((1..=3).map(|n| format!("/config/mtxlink/{n:02}")))
                .collect();
            paths.push("/config/linkcfg/hadly".to_owned());
            paths.push("/config/linkcfg/eq".to_owned());
            paths.push("/config/linkcfg/dyn".to_owned());
            paths.push("/config/linkcfg/fdrmute".to_owned());
            paths.extend((1..=32).map(|n| format!("/ch/{n:02}/config/source")));
            paths.push("/config/routing/OUT/1-4".to_owned());
            paths.push("/config/routing/OUT/5-8".to_owned());
            paths.push("/config/routing/OUT/9-12".to_owned());
            paths.push("/config/routing/OUT/13-16".to_owned());
            for out in 1..=16 {
                paths.push(format!("/outputs/main/{out:02}/delay/on"));
                paths.push(format!("/outputs/main/{out:02}/delay/time"));
                paths.push(format!("/outputs/main/{out:02}/src"));
            }
            for out in 1..=6 {
                paths.push(format!("/outputs/aux/{out:02}/src"));
            }
            return Some(Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(1000))
                        .load_batch(&paths)
                        .map_err(|error| error.to_string())
                },
                Message::ParametersLoaded,
            ));
        }
        AppView::Rta => {
            let paths = vec![
                "/-prefs/rta/source".to_owned(),
                "/-prefs/rta/gain".to_owned(),
                "/-prefs/rta/autogain".to_owned(),
                "/-prefs/rta/decay".to_owned(),
                "/-prefs/rta/mode".to_owned(),
            ];
            return Some(Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(400))
                        .load_batch(&paths)
                        .map_err(|error| error.to_string())
                },
                Message::ParametersLoaded,
            ));
        }
        _ => {}
    }
    let paths = panel_parameter_paths(app)?;
    Some(Task::perform(
        async move {
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(400))
                .load_batch(&paths)
                .map_err(|error| error.to_string())
        },
        Message::ParametersLoaded,
    ))
}

fn spawn_discovery() -> Task<Message> {
    Task::perform(
        async move {
            DiscoveryProbe::new()
                .with_timeout(Duration::from_millis(900))
                .discover()
                .map_err(|error| error.to_string())
        },
        Message::DiscoveryFinished,
    )
}

fn state_subscription(mixer_addr: SocketAddr) -> Subscription<Message> {
    Subscription::run_with(mixer_addr, state_worker).map(Message::ConsoleUpdateReceived)
}

fn mixer_addr_from_args_or_env() -> Option<SocketAddr> {
    let candidate = env::args()
        .nth(1)
        .or_else(|| env::var("MIXOSC_MIXER_ADDR").ok());

    candidate.and_then(|candidate| parse_target(&candidate).ok())
}

fn mixer_strips(app: &StatusApp) -> Element<'_, Message> {
    let strips = app.faders.iter().enumerate().fold(
        row!().spacing(0).align_y(iced::Alignment::End),
        |strips, (index, value)| {
            let gain_value = app.gain_drag_values[index]
                .or(app.gains[index])
                .unwrap_or(0.0);
            let gain_source = app.gain_sources[index];
            let fader_value = value.unwrap_or(0.0);
            let pan_value = app.pans[index].unwrap_or(0.5);
            let gain_label = format_gain_label(gain_value, gain_source);
            let value_label = value
                .map(format_fader_label)
                .unwrap_or_else(|| "--".to_owned());
            let pan_label = format_pan_label(pan_value);
            let target = VISIBLE_STRIPS[index];
            let is_muted = app.muted[index].unwrap_or(false);
            let is_soloed = app.soloed[index].unwrap_or(false);
            let meter = container(
                meters(1, &[app.meters_db[index]], STRIP_METER_HEIGHT)
                    .map(|()| unreachable!("meter widget does not emit messages")),
            )
            .height(Length::Fill);
            let scale = container(
                meter_ticks(STRIP_METER_HEIGHT)
                    .map(|()| unreachable!("tick widget does not emit messages")),
            )
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Bottom);
            let sends: Element<'_, Message> = match target {
                FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => SEND_BUSES
                    .iter()
                    .enumerate()
                    .fold(
                        column!().spacing(2).align_x(iced::Alignment::Center),
                        |column, (bus_index, _bus)| {
                            let send_value = app.sends[index][bus_index].unwrap_or(0.0);
                            column.push(
                                horizontal_slider(0.0..=1.0, send_value, move |next| {
                                    Message::SendChanged(index, bus_index, next)
                                })
                                .fill_from_start()
                                .step(0.01)
                                .double_click_reset(0.0)
                                .width(Length::Fixed(72.0))
                                .height(Length::Fixed(10.0)),
                            )
                        },
                    )
                    .into(),
                FaderTarget::Bus(_) | FaderTarget::Main => MATRIX_SENDS
                    .iter()
                    .enumerate()
                    .fold(
                        column!().spacing(2).align_x(iced::Alignment::Center),
                        |column, (bus_index, _bus)| {
                            let send_value = app.sends[index][bus_index].unwrap_or(0.0);
                            column.push(
                                horizontal_slider(0.0..=1.0, send_value, move |next| {
                                    Message::SendChanged(index, bus_index, next)
                                })
                                .fill_from_start()
                                .step(0.01)
                                .double_click_reset(0.0)
                                .width(Length::Fixed(72.0))
                                .height(Length::Fixed(10.0)),
                            )
                        },
                    )
                    .into(),
                FaderTarget::Mtx(_) | FaderTarget::Dca(_) => {
                    Space::new().height(Length::Fixed(0.0)).into()
                }
            };
            let hide_strip_top_controls = app.active_view != AppView::Mixer;
            let top_sends: Element<'_, Message> = if hide_strip_top_controls {
                Space::new().height(Length::Fixed(0.0)).into()
            } else {
                sends
            };
            let top_gain_label = if hide_strip_top_controls {
                String::new()
            } else {
                gain_label
            };
            let top_controls = strip_mixer_top(
                index,
                target,
                gain_value,
                gain_source,
                top_gain_label,
                pan_value,
                if hide_strip_top_controls {
                    String::new()
                } else {
                    pan_label
                },
                top_sends,
            );

            let solo_button: Element<'_, Message> = if matches!(target, FaderTarget::Mtx(_)) {
                Space::new().height(Length::Fixed(0.0)).into()
            } else {
                button(text("SOLO").size(12))
                    .padding([6, 8])
                    .style(move |_theme: &Theme, _status| {
                        toggle_button_style(is_soloed, Color::from_rgb8(0xF0, 0xC0, 0x30))
                    })
                    .on_press(Message::SoloPressed(index))
                    .into()
            };

            let mut strip = column![top_controls]
                .spacing(10)
                .align_x(iced::Alignment::Center);
            let strip_color = app.colors[index].unwrap_or(0);
            let color_rgb = x32_color_to_rgb(strip_color);
            let is_inverted = (9..=15).contains(&strip_color);
            let text_color = if is_inverted { Color::BLACK } else { color_rgb };
            let bg = if is_inverted {
                Some(Background::Color(color_rgb))
            } else {
                None
            };
            let is_selected = app.selected_strip == Some(SelectedStrip::Strip(index));
            let is_editing = app
                .editing_name
                .as_ref()
                .map(|(edit_index, _)| *edit_index == index)
                .unwrap_or(false);

            let name_element: Element<'_, Message> = if is_editing {
                let text = app
                    .editing_name
                    .as_ref()
                    .map(|(_, text)| text.clone())
                    .unwrap_or_default();
                text_input("Name", &text)
                    .size(12)
                    .width(Length::Fixed(80.0))
                    .on_input(move |t| Message::NameEditChanged(index, t))
                    .on_submit(Message::NameEditSubmitted(index))
                    .into()
            } else {
                button(
                    container(
                        text(strip_name(app, index, target))
                            .size(14)
                            .color(text_color),
                    )
                    .style(move |_theme: &Theme| container::Style {
                        border: Border {
                            color: color_rgb,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        background: bg,
                        ..Default::default()
                    })
                    .padding([2, 6]),
                )
                .style(button::text)
                .on_press(if is_selected {
                    Message::NameEditStarted(index)
                } else {
                    Message::StripSelected(SelectedStrip::Strip(index))
                })
                .into()
            };
            strip = strip.push(name_element);
            if !matches!(target, FaderTarget::Mtx(_)) {
                strip = strip.push(solo_button);
            }
            strip = strip.push(text(value_label).size(14));
            strip = strip.push(
                row![
                    vertical_slider(0.0..=1.0, fader_value, move |next| Message::FaderChanged(
                        index, next
                    ))
                    .height(Length::Fill)
                    .width(Length::Fixed(20.0))
                    .double_click_reset(0.75)
                    .step(0.01),
                    scale,
                    meter,
                ]
                .spacing(6)
                .height(Length::Fill)
                .align_y(iced::Alignment::End),
            );
            strip = strip.push(
                button(text("MUTE").size(12))
                    .padding([6, 8])
                    .style(move |_theme: &Theme, _status| {
                        toggle_button_style(is_muted, Color::from_rgb8(0xE0, 0x50, 0x50))
                    })
                    .on_press(Message::MutePressed(index)),
            );
            strip = strip.push(text(strip_label(target)).size(14));

            let in_spill = strip_in_spill(app, target, index);
            strips.push(
                container(strip)
                    .style(move |_theme: &Theme| container::Style {
                        border: Border {
                            color: if is_selected {
                                mixer_accent_color()
                            } else if in_spill {
                                Color::from_rgb8(0xF0, 0xC0, 0x50)
                            } else {
                                Color::from_rgb8(0x3B, 0x42, 0x52)
                            },
                            width: if is_selected || in_spill { 2.0 } else { 1.0 },
                            radius: 4.0.into(),
                        },
                        background: if in_spill {
                            Some(Background::Color(Color::from_rgb8(0x2A, 0x25, 0x15)))
                        } else {
                            None
                        },
                        ..Default::default()
                    })
                    .padding([0, 7]),
            )
        },
    );

    let master_selected = app.selected_strip == Some(SelectedStrip::Master);
    let master_strip = {
        let value = app.master_fader.unwrap_or(0.0);
        let value_label = app
            .master_fader
            .map(format_fader_label)
            .unwrap_or_else(|| "--".to_owned());
        let is_muted = app.master_muted.unwrap_or(false);
        let is_soloed = app.master_soloed.unwrap_or(false);
        let meter = container(
            meters(2, &app.master_meters_db, STRIP_METER_HEIGHT)
                .map(|()| unreachable!("meter widget does not emit messages")),
        )
        .height(Length::Fill);
        let scale = container(
            meter_ticks(STRIP_METER_HEIGHT)
                .map(|()| unreachable!("tick widget does not emit messages")),
        )
        .height(Length::Fill)
        .align_y(iced::alignment::Vertical::Bottom);

        column![
            Space::new().height(Length::Fixed(26.0)),
            Space::new().height(Length::Fixed(0.0)),
            {
                let master_color_val = app.master_color.unwrap_or(0);
                let color_rgb = x32_color_to_rgb(master_color_val);
                let is_inverted = (9..=15).contains(&master_color_val);
                let text_color = if is_inverted { Color::BLACK } else { color_rgb };
                let bg = if is_inverted {
                    Some(Background::Color(color_rgb))
                } else {
                    None
                };
                button(
                    container(text("LR").size(14).color(text_color))
                        .style(move |_theme: &Theme| container::Style {
                            border: Border {
                                color: color_rgb,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            background: bg,
                            ..Default::default()
                        })
                        .padding([2, 6]),
                )
                .style(button::text)
                .on_press(Message::StripSelected(SelectedStrip::Master))
            },
            button(text("SOLO").size(12))
                .padding([6, 8])
                .style(move |_theme: &Theme, _status| toggle_button_style(
                    is_soloed,
                    Color::from_rgb8(0xF0, 0xC0, 0x30)
                ))
                .on_press(Message::MasterSoloPressed),
            text(value_label).size(14),
            row![
                vertical_slider(0.0..=1.0, value, Message::MasterFaderChanged)
                    .height(Length::Fill)
                    .width(Length::Fixed(20.0))
                    .double_click_reset(0.75)
                    .step(0.01),
                scale,
                meter,
            ]
            .spacing(6)
            .height(Length::Fill)
            .align_y(iced::Alignment::End),
            button(text("MUTE").size(12))
                .padding([6, 8])
                .style(move |_theme: &Theme, _status| toggle_button_style(
                    is_muted,
                    Color::from_rgb8(0xE0, 0x50, 0x50)
                ))
                .on_press(Message::MasterMutePressed),
            text("LR").size(14),
        ]
        .spacing(10)
        .align_x(iced::Alignment::Center)
    };

    let master_strip = container(master_strip)
        .style(move |_theme: &Theme| container::Style {
            border: Border {
                color: if master_selected {
                    mixer_accent_color()
                } else {
                    Color::from_rgb8(0x3B, 0x42, 0x52)
                },
                width: if master_selected { 2.0 } else { 1.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .padding([0, 7]);

    container(
        row![
            scrollable(
                column![
                    strips.height(Length::Fill),
                    Space::new().height(Length::Fixed(18.0))
                ]
                .height(Length::Fill),
            )
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new()
            ))
            .width(Length::Fill)
            .height(Length::Fill),
            master_strip,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::Alignment::End),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn strip_mixer_top(
    index: usize,
    target: FaderTarget,
    gain_value: f32,
    gain_source: GainSource,
    gain_label: String,
    pan_value: f32,
    pan_label: String,
    sends: Element<'_, Message>,
) -> Element<'_, Message> {
    let hide_upper_controls = gain_label.is_empty();
    let hide_balance = pan_label.is_empty();
    let gain_block: Element<'static, Message> = if hide_upper_controls
        || matches!(
            target,
            FaderTarget::Bus(_) | FaderTarget::FxRtn(_) | FaderTarget::Mtx(_) | FaderTarget::Dca(_)
        ) {
        Space::new().height(Length::Fixed(26.0)).into()
    } else {
        column![
            text(gain_label).size(12),
            horizontal_slider(gain_range(gain_source), gain_value, move |next| {
                Message::GainChanged(index, next)
            })
            .fill_from_start()
            .filled_color(Color::from_rgb8(0xD9, 0x7A, 0x2B))
            .handle_color(Color::from_rgb8(0xF3, 0xB3, 0x6A))
            .step(gain_step(gain_source))
            .double_click_reset(0.0)
            .on_release(Message::GainReleased(index))
            .width(Length::Fixed(72.0))
            .height(Length::Fixed(10.0)),
        ]
        .spacing(4)
        .align_x(iced::Alignment::Center)
        .into()
    };

    let pan_block: Element<'static, Message> =
        if hide_balance || matches!(target, FaderTarget::Dca(_) | FaderTarget::Mtx(_)) {
            Space::new().height(Length::Fixed(0.0)).into()
        } else {
            column![
                text(pan_label).size(12),
                horizontal_slider(0.0..=1.0, pan_value, move |next| Message::PanChanged(
                    index, next
                ))
                .step(0.01)
                .double_click_reset(0.5)
                .width(Length::Fixed(72.0))
                .height(Length::Fixed(12.0)),
            ]
            .spacing(4)
            .align_x(iced::Alignment::Center)
            .into()
        };

    let mut top = column![gain_block]
        .spacing(10)
        .align_x(iced::Alignment::Center);
    if !hide_upper_controls && !matches!(target, FaderTarget::Mtx(_) | FaderTarget::Dca(_)) {
        top = top.push(sends);
        top = top.push(
            column![
                strip_module_item("Gate"),
                strip_module_item("EQ"),
                strip_module_item("Dyn"),
            ]
            .spacing(4)
            .align_x(iced::Alignment::Center),
        );
    }
    top.push(pan_block).into()
}

fn gate_summary<'a>(app: &'a StatusApp, base: &str) -> Element<'a, Message> {
    let on = param_bool(app, &format!("{base}/gate/on"));
    let thr = param_float(app, &format!("{base}/gate/thr"));
    let color = if on {
        Color::from_rgb8(0x7D, 0xD3, 0xA7)
    } else {
        Color::from_rgb8(0x8E, 0x94, 0x9D)
    };
    column![
        text(if on { "ON" } else { "OFF" }).size(14).color(color),
        text(format!("Thr: {thr:.1}"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center)
    .into()
}

fn eq_summary<'a>(app: &'a StatusApp, base: &str, bands: u8) -> Element<'a, Message> {
    let on = param_bool(app, &format!("{base}/eq/on"));
    let active = if on {
        (1..=bands)
            .filter(|b| param_bool(app, &format!("{base}/eq/{b:02}/on")))
            .count()
    } else {
        0
    };
    let color = if on {
        Color::from_rgb8(0x7D, 0xD3, 0xA7)
    } else {
        Color::from_rgb8(0x8E, 0x94, 0x9D)
    };
    column![
        text(if on { "ON" } else { "OFF" }).size(14).color(color),
        text(format!("{active}/{bands} bands"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center)
    .into()
}

const DYN_RATIO_NAMES: [&str; 12] = [
    "1.1", "1.3", "1.5", "2.0", "2.5", "3.0", "4.0", "5.0", "7.0", "10", "20", "100",
];

fn dyn_summary<'a>(app: &'a StatusApp, base: &str) -> Element<'a, Message> {
    let on = param_bool(app, &format!("{base}/dyn/on"));
    let thr = param_float(app, &format!("{base}/dyn/thr"));
    let ratio_idx = param_float(app, &format!("{base}/dyn/ratio")) as i32;
    let ratio_name = DYN_RATIO_NAMES
        .get(ratio_idx as usize)
        .copied()
        .unwrap_or("?");
    let color = if on {
        Color::from_rgb8(0x7D, 0xD3, 0xA7)
    } else {
        Color::from_rgb8(0x8E, 0x94, 0x9D)
    };
    column![
        text(if on { "ON" } else { "OFF" }).size(14).color(color),
        text(format!("Thr: {thr:.1}"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        text(format!("Ratio: {ratio_name}"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center)
    .into()
}

fn module_summary_panel<'a>(
    title: &'static str,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        column![
            text(title)
                .size(12)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            content,
        ]
        .spacing(8)
        .align_x(iced::Alignment::Center),
    )
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x1A, 0x1A, 0x1C))),
        border: Border {
            color: Color::from_rgb8(0x4B, 0x4B, 0x4B),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .padding([10, 10])
    .height(Length::Fixed(220.0))
    .width(Length::Fixed(140.0))
    .into()
}

fn color_selector(path: String, current: u8) -> Element<'static, Message> {
    const COLORS: [(u8, Color); 15] = [
        (0, Color::from_rgb8(0x3B, 0x42, 0x52)),
        (1, Color::from_rgb8(0xFF, 0x45, 0x45)),
        (2, Color::from_rgb8(0x32, 0xCD, 0x32)),
        (3, Color::from_rgb8(0xFF, 0xD7, 0x00)),
        (4, Color::from_rgb8(0x41, 0x69, 0xE1)),
        (5, Color::from_rgb8(0xFF, 0x00, 0xFF)),
        (6, Color::from_rgb8(0x00, 0xFF, 0xFF)),
        (7, Color::from_rgb8(0xFF, 0xFF, 0xFF)),
        (9, Color::from_rgb8(0xCC, 0x33, 0x33)),
        (10, Color::from_rgb8(0x28, 0xA4, 0x28)),
        (11, Color::from_rgb8(0xCC, 0xAC, 0x00)),
        (12, Color::from_rgb8(0x33, 0x55, 0xB4)),
        (13, Color::from_rgb8(0xCC, 0x00, 0xCC)),
        (14, Color::from_rgb8(0x00, 0xCC, 0xCC)),
        (15, Color::from_rgb8(0xDD, 0xDD, 0xDD)),
    ];

    COLORS
        .into_iter()
        .fold(row!().spacing(2), |r, (val, color)| {
            let selected = current == val;
            r.push(
                button(
                    Space::new()
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0)),
                )
                .on_press(Message::ParameterChanged(
                    path.clone(),
                    OscValue::Int(val as i32),
                ))
                .style(
                    move |_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(color)),
                        border: Border {
                            color: if selected {
                                Color::WHITE
                            } else {
                                Color::TRANSPARENT
                            },
                            width: if selected { 2.0 } else { 0.0 },
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    },
                ),
            )
        })
        .into()
}

fn icon_selector(path: String, current: i32) -> Element<'static, Message> {
    let prev = (current - 1).max(0);
    let next = (current + 1).min(74);
    row![
        button(text("-").size(12))
            .on_press(Message::ParameterChanged(path.clone(), OscValue::Int(prev)))
            .padding([2, 6]),
        text(format!("Icon {current}"))
            .size(11)
            .width(Length::Fixed(48.0)),
        button(text("+").size(12))
            .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
            .padding([2, 6]),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .into()
}

fn channel_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => 0,
    };
    let target = VISIBLE_STRIPS[index];
    let pan_value = app.pans[index].unwrap_or(0.5);
    let base = strip_base_path(target);

    let sends: Element<'_, Message> = match target {
        FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => SEND_BUSES
            .iter()
            .enumerate()
            .fold(column!().spacing(4), |column, (bus_index, bus)| {
                let send_value = app.sends[index][bus_index].unwrap_or(0.0);
                column.push(channel_send_row(index, bus_index, *bus, send_value))
            })
            .into(),
        FaderTarget::Bus(_) | FaderTarget::Main => MATRIX_SENDS
            .iter()
            .enumerate()
            .fold(column!().spacing(4), |column, (bus_index, bus)| {
                let send_value = app.sends[index][bus_index].unwrap_or(0.0);
                column.push(channel_send_row(index, bus_index, *bus, send_value))
            })
            .into(),
        FaderTarget::Mtx(_) | FaderTarget::Dca(_) => text("No sends")
            .size(14)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D))
            .into(),
    };

    let gate_content: Element<'_, Message> = if matches!(target, FaderTarget::Channel(_)) {
        gate_summary(app, &base)
    } else {
        text("N/A")
            .size(12)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D))
            .into()
    };

    let eq_content: Element<'_, Message> = {
        let bands = eq_band_count(target);
        if bands > 0 {
            eq_summary(app, &base, bands)
        } else {
            text("N/A")
                .size(12)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D))
                .into()
        }
    };

    let dyn_content: Element<'_, Message> = if matches!(
        target,
        FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Dca(_)
    ) {
        text("N/A")
            .size(12)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D))
            .into()
    } else {
        dyn_summary(app, &base)
    };

    let gate_panel = module_summary_panel("Noise Gate", gate_content);
    let eq_panel = module_summary_panel("Equalizer", eq_content);
    let dyn_panel = module_summary_panel("Dynamics", dyn_content);
    let sends_panel = detail_panel("Bus Sends", sends);

    let groups_panel: Option<Element<'_, Message>> = if matches!(
        target,
        FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Bus(_)
    ) {
        Some(detail_panel(
            "Groups",
            column![dca_group_chips(app, &base), mute_group_chips(app, &base),].spacing(8),
        ))
    } else {
        None
    };

    let balance_panel = detail_panel(
        "Balance",
        column![
            text(format_pan_label(pan_value))
                .size(18)
                .color(Color::from_rgb8(0xE6, 0xE8, 0xEE)),
            horizontal_slider(0.0..=1.0, pan_value, move |next| Message::PanChanged(
                index, next
            ))
            .step(0.01)
            .double_click_reset(0.5)
            .width(Length::Fixed(150.0))
            .height(Length::Fixed(14.0)),
        ]
        .spacing(10)
        .align_x(iced::Alignment::Center),
    );

    let mut panels_row = row![gate_panel, eq_panel, dyn_panel, sends_panel,].spacing(2);
    if let Some(groups) = groups_panel {
        panels_row = panels_row.push(groups);
    }
    panels_row = panels_row.push(balance_panel);
    container(panels_row)
        .height(Length::Shrink)
        .width(Length::Fill)
        .into()
}

const INSERT_NAMES: [&str; 23] = [
    "OFF", "FX1L", "FX1R", "FX2L", "FX2R", "FX3L", "FX3R", "FX4L", "FX4R", "FX5L", "FX5R", "FX6L",
    "FX6R", "FX7L", "FX7R", "FX8L", "FX8R", "AUX1", "AUX2", "AUX3", "AUX4", "AUX5", "AUX6",
];

fn insert_selector<'a>(path: String, current: i32) -> Element<'a, Message> {
    let name = INSERT_NAMES.get(current as usize).copied().unwrap_or("OFF");
    let prev = (current - 1).max(0);
    let next = (current + 1).min(22);
    row![
        button(text("-").size(12))
            .on_press(Message::ParameterChanged(path.clone(), OscValue::Int(prev)))
            .padding([2, 6]),
        text(name).size(11).width(Length::Fixed(48.0)),
        button(text("+").size(12))
            .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
            .padding([2, 6]),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .into()
}

fn dca_group_chips<'a>(app: &'a StatusApp, base: &str) -> Element<'a, Message> {
    let path = format!("{base}/grp/dca");
    let dca_val = match app.parameter_values.get(&path) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let chips: Element<'_, Message> = (1..=8)
        .fold(row!().spacing(3), |row, dca| {
            let active = (dca_val & (1 << (dca - 1))) != 0;
            let bit = 1 << (dca - 1);
            let new_val = if active {
                dca_val & !bit
            } else {
                dca_val | bit
            };
            let (bg, border) = if active {
                (
                    Color::from_rgb8(0x3A, 0x5A, 0x3A),
                    Color::from_rgb8(0x5A, 0x8A, 0x5A),
                )
            } else {
                (
                    Color::from_rgb8(0x2A, 0x2A, 0x2C),
                    Color::from_rgb8(0x4A, 0x4A, 0x4C),
                )
            };
            row.push(
                button(text(format!("D{dca}")).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(new_val),
                    ))
                    .padding([2, 4])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(bg)),
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: border,
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        },
                    ),
            )
        })
        .into();
    column![
        text("DCA Groups")
            .size(11)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        chips,
    ]
    .spacing(4)
    .into()
}

fn mute_group_chips<'a>(app: &'a StatusApp, base: &str) -> Element<'a, Message> {
    let path = format!("{base}/grp/mute");
    let mute_val = match app.parameter_values.get(&path) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let chips: Element<'_, Message> = (1..=6)
        .fold(row!().spacing(3), |row, grp| {
            let active = (mute_val & (1 << (grp - 1))) != 0;
            let bit = 1 << (grp - 1);
            let new_val = if active {
                mute_val & !bit
            } else {
                mute_val | bit
            };
            let (bg, border) = if active {
                (
                    Color::from_rgb8(0x5A, 0x3A, 0x3A),
                    Color::from_rgb8(0x8A, 0x5A, 0x5A),
                )
            } else {
                (
                    Color::from_rgb8(0x2A, 0x2A, 0x2C),
                    Color::from_rgb8(0x4A, 0x4A, 0x4C),
                )
            };
            row.push(
                button(text(format!("M{grp}")).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(new_val),
                    ))
                    .padding([2, 4])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(bg)),
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: border,
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        },
                    ),
            )
        })
        .into();
    column![
        text("Mute Groups")
            .size(11)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        chips,
    ]
    .spacing(4)
    .into()
}

fn config_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return config_detail_panel_for_base(app, "/main/st".to_owned(), FaderTarget::Main, 0);
        }
    };
    let target = VISIBLE_STRIPS[index];
    let base = strip_base_path(target);
    config_detail_panel_for_base(app, base, target, index)
}

fn config_detail_panel_for_base<'a>(
    app: &'a StatusApp,
    base: String,
    target: FaderTarget,
    strip_index: usize,
) -> Element<'a, Message> {
    let mut panels = row!().spacing(8);

    // Preamp section
    match target {
        FaderTarget::Channel(_) | FaderTarget::Aux(_) => {
            let trim = param_float(app, &format!("{base}/preamp/trim"));
            let invert = param_bool(app, &format!("{base}/preamp/invert"));
            let mut preamp_col = column!().spacing(8);
            preamp_col = preamp_col.push(param_slider_labeled(
                "Trim",
                format!("{base}/preamp/trim"),
                trim,
                |v| format_db1(linf_value(v, -18.0, 18.0)),
            ));
            preamp_col = preamp_col.push(param_toggle(
                "Invert",
                format!("{base}/preamp/invert"),
                invert,
            ));
            if matches!(target, FaderTarget::Channel(_)) {
                if let GainSource::Headamp(idx) = app.gain_sources[strip_index] {
                    let gain_path = format!("/headamp/{idx:03}/gain");
                    let gain = param_float(app, &gain_path);
                    preamp_col =
                        preamp_col.push(param_slider_labeled("Gain", gain_path, gain, |v| {
                            format_db1(linf_value(v, -12.0, 60.0))
                        }));
                }
                let hpon = param_bool(app, &format!("{base}/preamp/hpon"));
                let hpf = param_float(app, &format!("{base}/preamp/hpf"));
                let hpslope = match app.parameter_values.get(&format!("{base}/preamp/hpslope")) {
                    Some(OscValue::Int(v)) => *v,
                    _ => 0,
                };
                preamp_col =
                    preamp_col.push(param_toggle("HP On", format!("{base}/preamp/hpon"), hpon));
                preamp_col = preamp_col.push(param_slider_labeled(
                    "HP Freq",
                    format!("{base}/preamp/hpf"),
                    hpf,
                    |v| format_hz(logf_value(v, 20.0, 400.0)),
                ));
                preamp_col = preamp_col.push(cycle_button(
                    "Slope",
                    format!("{base}/preamp/hpslope"),
                    hpslope,
                    &["12", "18", "24"],
                ));
            }
            panels = panels.push(detail_panel("Preamp", preamp_col));
        }
        FaderTarget::Mtx(_) => {
            let invert = param_bool(app, &format!("{base}/preamp/invert"));
            let col = column![param_toggle(
                "Invert",
                format!("{base}/preamp/invert"),
                invert
            ),]
            .spacing(8);
            panels = panels.push(detail_panel("Preamp", col));
        }
        _ => {}
    }

    // Delay section
    if matches!(target, FaderTarget::Channel(_) | FaderTarget::Aux(_)) {
        let delay_on = param_bool(app, &format!("{base}/delay/on"));
        let delay_time = param_float(app, &format!("{base}/delay/time"));
        panels = panels.push(detail_panel(
            "Delay",
            column![
                param_toggle("On", format!("{base}/delay/on"), delay_on),
                param_slider_labeled("Time", format!("{base}/delay/time"), delay_time, |v| {
                    format_ms(linf_value(v, 0.3, 500.0))
                }),
            ]
            .spacing(8),
        ));
    }

    // Insert section
    if matches!(
        target,
        FaderTarget::Channel(_) | FaderTarget::Bus(_) | FaderTarget::Mtx(_) | FaderTarget::Main
    ) {
        let insert_on = param_bool(app, &format!("{base}/insert/on"));
        let insert_pos = param_bool(app, &format!("{base}/insert/pos"));
        let insert_sel = match app.parameter_values.get(&format!("{base}/insert/sel")) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let insert_col = column![
            param_toggle("On", format!("{base}/insert/on"), insert_on),
            param_toggle("Post", format!("{base}/insert/pos"), insert_pos),
            insert_selector(format!("{base}/insert/sel"), insert_sel),
        ]
        .spacing(8);
        let title = if target == FaderTarget::Main {
            "Insert ST"
        } else {
            "Insert"
        };
        panels = panels.push(detail_panel(title, insert_col));
    }

    // Main Mono insert
    if target == FaderTarget::Main {
        let insert_on = param_bool(app, "/main/m/insert/on");
        let insert_pos = param_bool(app, "/main/m/insert/pos");
        let insert_sel = match app.parameter_values.get("/main/m/insert/sel") {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let insert_col = column![
            param_toggle("On", "/main/m/insert/on".to_owned(), insert_on),
            param_toggle("Post", "/main/m/insert/pos".to_owned(), insert_pos),
            insert_selector("/main/m/insert/sel".to_owned(), insert_sel),
        ]
        .spacing(8);
        panels = panels.push(detail_panel("Insert M", insert_col));
    }

    // Phantom power for channels
    if let FaderTarget::Channel(ch) = target {
        let headamp_index = match app.gain_sources[strip_index] {
            GainSource::Headamp(idx) => idx,
            _ => ch - 1,
        };
        let phantom_path = format!("/headamp/{headamp_index:03}/phantom");
        let phantom_on = param_bool(app, &phantom_path);
        panels = panels.push(detail_panel(
            "Phantom",
            column![param_toggle("+48V", phantom_path, phantom_on)].spacing(8),
        ));
    }

    // Color / Icon
    if !matches!(target, FaderTarget::Main) {
        let color_path = format!("{base}/config/color");
        let color_val = match app.parameter_values.get(&color_path) {
            Some(OscValue::Int(v)) => *v as u8,
            _ => 0,
        };
        let mut color_icon_col = column![
            text("Color")
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            color_selector(color_path, color_val),
        ]
        .spacing(4);
        if !matches!(target, FaderTarget::Dca(_)) {
            let icon_path = format!("{base}/config/icon");
            let icon_val = match app.parameter_values.get(&icon_path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            color_icon_col = color_icon_col.push(icon_selector(icon_path, icon_val));
        }
        panels = panels.push(detail_panel("Appearance", color_icon_col));
    }

    // DCA / Mute groups
    if matches!(
        target,
        FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Bus(_)
    ) {
        panels = panels.push(detail_panel(
            "Groups",
            column![dca_group_chips(app, &base), mute_group_chips(app, &base),].spacing(8),
        ));
    }

    // Automix (channels 1-8 only)
    if let FaderTarget::Channel(ch) = target
        && ch <= 8
    {
        let amix_on = param_bool(app, &format!("{base}/amix/on"));
        let amix_group = match app.parameter_values.get(&format!("{base}/amix/group")) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let amix_weight = param_float(app, &format!("{base}/amix/weight"));
        panels = panels.push(detail_panel(
            "Automix",
            column![
                param_toggle("On", format!("{base}/amix/on"), amix_on),
                cycle_button(
                    "Group",
                    format!("{base}/amix/group"),
                    amix_group,
                    &["A", "B"]
                ),
                param_slider_labeled(
                    "Weight",
                    format!("{base}/amix/weight"),
                    amix_weight,
                    |v| format!("{:.0}", v * 100.0)
                ),
            ]
            .spacing(6),
        ));
    }

    // Copy / Paste
    let copy_paste_col = column![
        button(text("Copy").size(10))
            .on_press(Message::CopyStrip(strip_index))
            .padding([3, 10])
            .style(
                move |_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x8A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x8A, 0xC0),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }
            ),
        button(text("Paste").size(10))
            .on_press(Message::PasteStrip(strip_index))
            .padding([3, 10])
            .style(
                move |_theme: &Theme, _status: button::Status| button::Style {
                    background: if app.copy_buffer.is_some() {
                        Some(Background::Color(Color::from_rgb8(0x3A, 0x8A, 0x5A)))
                    } else {
                        Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                    },
                    text_color: if app.copy_buffer.is_some() {
                        Color::WHITE
                    } else {
                        Color::from_rgb8(0x8E, 0x94, 0x9D)
                    },
                    border: Border {
                        color: if app.copy_buffer.is_some() {
                            Color::from_rgb8(0x5A, 0xC0, 0x5A)
                        } else {
                            Color::from_rgb8(0x4A, 0x4A, 0x4C)
                        },
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }
            ),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Utility", copy_paste_col));

    top_panel_shell(panels)
}

fn gate_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return top_panel_shell(row![text("Main stereo has no noise gate").size(14)]);
        }
    };
    let target = VISIBLE_STRIPS[index];

    if !matches!(target, FaderTarget::Channel(_)) {
        return top_panel_shell(row![text("No noise gate for this strip").size(14)]);
    }

    let base = strip_base_path(target);

    let on = param_bool(app, &format!("{base}/gate/on"));
    let thr = param_float(app, &format!("{base}/gate/thr"));
    let range = param_float(app, &format!("{base}/gate/range"));
    let attack = param_float(app, &format!("{base}/gate/attack"));
    let hold = param_float(app, &format!("{base}/gate/hold"));
    let release = param_float(app, &format!("{base}/gate/release"));

    let mode = match app.parameter_values.get(&format!("{base}/gate/mode")) {
        Some(OscValue::Int(v)) => *v,
        _ => 3, // default GATE
    };
    let auto = param_bool(app, &format!("{base}/gate/auto"));
    let keysrc = match app.parameter_values.get(&format!("{base}/gate/keysrc")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let filter_on = param_bool(app, &format!("{base}/gate/filter/on"));
    let filter_type = match app
        .parameter_values
        .get(&format!("{base}/gate/filter/type"))
    {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let filter_f = param_float(app, &format!("{base}/gate/filter/f"));

    top_panel_shell(row![
        detail_panel(
            "Gate",
            column![
                param_toggle("On", format!("{base}/gate/on"), on),
                cycle_button(
                    "Mode",
                    format!("{base}/gate/mode"),
                    mode,
                    &["EXP2", "EXP3", "EXP4", "GATE", "DUCK"]
                ),
                param_toggle("Auto", format!("{base}/gate/auto"), auto),
                param_slider_labeled(
                    "Threshold",
                    format!("{base}/gate/thr"),
                    thr,
                    |v| format_db1(linf_value(v, -80.0, 0.0))
                ),
                param_slider_labeled(
                    "Range",
                    format!("{base}/gate/range"),
                    range,
                    |v| format_db1(linf_value(v, 3.0, 60.0))
                ),
                param_slider_labeled(
                    "Keysrc",
                    format!("{base}/gate/keysrc"),
                    keysrc as f32 / 66.0,
                    |v| key_source_name((v * 66.0) as i32)
                ),
            ]
            .spacing(6)
        ),
        detail_panel(
            "Envelope",
            column![
                param_slider_labeled("Attack", format!("{base}/gate/attack"), attack, |v| {
                    format_ms(linf_value(v, 0.0, 120.0))
                }),
                param_slider_labeled("Hold", format!("{base}/gate/hold"), hold, |v| format_ms(
                    logf_value(v, 0.02, 2000.0)
                )),
                param_slider_labeled("Release", format!("{base}/gate/release"), release, |v| {
                    format_ms(logf_value(v, 5.0, 4000.0))
                }),
            ]
            .spacing(8)
        ),
        detail_panel(
            "Filter",
            column![
                param_toggle("On", format!("{base}/gate/filter/on"), filter_on),
                cycle_button(
                    "Type",
                    format!("{base}/gate/filter/type"),
                    filter_type,
                    &[
                        "LC6", "LC12", "HC6", "HC12", "1.0", "2.0", "3.0", "5.0", "10.0"
                    ]
                ),
                param_slider_labeled("Freq", format!("{base}/gate/filter/f"), filter_f, |v| {
                    format_hz(logf_value(v, 20.0, 20000.0))
                }),
            ]
            .spacing(6)
        ),
    ])
}

fn dyn_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return dyn_detail_panel_for_base(app, "/main/st".to_owned());
        }
    };
    let target = VISIBLE_STRIPS[index];

    if matches!(
        target,
        FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Dca(_)
    ) {
        return top_panel_shell(row![text("No dynamics for this strip").size(14)]);
    }

    let base = strip_base_path(target);
    dyn_detail_panel_for_base(app, base)
}

fn cycle_button(
    label: &'static str,
    path: String,
    current: i32,
    names: &[&'static str],
) -> Element<'static, Message> {
    let name = names.get(current as usize).copied().unwrap_or("?");
    let next = ((current + 1) % names.len() as i32).max(0);
    row![
        text(label)
            .size(11)
            .width(Length::Fixed(40.0))
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        button(text(name).size(11))
            .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
            .padding([2, 6])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Background::Color(Color::from_rgb8(0x2A, 0x2D, 0x33))),
                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                border: Border {
                    color: Color::from_rgb8(0x4A, 0x4D, 0x52),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .into()
}

fn dyn_detail_panel_for_base<'a>(app: &'a StatusApp, base: String) -> Element<'a, Message> {
    let on = param_bool(app, &format!("{base}/dyn/on"));
    let thr = param_float(app, &format!("{base}/dyn/thr"));
    let ratio = param_float(app, &format!("{base}/dyn/ratio")) as i32;
    let knee = param_float(app, &format!("{base}/dyn/knee"));
    let mgain = param_float(app, &format!("{base}/dyn/mgain"));
    let attack = param_float(app, &format!("{base}/dyn/attack"));
    let hold = param_float(app, &format!("{base}/dyn/hold"));
    let release = param_float(app, &format!("{base}/dyn/release"));
    let mix = param_float(app, &format!("{base}/dyn/mix"));
    let mode = match app.parameter_values.get(&format!("{base}/dyn/mode")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let det = match app.parameter_values.get(&format!("{base}/dyn/det")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let env = match app.parameter_values.get(&format!("{base}/dyn/env")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let pos = param_bool(app, &format!("{base}/dyn/pos"));
    let keysrc = match app.parameter_values.get(&format!("{base}/dyn/keysrc")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let auto = param_bool(app, &format!("{base}/dyn/auto"));
    let filter_on = param_bool(app, &format!("{base}/dyn/filter/on"));
    let filter_type = match app.parameter_values.get(&format!("{base}/dyn/filter/type")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let filter_f = param_float(app, &format!("{base}/dyn/filter/f"));

    top_panel_shell(row![
        detail_panel(
            "Dynamics",
            column![
                param_toggle("On", format!("{base}/dyn/on"), on),
                cycle_button("Mode", format!("{base}/dyn/mode"), mode, &["COMP", "EXP"]),
                cycle_button("Det", format!("{base}/dyn/det"), det, &["PEAK", "RMS"]),
                cycle_button("Env", format!("{base}/dyn/env"), env, &["LIN", "LOG"]),
                param_toggle("Auto", format!("{base}/dyn/auto"), auto),
                param_slider_labeled("Threshold", format!("{base}/dyn/thr"), thr, |v| format_db1(
                    linf_value(v, -80.0, 0.0)
                )),
                cycle_button(
                    "Ratio",
                    format!("{base}/dyn/ratio"),
                    ratio,
                    &[
                        "1.1", "1.3", "1.5", "2.0", "2.5", "3.0", "4.0", "5.0", "7.0", "10", "20",
                        "100"
                    ]
                ),
                param_slider_labeled("Knee", format!("{base}/dyn/knee"), knee, |v| format!(
                    "{:.1}",
                    linf_value(v, 0.0, 5.0)
                )),
            ]
            .spacing(5)
        ),
        detail_panel(
            "Envelope",
            column![
                param_slider_labeled("Attack", format!("{base}/dyn/attack"), attack, |v| {
                    format_ms(linf_value(v, 0.0, 120.0))
                }),
                param_slider_labeled("Hold", format!("{base}/dyn/hold"), hold, |v| format_ms(
                    logf_value(v, 0.02, 2000.0)
                )),
                param_slider_labeled("Release", format!("{base}/dyn/release"), release, |v| {
                    format_ms(logf_value(v, 5.0, 4000.0))
                }),
                param_toggle("Post", format!("{base}/dyn/pos"), pos),
                param_slider_labeled(
                    "Keysrc",
                    format!("{base}/dyn/keysrc"),
                    keysrc as f32 / 66.0,
                    |v| key_source_name((v * 66.0) as i32)
                ),
            ]
            .spacing(6)
        ),
        detail_panel(
            "Output",
            column![
                param_slider_labeled("Gain", format!("{base}/dyn/mgain"), mgain, |v| format_db1(
                    linf_value(v, 0.0, 24.0)
                )),
                param_slider_labeled("Mix", format!("{base}/dyn/mix"), mix, format_pct),
            ]
            .spacing(8)
        ),
        detail_panel(
            "Filter",
            column![
                param_toggle("On", format!("{base}/dyn/filter/on"), filter_on),
                cycle_button(
                    "Type",
                    format!("{base}/dyn/filter/type"),
                    filter_type,
                    &[
                        "LC6", "LC12", "HC6", "HC12", "1.0", "2.0", "3.0", "5.0", "10.0"
                    ]
                ),
                param_slider_labeled("Freq", format!("{base}/dyn/filter/f"), filter_f, |v| {
                    format_hz(logf_value(v, 20.0, 20000.0))
                }),
            ]
            .spacing(6)
        ),
    ])
}

fn eq_band_count(target: FaderTarget) -> u8 {
    match target {
        FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => 4,
        FaderTarget::Bus(_) | FaderTarget::Mtx(_) | FaderTarget::Main => 6,
        FaderTarget::Dca(_) => 0,
    }
}

fn eq_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return eq_detail_panel_for_base(app, "/main/st".to_owned(), 6);
        }
    };
    let target = VISIBLE_STRIPS[index];
    let base = strip_base_path(target);
    let bands = eq_band_count(target);

    if bands == 0 {
        return top_panel_shell(row![text("No EQ for this strip").size(14)]);
    }

    eq_detail_panel_for_base(app, base, bands)
}

fn eq_detail_panel_for_base<'a>(
    app: &'a StatusApp,
    base: String,
    bands: u8,
) -> Element<'a, Message> {
    let eq_on = param_bool(app, &format!("{base}/eq/on"));
    let eq_toggle = param_toggle("EQ", format!("{base}/eq/on"), eq_on);

    let bands_row: Element<'_, Message> = {
        let mut row = row!().spacing(6);
        for band in 1..=bands {
            let band_base = format!("{base}/eq/{band:02}");
            let f = param_float(app, &format!("{band_base}/f"));
            let g = param_float(app, &format!("{band_base}/g"));
            let q = param_float(app, &format!("{band_base}/q"));
            let on = param_bool(app, &format!("{band_base}/on"));

            let col = column![
                text(format!("Band {band}"))
                    .size(11)
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                param_toggle("On", format!("{band_base}/on"), on),
                param_slider_labeled("Freq", format!("{band_base}/f"), f, |v| format_hz(
                    logf_value(v, 20.0, 20000.0)
                )),
                param_slider_labeled("Gain", format!("{band_base}/g"), g, |v| format_db(
                    linf_value(v, -15.0, 15.0)
                )),
                param_slider_labeled("Q", format!("{band_base}/q"), q, |v| format_q(logf_value(
                    v, 10.0, 0.3
                ))),
            ]
            .spacing(3)
            .width(Length::FillPortion(1));
            row = row.push(col);
        }
        row.into()
    };

    top_panel_shell(
        column![
            row![eq_toggle, text("Equalizer").size(14)]
                .spacing(12)
                .align_y(iced::Alignment::Center),
            bands_row,
        ]
        .spacing(12),
    )
}

fn sends_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => return top_panel_shell(row![text("Main sends").size(14)]),
    };
    let target = VISIBLE_STRIPS[index];
    let base = strip_base_path(target);

    let (sends, has_tap_types, has_main_lr, has_main_mono): (Vec<u8>, bool, bool, bool) =
        match target {
            FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => {
                ((1..=16).collect(), true, true, true)
            }
            FaderTarget::Bus(_) => ((1..=6).collect(), false, true, true),
            FaderTarget::Mtx(_) => (Vec::new(), false, true, false),
            FaderTarget::Dca(_) => (Vec::new(), false, true, false),
            FaderTarget::Main => (Vec::new(), false, true, true),
        };

    if sends.is_empty() && !has_main_lr {
        return top_panel_shell(row![text("No sends for this strip").size(14)]);
    }

    let mut panels = row!().spacing(8);

    // Send grid
    if !sends.is_empty() {
        let send_rows: Element<'_, Message> = sends
            .chunks(4)
            .fold(column!().spacing(6), |col, chunk| {
                let row = chunk.iter().fold(row!().spacing(6), |row, send| {
                    let send_base = format!("{base}/mix/{send:02}");
                    let level = param_float(app, &format!("{send_base}/level"));
                    let on = param_bool(app, &format!("{send_base}/on"));
                    let is_odd = send % 2 == 1;
                    let pan = if is_odd {
                        param_float(app, &format!("{send_base}/pan"))
                    } else {
                        0.5
                    };
                    let (label, stereo) = match target {
                        FaderTarget::Bus(_) => {
                            let st = if is_odd {
                                param_bool(app, &format!("/mtx/{send:02}/mix/st"))
                            } else {
                                param_bool(app, &format!("/mtx/{:02}/mix/st", send - 1))
                            };
                            (format!("Mtx {send}"), st)
                        }
                        _ => {
                            let st = if is_odd {
                                param_bool(app, &format!("/bus/{send:02}/mix/st"))
                            } else {
                                param_bool(app, &format!("/bus/{:02}/mix/st", send - 1))
                            };
                            (format!("Bus {send}"), st)
                        }
                    };
                    let mut send_col = column![
                        text(label)
                            .size(11)
                            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                        param_toggle("On", format!("{send_base}/on"), on),
                        param_slider_labeled(
                            "Level",
                            format!("{send_base}/level"),
                            level,
                            format_fader_label
                        ),
                    ]
                    .spacing(2)
                    .width(Length::Fixed(90.0));
                    if is_odd && stereo {
                        send_col = send_col.push(param_slider_labeled(
                            "Pan",
                            format!("{send_base}/pan"),
                            pan,
                            format_pan_label,
                        ));
                    }
                    row.push(send_col)
                });
                col.push(row)
            })
            .into();

        panels = panels.push(detail_panel(
            "Sends",
            scrollable(send_rows).height(Length::Fixed(200.0)),
        ));
    }

    // Tap-point selectors for bus pairs
    if has_tap_types {
        let tap_rows: Element<'_, Message> = (1..=16)
            .step_by(2)
            .fold(column!().spacing(4), |col, bus| {
                let type_path = format!("{base}/mix/{bus:02}/type");
                let current_type = match app.parameter_values.get(&type_path) {
                    Some(OscValue::Int(v)) => *v,
                    _ => 4, // default POST
                };
                col.push(tap_type_selector(bus, type_path, current_type))
            })
            .into();
        panels = panels.push(detail_panel(
            "Tap Points",
            scrollable(tap_rows).height(Length::Fixed(200.0)),
        ));
    }

    // Main LR + Mono
    if has_main_lr {
        let mut main_col = column!().spacing(8);
        if target != FaderTarget::Main {
            let lr_on = param_bool(app, &format!("{base}/mix/st"));
            let lr_fader = param_float(app, &format!("{base}/mix/fader"));
            let lr_pan = param_float(app, &format!("{base}/mix/pan"));
            main_col = main_col.push(param_toggle("LR On", format!("{base}/mix/st"), lr_on));
            main_col = main_col.push(param_slider_labeled(
                "LR Fader",
                format!("{base}/mix/fader"),
                lr_fader,
                format_fader_label,
            ));
            main_col = main_col.push(param_slider_labeled(
                "LR Pan",
                format!("{base}/mix/pan"),
                lr_pan,
                format_pan_label,
            ));
        }
        if has_main_mono
            && !matches!(
                target,
                FaderTarget::Main | FaderTarget::Mtx(_) | FaderTarget::Dca(_)
            )
        {
            let mono_on = param_bool(app, &format!("{base}/mix/mono"));
            let mono_level = param_float(app, &format!("{base}/mix/mlevel"));
            main_col = main_col.push(param_toggle("Mono On", format!("{base}/mix/mono"), mono_on));
            main_col = main_col.push(param_slider_labeled(
                "Mono Level",
                format!("{base}/mix/mlevel"),
                mono_level,
                format_fader_label,
            ));
        }
        if target == FaderTarget::Main {
            let st_on = param_bool(app, "/main/st/mix/on");
            let st_fader = param_float(app, "/main/st/mix/fader");
            let st_pan = param_float(app, "/main/st/mix/pan");
            main_col = main_col.push(param_toggle("ST On", "/main/st/mix/on".to_owned(), st_on));
            main_col = main_col.push(param_slider_labeled(
                "ST Fader",
                "/main/st/mix/fader".to_owned(),
                st_fader,
                format_fader_label,
            ));
            main_col = main_col.push(param_slider_labeled(
                "ST Pan",
                "/main/st/mix/pan".to_owned(),
                st_pan,
                format_pan_label,
            ));
        }
        if matches!(target, FaderTarget::Main | FaderTarget::Dca(_)) {
            let m_on = param_bool(app, "/main/m/mix/on");
            let m_fader = param_float(app, "/main/m/mix/fader");
            main_col = main_col.push(param_toggle("M On", "/main/m/mix/on".to_owned(), m_on));
            main_col = main_col.push(param_slider_labeled(
                "M Fader",
                "/main/m/mix/fader".to_owned(),
                m_fader,
                format_fader_label,
            ));
        }
        panels = panels.push(detail_panel("Main", main_col));
    }

    top_panel_shell(panels)
}

fn tap_type_selector<'a>(bus: u8, path: String, current: i32) -> Element<'a, Message> {
    const TAP_LABELS: [&str; 6] = ["IN/LC", "<-EQ", "EQ->", "PRE", "POST", "GRP"];
    let label = TAP_LABELS.get(current as usize).unwrap_or(&"POST");
    let next = (current + 1) % 6;
    row![
        text(format!("{bus:02}/{:02}", bus + 1))
            .size(11)
            .width(Length::Fixed(36.0))
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        button(text(*label).size(11))
            .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
            .padding([2, 6])
            .style(
                move |_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x2A, 0x2D, 0x33))),
                    text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                    border: Border {
                        color: Color::from_rgb8(0x4A, 0x4D, 0x52),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            ),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .into()
}

fn main_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return top_panel_shell(row![
                detail_panel(
                    "Main Stereo",
                    column![
                        param_toggle(
                            "On",
                            "/main/st/mix/on".to_owned(),
                            param_bool(app, "/main/st/mix/on")
                        ),
                        param_slider_labeled(
                            "Fader",
                            "/main/st/mix/fader".to_owned(),
                            param_float(app, "/main/st/mix/fader"),
                            format_fader_label
                        ),
                        param_slider_labeled(
                            "Pan",
                            "/main/st/mix/pan".to_owned(),
                            param_float(app, "/main/st/mix/pan"),
                            format_pan_label
                        ),
                    ]
                    .spacing(8)
                ),
                detail_panel(
                    "Main Mono",
                    column![
                        param_toggle(
                            "On",
                            "/main/m/mix/on".to_owned(),
                            param_bool(app, "/main/m/mix/on")
                        ),
                        param_slider_labeled(
                            "Fader",
                            "/main/m/mix/fader".to_owned(),
                            param_float(app, "/main/m/mix/fader"),
                            format_fader_label
                        ),
                    ]
                    .spacing(8)
                ),
            ]);
        }
    };
    let target = VISIBLE_STRIPS[index];
    let base = strip_base_path(target);

    let mut panels = row!().spacing(8);

    match target {
        FaderTarget::Channel(_)
        | FaderTarget::Aux(_)
        | FaderTarget::FxRtn(_)
        | FaderTarget::Bus(_) => {
            let lr_on = param_bool(app, &format!("{base}/mix/st"));
            let lr_fader = param_float(app, &format!("{base}/mix/fader"));
            let lr_pan = param_float(app, &format!("{base}/mix/pan"));
            let mono_on = param_bool(app, &format!("{base}/mix/mono"));
            let mono_level = param_float(app, &format!("{base}/mix/mlevel"));

            panels = panels.push(detail_panel(
                "Main LR",
                column![
                    param_toggle("On", format!("{base}/mix/st"), lr_on),
                    param_slider_labeled(
                        "Fader",
                        format!("{base}/mix/fader"),
                        lr_fader,
                        format_fader_label
                    ),
                    param_slider_labeled(
                        "Pan",
                        format!("{base}/mix/pan"),
                        lr_pan,
                        format_pan_label
                    ),
                ]
                .spacing(8),
            ));
            panels = panels.push(detail_panel(
                "Main Mono",
                column![
                    param_toggle("On", format!("{base}/mix/mono"), mono_on),
                    param_slider_labeled(
                        "Level",
                        format!("{base}/mix/mlevel"),
                        mono_level,
                        format_fader_label
                    ),
                ]
                .spacing(8),
            ));
        }
        FaderTarget::Mtx(_) => {
            let on = param_bool(app, &format!("{base}/mix/on"));
            let fader = param_float(app, &format!("{base}/mix/fader"));
            let pan = param_float(app, &format!("{base}/mix/pan"));
            panels = panels.push(detail_panel(
                "Matrix Output",
                column![
                    param_toggle("On", format!("{base}/mix/on"), on),
                    param_slider_labeled(
                        "Fader",
                        format!("{base}/mix/fader"),
                        fader,
                        format_fader_label
                    ),
                    param_slider_labeled("Pan", format!("{base}/mix/pan"), pan, format_pan_label),
                ]
                .spacing(8),
            ));
        }
        FaderTarget::Dca(_) => {
            let on = param_bool(app, &format!("{base}/on"));
            let fader = param_float(app, &format!("{base}/fader"));
            panels = panels.push(detail_panel(
                "DCA",
                column![
                    param_toggle("On", format!("{base}/on"), on),
                    param_slider_labeled(
                        "Fader",
                        format!("{base}/fader"),
                        fader,
                        format_fader_label
                    ),
                ]
                .spacing(8),
            ));
        }
        FaderTarget::Main => unreachable!(),
    }

    top_panel_shell(panels)
}

fn output_routing_message(
    path: String,
    byte_offset: u8,
    new_source: i32,
    app: &StatusApp,
) -> Message {
    let current = match app.parameter_values.get(&path) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let shift = byte_offset * 8;
    let new_val = (current & !(0xFF << shift)) | ((new_source & 0xFF) << shift);
    Message::ParameterChanged(path, OscValue::Int(new_val))
}

fn fx_type_name(slot: u8, fx_type: i32) -> &'static str {
    if slot <= 4 {
        const FX1_4_NAMES: [&str; 61] = [
            "HALL", "AMBI", "RPLT", "ROOM", "CHAM", "PLAT", "VREV", "VRM", "GATE", "RVRS", "DLY",
            "3TAP", "4TAP", "CRS", "FLNG", "PHAS", "DIMC", "FILT", "ROTA", "PAN", "SUB", "D/RV",
            "CR/R", "FL/R", "D/CR", "D/FL", "MODD", "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES",
            "P1A", "P1A2", "PQ5", "PQ5S", "WAVD", "LIM", "CMB", "CMB2", "FAC", "FAC1M", "FAC2",
            "LEC", "LEC2", "ULC", "ULC2", "ENH2", "ENH", "EXC2", "EXC", "IMG", "EDI", "SON",
            "AMP2", "AMP", "DRV2", "DRV", "PIT2", "PIT",
        ];
        FX1_4_NAMES
            .get(fx_type as usize)
            .copied()
            .unwrap_or("UNKNOWN")
    } else {
        const FX5_8_NAMES: [&str; 34] = [
            "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES", "P1A", "P1A2", "PQ5", "PQ5S", "WAVD",
            "LIM", "FAC", "FAC1M", "FAC2", "LEC", "LEC2", "ULC", "ULC2", "ENH2", "ENH", "EXC2",
            "EXC", "IMG", "EDI", "SON", "AMP2", "AMP", "DRV2", "DRV", "PHAS", "FILT", "PAN", "SUB",
        ];
        FX5_8_NAMES
            .get(fx_type as usize)
            .copied()
            .unwrap_or("UNKNOWN")
    }
}

fn fx_source_name(source: i32) -> &'static str {
    const SOURCES: [&str; 18] = [
        "INS", "MIX1", "MIX2", "MIX3", "MIX4", "MIX5", "MIX6", "MIX7", "MIX8", "MIX9", "MIX10",
        "MIX11", "MIX12", "MIX13", "MIX14", "MIX15", "MIX16", "M/C",
    ];
    SOURCES.get(source as usize).copied().unwrap_or("-")
}

fn fx_param_names(fx_name: &str) -> [&'static str; 8] {
    match fx_name {
        "HALL" | "AMBI" | "RPLT" | "ROOM" | "CHAM" | "PLAT" => [
            "Pre Delay",
            "Decay",
            "Size",
            "Damping",
            "Diffuse",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "VREV" => [
            "Pre Delay",
            "Decay",
            "Modulate",
            "Vintage",
            "Position",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "VRM" => [
            "Rvb Delay",
            "Decay",
            "Size",
            "Density",
            "ER Level",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "GATE" => [
            "Pre Delay",
            "Decay",
            "Attack",
            "Density",
            "Spread",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "RVRS" => [
            "Pre Delay",
            "Decay",
            "Rise",
            "Diffuse",
            "Spread",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "DLY" => [
            "Mix", "Time", "Mode", "Factor L", "Factor R", "Feedback", "Hi Cut", "X Feed",
        ],
        "3TAP" => [
            "Time", "Gain", "Pan", "Feedback", "Lo Cut", "Hi Cut", "Tap 2", "Tap 3",
        ],
        "4TAP" => [
            "Time", "Gain", "Feedback", "Lo Cut", "Hi Cut", "Tap 2", "Tap 3", "Tap 4",
        ],
        "CRS" => [
            "Speed", "Depth L", "Depth R", "Delay L", "Delay R", "Phase", "Mod", "Mix",
        ],
        "FLNG" => [
            "Speed", "Depth L", "Depth R", "Delay L", "Delay R", "Phase", "Feed", "Mix",
        ],
        "PHAS" => [
            "Speed",
            "Depth",
            "Resonance",
            "Base",
            "Stages",
            "Mix",
            "Spacing",
            "Pole",
        ],
        "DIMC" => ["Active", "Mode", "Dry", "M1", "M2", "M3", "M4", "M5"],
        "FILT" => [
            "Speed",
            "Depth",
            "Resonance",
            "Base",
            "Mode",
            "Polarity",
            "Mix",
            "Level",
        ],
        "ROTA" => [
            "Lo Speed", "Hi Speed", "Accel", "Distance", "Balance", "Mic Dist", "Mix", "Level",
        ],
        "PAN" => [
            "Speed", "Phase", "Wave", "Depth", "Env Spd", "Env Dpth", "Pan Ctr", "Mix",
        ],
        "SUB" => [
            "Active L", "Dry L", "Oct -1 L", "Oct -2 L", "Active R", "Dry R", "Oct -1 R",
            "Oct -2 R",
        ],
        "D/RV" => [
            "Time", "Pattern", "Feedback", "X Feed", "Hi Cut", "Mix", "Reverb", "D/R Lvl",
        ],
        "CR/R" => [
            "Speed", "Depth", "Delay", "Phase", "Wave", "Balance", "Reverb", "Level",
        ],
        "FL/R" => [
            "Speed", "Depth", "Delay", "Phase", "Feed", "Balance", "Reverb", "Level",
        ],
        "D/CR" => [
            "Time", "Pattern", "Hi Cut", "Feedback", "X Feed", "Mix", "Chorus", "D/C Lvl",
        ],
        "D/FL" => [
            "Time", "Pattern", "Hi Cut", "Feedback", "X Feed", "Mix", "Flanger", "D/F Lvl",
        ],
        "MODD" => [
            "Time", "Delay", "Feed", "Lo Cut", "Hi Cut", "Mod Spd", "Mod Dpth", "Mix",
        ],
        "GEQ2" | "GEQ" | "TEQ2" | "TEQ" => [
            "Band 1", "Band 2", "Band 3", "Band 4", "Band 5", "Band 6", "Band 7", "Band 8",
        ],
        "DES2" => [
            "Lo A", "Hi A", "Lo B", "Hi B", "Voice A", "Voice B", "In Gain", "Out Gain",
        ],
        "DES" => [
            "Lo L", "Hi L", "Lo R", "Hi R", "Voice L", "Voice R", "In Gain", "Out Gain",
        ],
        "P1A" => [
            "Active",
            "Gain",
            "Lo Boost",
            "Lo Freq",
            "Mid W",
            "Mid Boost",
            "Mid Freq",
            "Hi Boost",
        ],
        "P1A2" => [
            "Act A",
            "Gain A",
            "Lo A",
            "Lo F A",
            "Mid A",
            "Mid Bst A",
            "Mid F A",
            "Hi A",
        ],
        "PQ5" => [
            "Active",
            "Gain",
            "Lo Freq",
            "Mid Boost",
            "Hi Freq",
            "Hi Boost",
            "In Gain",
            "Out Gain",
        ],
        "PQ5S" => [
            "Act A", "Gain A", "Lo F A", "Mid A", "Hi F A", "Hi Bst A", "In Gain", "Out Gain",
        ],
        "WAVD" => ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8"],
        "LIM" => [
            "In Gain",
            "Out Gain",
            "Squeeze",
            "Knee",
            "Attack",
            "Release",
            "Stereo Lk",
            "Auto Gain",
        ],
        "CMB" => ["Active", "Solo", "P1", "P2", "P3", "P4", "P5", "P6"],
        "CMB2" => ["Act A", "Solo A", "P1", "P2", "P3", "P4", "P5", "P6"],
        "FAC" | "FAC1M" => [
            "Active",
            "In Gain",
            "Threshold",
            "Time",
            "Bias",
            "Gain",
            "P7",
            "P8",
        ],
        "FAC2" => [
            "Act A",
            "In Gain A",
            "Thr A",
            "Time A",
            "Bias A",
            "Gain A",
            "P7",
            "P8",
        ],
        "LEC" => ["Active", "Gain", "Peak", "Mode", "Gain", "P6", "P7", "P8"],
        "LEC2" => [
            "Act A", "Gain A", "Peak A", "Mode A", "Gain A", "P6", "P7", "P8",
        ],
        "ULC" => [
            "Active", "In Gain", "Out Gain", "Attack", "Release", "Ratio", "P7", "P8",
        ],
        "ULC2" => [
            "Act A",
            "In Gain A",
            "Out A",
            "Att A",
            "Rel A",
            "Ratio A",
            "P7",
            "P8",
        ],
        "ENH2" => [
            "Out A",
            "Speed A",
            "Bass A",
            "Bass F A",
            "Mid A",
            "Mid F A",
            "Treble A",
            "Treble F A",
        ],
        "ENH" => [
            "Out Gain", "Speed", "Bass", "Bass F", "Mid", "Mid F", "Treble", "Treble F",
        ],
        "EXC2" => [
            "Tune A", "Peak A", "Zero A", "Timbre A", "Harm A", "Mix A", "P7", "P8",
        ],
        "EXC" => ["Tune", "Peak", "Zero", "Timbre", "Harm", "Mix", "P7", "P8"],
        "IMG" => [
            "Balance",
            "Mono Pan",
            "Stereo Pan",
            "Shv Gain",
            "Shv Freq",
            "Width",
            "Mix",
            "Level",
        ],
        "EDI" => [
            "Active", "In Gain", "Out Gain", "Attack", "Release", "Ratio", "P7", "P8",
        ],
        "SON" => ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8"],
        "AMP2" => [
            "Pre A", "Buzz A", "Punch A", "Crunch A", "Drive A", "Low A", "Mid A", "High A",
        ],
        "AMP" => [
            "Preamp", "Buzz", "Punch", "Crunch", "Drive", "Low", "Mid", "High",
        ],
        "DRV2" => [
            "Drive A", "Even A", "Odd A", "Gain A", "Lo Cut A", "Hi Cut A", "Mix A", "Level A",
        ],
        "DRV" => [
            "Drive", "Even", "Odd", "Gain", "Lo Cut", "Hi Cut", "Mix", "Level",
        ],
        "PIT2" => [
            "Semi A", "Cent A", "Delay A", "Lo Cut A", "Hi Cut A", "Mix A", "P7", "P8",
        ],
        "PIT" => [
            "Semitone", "Cent", "Delay", "Lo Cut", "Hi Cut", "Mix", "P7", "P8",
        ],
        _ => ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8"],
    }
}

fn fx_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let mut slots = row!().spacing(6);
    for slot in 1..=8 {
        let base = format!("/fx/{slot:02}");
        let fx_type = match app.parameter_values.get(&format!("{base}/type")) {
            Some(OscValue::Int(t)) => fx_type_name(slot, *t),
            _ => "-",
        };
        let source_l = match app.parameter_values.get(&format!("{base}/source/l")) {
            Some(OscValue::Int(v)) => fx_source_name(*v),
            _ => "-",
        };
        let source_r = match app.parameter_values.get(&format!("{base}/source/r")) {
            Some(OscValue::Int(v)) => fx_source_name(*v),
            _ => "-",
        };

        let mut col = column![
            text(format!("FX {slot}"))
                .size(12)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            text(fx_type)
                .size(11)
                .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        ]
        .spacing(2)
        .width(Length::Fixed(90.0));

        if slot <= 4 {
            col = col.push(text(format!("L: {source_l}")).size(10));
            col = col.push(text(format!("R: {source_r}")).size(10));
        }

        // Show 8 parameters with effect-specific names
        let param_names = fx_param_names(fx_type);
        for par in 1..=8 {
            let par_path = format!("{base}/par/{par:02}");
            let par_val = param_float(app, &par_path);
            let par_name = param_names.get(par - 1).copied().unwrap_or("P?");
            col = col.push(
                row![
                    text(par_name).size(9).width(Length::Fixed(36.0)),
                    horizontal_slider(0.0..=1.0, par_val, move |v| {
                        Message::ParameterChanged(par_path.clone(), OscValue::Float(v))
                    })
                    .fill_from_start()
                    .step(0.01)
                    .width(Length::Fixed(50.0))
                    .height(Length::Fixed(12.0)),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center),
            );
        }

        slots = slots.push(col);
    }

    top_panel_shell(
        scrollable(slots).direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new(),
        )),
    )
}

fn scenes_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let scenes_grid: Element<'_, Message> = (1..=100)
        .fold(column!().spacing(4), |mut col, scene| {
            let name_path = format!("/-show/showfile/scene/{scene:03}/name");
            let has_data_path = format!("/-show/showfile/scene/{scene:03}/hasData");
            let name = match app.parameter_values.get(&name_path) {
                Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                _ => format!("Scene {scene}"),
            };
            let has_data = param_bool(app, &has_data_path);
            let name_color = if has_data {
                Color::from_rgb8(0xC7, 0xC9, 0xD3)
            } else {
                Color::from_rgb8(0x60, 0x60, 0x60)
            };

            let safes_path = format!("/-show/showfile/scene/{scene:03}/safes");
            let safes_val = param_int(app, &safes_path);
            let has_safes = safes_val != 0;
            let notes = match app
                .parameter_values
                .get(&format!("/-show/showfile/scene/{scene:03}/notes"))
            {
                Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                _ => String::new(),
            };
            let notes_short = if notes.len() > 20 {
                format!("{}…", &notes[..20])
            } else {
                notes.clone()
            };

            let row = row![
                text(format!("{scene:03}"))
                    .size(11)
                    .width(Length::Fixed(28.0))
                    .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                text(name)
                    .size(11)
                    .width(Length::Fixed(100.0))
                    .color(name_color),
                text(notes_short)
                    .size(9)
                    .width(Length::Fixed(80.0))
                    .color(Color::from_rgb8(0x6E, 0x74, 0x7D)),
                button(text("Recall").size(10))
                    .on_press(Message::SceneRecall(scene))
                    .padding([2, 6])
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x2A, 0x5A, 0x3A))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                            width: 1.0,
                            radius: 2.0.into()
                        },
                        ..Default::default()
                    }),
                button(text("Save").size(10))
                    .on_press(Message::SceneSave(scene))
                    .padding([2, 6])
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x3A, 0x3A, 0x5A))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x5A, 0x5A, 0x8A),
                            width: 1.0,
                            radius: 2.0.into()
                        },
                        ..Default::default()
                    }),
                button(text(if has_safes { "S!" } else { "S" }).size(10))
                    .on_press(Message::EditSceneSafes(scene))
                    .padding([2, 4])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if has_safes {
                                Some(Background::Color(Color::from_rgb8(0x8A, 0x6A, 0x2A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: if has_safes {
                                    Color::from_rgb8(0xC0, 0xA0, 0x5A)
                                } else {
                                    Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                },
                                width: 1.0,
                                radius: 2.0.into()
                            },
                            ..Default::default()
                        }
                    ),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center);
            col = col.push(row);

            if app.editing_scene_safes == Some(scene) {
                let safe_labels = [
                    (1, "TB"),
                    (2, "FX"),
                    (3, "Bus"),
                    (4, "Ch"),
                    (5, "Cfg"),
                    (6, "Pre"),
                    (7, "Out"),
                    (8, "Rte"),
                ];
                let safe_row = safe_labels
                    .iter()
                    .fold(row!().spacing(2), |r, (bit, label)| {
                        let active = (safes_val & (1 << bit)) != 0;
                        let new_val = if active {
                            safes_val & !(1 << bit)
                        } else {
                            safes_val | (1 << bit)
                        };
                        r.push(
                            button(text(*label).size(9))
                                .on_press(Message::ParameterChanged(
                                    safes_path.clone(),
                                    OscValue::Int(new_val),
                                ))
                                .padding([1, 4])
                                .style(move |_theme: &Theme, _status: button::Status| {
                                    button::Style {
                                        background: if active {
                                            Some(Background::Color(Color::from_rgb8(
                                                0x8A, 0x6A, 0x2A,
                                            )))
                                        } else {
                                            Some(Background::Color(Color::from_rgb8(
                                                0x2A, 0x2A, 0x2C,
                                            )))
                                        },
                                        text_color: if active {
                                            Color::WHITE
                                        } else {
                                            Color::from_rgb8(0x8E, 0x94, 0x9D)
                                        },
                                        border: Border {
                                            color: if active {
                                                Color::from_rgb8(0xC0, 0xA0, 0x5A)
                                            } else {
                                                Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                            },
                                            width: 1.0,
                                            radius: 2.0.into(),
                                        },
                                        ..Default::default()
                                    }
                                }),
                        )
                    });
                col = col.push(safe_row);
            }
            col
        })
        .into();

    // Cues section
    let cues_grid: Element<'_, Message> = (0..100)
        .fold(column!().spacing(4), |col, cue| {
            let name_path = format!("/-show/showfile/cue/{cue:03}/name");
            let scene_path = format!("/-show/showfile/cue/{cue:03}/scene");
            let skip_path = format!("/-show/showfile/cue/{cue:03}/skip");
            let name = match app.parameter_values.get(&name_path) {
                Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                _ => return col,
            };
            let scene_idx = match app.parameter_values.get(&scene_path) {
                Some(OscValue::Int(v)) => *v,
                _ => -1,
            };
            let skipped = param_bool(app, &skip_path);
            let scene_label = if scene_idx >= 0 {
                format!("→ Sc {scene_idx:03}")
            } else {
                "—".to_owned()
            };
            let name_color = if skipped {
                Color::from_rgb8(0x60, 0x60, 0x60)
            } else {
                Color::from_rgb8(0xC7, 0xC9, 0xD3)
            };

            let midi_type = match app
                .parameter_values
                .get(&format!("/-show/showfile/cue/{cue:03}/miditype"))
            {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let midi_label = if midi_type > 0 {
                let midi_type_name = match midi_type {
                    1 => "PC",
                    2 => "CC",
                    3 => "Note",
                    _ => "?",
                };
                let midi_chan = match app
                    .parameter_values
                    .get(&format!("/-show/showfile/cue/{cue:03}/midichan"))
                {
                    Some(OscValue::Int(v)) => *v + 1,
                    _ => 0,
                };
                let midi_para1 = match app
                    .parameter_values
                    .get(&format!("/-show/showfile/cue/{cue:03}/midipara1"))
                {
                    Some(OscValue::Int(v)) => *v,
                    _ => 0,
                };
                format!("{midi_type_name} Ch{midi_chan} P{midi_para1}")
            } else {
                String::new()
            };

            let row = row![
                text(format!("{cue:03}"))
                    .size(11)
                    .width(Length::Fixed(28.0))
                    .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                text(name)
                    .size(11)
                    .width(Length::Fixed(110.0))
                    .color(name_color),
                text(scene_label)
                    .size(10)
                    .width(Length::Fixed(44.0))
                    .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                text(midi_label)
                    .size(9)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                button(text("Go").size(10))
                    .on_press(Message::SceneRecall(cue))
                    .padding([2, 6])
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x2A, 0x5A, 0x3A))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                            width: 1.0,
                            radius: 2.0.into()
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center);
            col.push(row)
        })
        .into();

    // Snippets section
    let snippets_grid: Element<'_, Message> = (0..100)
        .fold(column!().spacing(4), |col, snip| {
            let name_path = format!("/-show/showfile/snippet/{snip:03}/name");
            let has_data_path = format!("/-show/showfile/snippet/{snip:03}/hasData");
            let name = match app.parameter_values.get(&name_path) {
                Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                _ => return col,
            };
            let has_data = param_bool(app, &has_data_path);
            let name_color = if has_data {
                Color::from_rgb8(0xC7, 0xC9, 0xD3)
            } else {
                Color::from_rgb8(0x60, 0x60, 0x60)
            };

            let editing = app.editing_snippet_filters == Some(snip);
            let row = row![
                text(format!("{snip:03}"))
                    .size(11)
                    .width(Length::Fixed(28.0))
                    .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                text(name)
                    .size(11)
                    .width(Length::Fixed(110.0))
                    .color(name_color),
                button(text("Recall").size(10))
                    .on_press(Message::SnippetRecall(snip))
                    .padding([2, 5])
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x2A, 0x5A, 0x3A))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                            width: 1.0,
                            radius: 2.0.into()
                        },
                        ..Default::default()
                    }),
                button(text("Save").size(10))
                    .on_press(Message::SnippetSave(snip))
                    .padding([2, 5])
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x3A, 0x3A, 0x5A))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x5A, 0x5A, 0x8A),
                            width: 1.0,
                            radius: 2.0.into()
                        },
                        ..Default::default()
                    }),
                button(text("Filt").size(10))
                    .on_press(Message::EditSnippetFilters(snip))
                    .padding([2, 5])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if editing {
                                Some(Background::Color(Color::from_rgb8(0x8A, 0x6A, 0x2A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: if editing {
                                    Color::from_rgb8(0xC0, 0xA0, 0x5A)
                                } else {
                                    Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                },
                                width: 1.0,
                                radius: 2.0.into()
                            },
                            ..Default::default()
                        }
                    ),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center);
            let mut col = col.push(row);

            if editing {
                let eventtyp_path = format!("/-show/showfile/snippet/{snip:03}/eventtyp");
                let eventtyp_val = match app.parameter_values.get(&eventtyp_path) {
                    Some(OscValue::Int(v)) => *v as u32,
                    _ => 0,
                };
                let channels_path = format!("/-show/showfile/snippet/{snip:03}/channels");
                let channels_val = match app.parameter_values.get(&channels_path) {
                    Some(OscValue::Int(v)) => *v as u32,
                    _ => 0,
                };
                let auxbuses_path = format!("/-show/showfile/snippet/{snip:03}/auxbuses");
                let auxbuses_val = match app.parameter_values.get(&auxbuses_path) {
                    Some(OscValue::Int(v)) => *v as u32,
                    _ => 0,
                };
                let maingrps_path = format!("/-show/showfile/snippet/{snip:03}/maingrps");
                let maingrps_val = match app.parameter_values.get(&maingrps_path) {
                    Some(OscValue::Int(v)) => *v as u32,
                    _ => 0,
                };

                let make_bit_toggles = |label: &'static str,
                                        path: String,
                                        value: u32,
                                        groups: &'static [(u32, &'static str)]|
                 -> Element<'_, Message> {
                    let toggles = groups.iter().fold(row!().spacing(2), |r, (mask, name)| {
                        let active = (value & mask) != 0;
                        let new_val = if (value & mask) == *mask {
                            (value & !mask) as i32
                        } else {
                            (value | mask) as i32
                        };
                        r.push(
                            button(text(*name).size(9))
                                .on_press(Message::ParameterChanged(
                                    path.clone(),
                                    OscValue::Int(new_val),
                                ))
                                .padding([1, 3])
                                .style(move |_theme: &Theme, _status: button::Status| {
                                    button::Style {
                                        background: if active {
                                            Some(Background::Color(Color::from_rgb8(
                                                0x8A, 0x6A, 0x2A,
                                            )))
                                        } else {
                                            Some(Background::Color(Color::from_rgb8(
                                                0x2A, 0x2A, 0x2C,
                                            )))
                                        },
                                        text_color: if active {
                                            Color::WHITE
                                        } else {
                                            Color::from_rgb8(0x8E, 0x94, 0x9D)
                                        },
                                        border: Border {
                                            color: if active {
                                                Color::from_rgb8(0xC0, 0xA0, 0x5A)
                                            } else {
                                                Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                            },
                                            width: 1.0,
                                            radius: 2.0.into(),
                                        },
                                        ..Default::default()
                                    }
                                }),
                        )
                    });
                    row![
                        text(label)
                            .size(9)
                            .width(Length::Fixed(36.0))
                            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                        toggles,
                    ]
                    .spacing(4)
                    .align_y(iced::Alignment::Center)
                    .into()
                };

                let eventtyp_groups: &[(u32, &str)] = &[
                    (1 << 0, "Pre"),
                    (1 << 1, "Cfg"),
                    (1 << 2, "EQ"),
                    (1 << 3, "Dyn"),
                    (1 << 4, "Ins"),
                    (1 << 5, "Grp"),
                    (1 << 6, "Fdr"),
                    (1 << 7, "Mute"),
                    (0x1F << 8, "Snd"),
                    (0xFF << 13, "FX"),
                    (1 << 22, "Solo"),
                    (1 << 23, "Rte"),
                    (1 << 24, "Out"),
                ];
                let channels_groups: &[(u32, &str)] = &[
                    (0xFF, "1-8"),
                    (0xFF << 8, "9-16"),
                    (0xFF << 16, "17-24"),
                    (0xFF << 24, "25-32"),
                ];
                let auxbuses_groups: &[(u32, &str)] = &[
                    (0xFF, "Aux"),
                    (0xFF << 8, "FxR"),
                    (0xFF << 16, "Bus1"),
                    (0xFF << 24, "Bus9"),
                ];
                let maingrps_groups: &[(u32, &str)] =
                    &[(0x3F, "Mtx"), (0x3 << 8, "Main"), (0xFF << 16, "DCA")];

                col = col.push(make_bit_toggles(
                    "Evt",
                    eventtyp_path,
                    eventtyp_val,
                    eventtyp_groups,
                ));
                col = col.push(make_bit_toggles(
                    "Ch",
                    channels_path,
                    channels_val,
                    channels_groups,
                ));
                col = col.push(make_bit_toggles(
                    "Aux",
                    auxbuses_path,
                    auxbuses_val,
                    auxbuses_groups,
                ));
                col = col.push(make_bit_toggles(
                    "Main",
                    maingrps_path,
                    maingrps_val,
                    maingrps_groups,
                ));
            }
            col
        })
        .into();

    let show_file_row = row![
        text_input("Show filename", &app.show_file_name)
            .size(11)
            .width(Length::Fixed(180.0))
            .on_input(Message::ShowFileNameChanged),
        button(text("Load").size(10))
            .on_press(Message::ShowFileLoad)
            .padding([3, 8])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Background::Color(Color::from_rgb8(0x2A, 0x5A, 0x3A))),
                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                border: Border {
                    color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                    width: 1.0,
                    radius: 2.0.into()
                },
                ..Default::default()
            }),
        button(text("Save").size(10))
            .on_press(Message::ShowFileSave)
            .padding([3, 8])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Background::Color(Color::from_rgb8(0x3A, 0x3A, 0x5A))),
                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                border: Border {
                    color: Color::from_rgb8(0x5A, 0x5A, 0x8A),
                    width: 1.0,
                    radius: 2.0.into()
                },
                ..Default::default()
            }),
        Space::new().width(Length::Fixed(12.0)),
        button(text("Undo").size(10))
            .on_press(Message::Undo)
            .padding([3, 8])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Background::Color(Color::from_rgb8(0x5A, 0x5A, 0x3A))),
                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                border: Border {
                    color: Color::from_rgb8(0x8A, 0x8A, 0x5A),
                    width: 1.0,
                    radius: 2.0.into()
                },
                ..Default::default()
            }),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    top_panel_shell(
        column![
            text("Show File")
                .size(12)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            show_file_row,
            text("Scenes")
                .size(12)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            scrollable(scenes_grid).height(Length::Fixed(100.0)),
            text("Cues")
                .size(12)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            scrollable(cues_grid).height(Length::Fixed(60.0)),
            text("Snippets")
                .size(12)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            scrollable(snippets_grid).height(Length::Fixed(60.0)),
        ]
        .spacing(4),
    )
}

fn setup_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let mut panels = row!().spacing(8);

    // Talkback
    let talk_enable = param_bool(app, "/config/talk/enable");
    let talk_source = match app.parameter_values.get("/config/talk/source") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let talk_a_level = param_float(app, "/config/talk/A/level");
    let talk_a_latch = param_bool(app, "/config/talk/A/latch");
    let talk_a_dim = param_bool(app, "/config/talk/A/dim");
    let talk_a_destmap = match app.parameter_values.get("/config/talk/A/destmap") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let talk_b_level = param_float(app, "/config/talk/B/level");
    let talk_b_latch = param_bool(app, "/config/talk/B/latch");
    let talk_b_dim = param_bool(app, "/config/talk/B/dim");
    let talk_b_destmap = match app.parameter_values.get("/config/talk/B/destmap") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };

    let make_talk_dests = |path: String, value: i32| -> Element<'_, Message> {
        let dest_labels: [(u32, &str); 18] = [
            (1 << 0, "B1"),
            (1 << 1, "B2"),
            (1 << 2, "B3"),
            (1 << 3, "B4"),
            (1 << 4, "B5"),
            (1 << 5, "B6"),
            (1 << 6, "B7"),
            (1 << 7, "B8"),
            (1 << 8, "B9"),
            (1 << 9, "B10"),
            (1 << 10, "B11"),
            (1 << 11, "B12"),
            (1 << 12, "B13"),
            (1 << 13, "B14"),
            (1 << 14, "B15"),
            (1 << 15, "B16"),
            (1 << 16, "LR"),
            (1 << 17, "MC"),
        ];
        let rows = dest_labels
            .chunks(6)
            .fold(column!().spacing(2), |col, chunk| {
                let row = chunk.iter().fold(row!().spacing(2), |r, (mask, label)| {
                    let active = (value as u32 & mask) != 0;
                    let new_val = if active {
                        value & !(*mask as i32)
                    } else {
                        value | (*mask as i32)
                    };
                    r.push(
                        button(text(*label).size(8))
                            .on_press(Message::ParameterChanged(
                                path.clone(),
                                OscValue::Int(new_val),
                            ))
                            .padding([1, 3])
                            .style(
                                move |_theme: &Theme, _status: button::Status| button::Style {
                                    background: if active {
                                        Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                                    } else {
                                        Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                                    },
                                    text_color: if active {
                                        Color::WHITE
                                    } else {
                                        Color::from_rgb8(0x8E, 0x94, 0x9D)
                                    },
                                    border: Border {
                                        color: if active {
                                            Color::from_rgb8(0x5A, 0x8A, 0x5A)
                                        } else {
                                            Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                        },
                                        width: 1.0,
                                        radius: 2.0.into(),
                                    },
                                    ..Default::default()
                                },
                            ),
                    )
                });
                col.push(row)
            });
        rows.into()
    };

    let talk_col = column![
        param_toggle("Talk Enable", "/config/talk/enable".to_owned(), talk_enable),
        param_slider_labeled(
            "Talk Src",
            "/config/talk/source".to_owned(),
            talk_source as f32 / 37.0,
            |v| format!("{:.0}", v * 37.0)
        ),
        text("Talkback A")
            .size(12)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        param_slider_labeled(
            "Level",
            "/config/talk/A/level".to_owned(),
            talk_a_level,
            format_fader_label
        ),
        param_toggle("Latch", "/config/talk/A/latch".to_owned(), talk_a_latch),
        param_toggle("Dim", "/config/talk/A/dim".to_owned(), talk_a_dim),
        make_talk_dests("/config/talk/A/destmap".to_owned(), talk_a_destmap),
        text("Talkback B")
            .size(12)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        param_slider_labeled(
            "Level",
            "/config/talk/B/level".to_owned(),
            talk_b_level,
            format_fader_label
        ),
        param_toggle("Latch", "/config/talk/B/latch".to_owned(), talk_b_latch),
        param_toggle("Dim", "/config/talk/B/dim".to_owned(), talk_b_dim),
        make_talk_dests("/config/talk/B/destmap".to_owned(), talk_b_destmap),
    ]
    .spacing(5);
    panels = panels.push(detail_panel("Talkback", talk_col));

    // Oscillator
    let osc_type = match app.parameter_values.get("/config/osc/type") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let osc_f = param_float(app, "/config/osc/f");
    let osc_fsel = match app.parameter_values.get("/config/osc/fsel") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let osc_level = param_float(app, "/config/osc/level");
    let osc_dest = match app.parameter_values.get("/config/osc/dest") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };

    let osc_col = column![
        cycle_button(
            "Type",
            "/config/osc/type".to_owned(),
            osc_type,
            &["SINE", "PINK", "WHITE"]
        ),
        param_slider_labeled("Freq", "/config/osc/f".to_owned(), osc_f, |v| format_hz(
            logf_value(v, 20.0, 20000.0)
        )),
        cycle_button(
            "F Sel",
            "/config/osc/fsel".to_owned(),
            osc_fsel,
            &["F1", "F2"]
        ),
        param_slider_labeled(
            "Level",
            "/config/osc/level".to_owned(),
            osc_level,
            format_fader_label
        ),
        param_slider_labeled(
            "Dest",
            "/config/osc/dest".to_owned(),
            osc_dest as f32 / 25.0,
            |v| format!("{:.0}", v * 25.0)
        ),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Oscillator", osc_col));

    // Solo / Monitor
    let solo_level = param_float(app, "/config/solo/level");
    let solo_source = match app.parameter_values.get("/config/solo/source") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_sourcetrim = match app.parameter_values.get("/config/solo/sourcetrim") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_chmode = match app.parameter_values.get("/config/solo/chmode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_busmode = match app.parameter_values.get("/config/solo/busmode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_dcamode = match app.parameter_values.get("/config/solo/dcamode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_exclusive = param_bool(app, "/config/solo/exclusive");
    let solo_followsel = param_bool(app, "/config/solo/followsel");
    let solo_followsolo = param_bool(app, "/config/solo/followsolo");
    let solo_dimatt = match app.parameter_values.get("/config/solo/dimatt") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_dim = param_bool(app, "/config/solo/dim");
    let solo_mono = param_bool(app, "/config/solo/mono");
    let solo_delay = param_bool(app, "/config/solo/delay");
    let solo_delaytime = match app.parameter_values.get("/config/solo/delaytime") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_masterctrl = param_bool(app, "/config/solo/masterctrl");
    let solo_mute = param_bool(app, "/config/solo/mute");
    let solo_dimpfl = param_bool(app, "/config/solo/dimpfl");

    let solo_col = column![
        param_slider_labeled(
            "Level",
            "/config/solo/level".to_owned(),
            solo_level,
            format_fader_label
        ),
        param_slider_labeled(
            "Source",
            "/config/solo/source".to_owned(),
            solo_source as f32 / 37.0,
            |v| format!("{:.0}", v * 37.0)
        ),
        param_slider_labeled(
            "Trim",
            "/config/solo/sourcetrim".to_owned(),
            solo_sourcetrim as f32 / 18.0,
            |v| format!("{:.0}", v * 18.0)
        ),
        cycle_button(
            "Ch Mode",
            "/config/solo/chmode".to_owned(),
            solo_chmode,
            &["AFL", "PFL"]
        ),
        cycle_button(
            "Bus Mode",
            "/config/solo/busmode".to_owned(),
            solo_busmode,
            &["AFL", "PFL"]
        ),
        cycle_button(
            "DCA Mode",
            "/config/solo/dcamode".to_owned(),
            solo_dcamode,
            &["AFL", "PFL"]
        ),
        param_toggle(
            "Exclusive",
            "/config/solo/exclusive".to_owned(),
            solo_exclusive
        ),
        param_toggle(
            "Follow Sel",
            "/config/solo/followsel".to_owned(),
            solo_followsel
        ),
        param_toggle(
            "Follow Solo",
            "/config/solo/followsolo".to_owned(),
            solo_followsolo
        ),
        param_slider_labeled(
            "Dim Att",
            "/config/solo/dimatt".to_owned(),
            solo_dimatt as f32 / 40.0,
            |v| format!("{:.0} dB", v * 40.0)
        ),
        param_toggle("Dim", "/config/solo/dim".to_owned(), solo_dim),
        param_toggle("Mono", "/config/solo/mono".to_owned(), solo_mono),
        param_toggle("Delay", "/config/solo/delay".to_owned(), solo_delay),
        param_slider_labeled(
            "Dly Time",
            "/config/solo/delaytime".to_owned(),
            solo_delaytime as f32 / 500.0,
            |v| format_ms(v * 500.0)
        ),
        param_toggle(
            "Mst Ctrl",
            "/config/solo/masterctrl".to_owned(),
            solo_masterctrl
        ),
        param_toggle("Mute", "/config/solo/mute".to_owned(), solo_mute),
        param_toggle("Dim PFL", "/config/solo/dimpfl".to_owned(), solo_dimpfl),
    ]
    .spacing(4);
    panels = panels.push(detail_panel("Solo / Mon", solo_col));

    // Sends on Fader
    let sends_on_fader = param_bool(app, "/-stat/sends on fader");
    let sof_col = column![param_toggle(
        "Sends on Fdr",
        "/-stat/sends on fader".to_owned(),
        sends_on_fader
    ),]
    .spacing(6);
    panels = panels.push(detail_panel("Sends on Fdr", sof_col));

    // GEQ on Faders
    let geq_on_fader = param_bool(app, "/-stat/geqonfdr");
    let geq_pos = match app.parameter_values.get("/-stat/geqpos") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let geq_fx_slot = (geq_pos >> 8) as u8;
    let geq_window = (geq_pos & 0xFF) as u8;
    let geq_pos_text = if geq_on_fader {
        format!("FX{} Win{}", geq_fx_slot, geq_window)
    } else {
        "—".to_owned()
    };
    let geq_col = column![
        param_toggle("GEQ on Fdr", "/-stat/geqonfdr".to_owned(), geq_on_fader),
        text(geq_pos_text)
            .size(10)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("GEQ", geq_col));

    // Recorder
    let rec_state = match app.parameter_values.get("/-stat/urec/state") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let rec_state_name = match rec_state {
        0 => "STOP",
        1 => "PAUSE",
        2 => "PLAY",
        3 => "REC",
        _ => "?",
    };
    let rec_rtime = match app.parameter_values.get("/-stat/urec/rtime") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let rec_etime = match app.parameter_values.get("/-stat/urec/etime") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };

    let rec_col = column![
        text(format!("State: {rec_state_name}"))
            .size(12)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        text(format!("RTime: {rec_rtime}s"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        text(format!("ETime: {rec_etime}s"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        row![
            button(text("REC").size(11))
                .on_press(Message::RecorderAction("recrun"))
                .padding([4, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x8A, 0x3A, 0x3A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0xC0, 0x5A, 0x5A),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
            button(text("STOP").size(11))
                .on_press(Message::RecorderAction("recstop"))
                .padding([4, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x3A, 0x3A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x5A, 0x5A),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(4),
        row![
            button(text("PLAY").size(11))
                .on_press(Message::RecorderAction("playrun"))
                .padding([4, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x8A, 0x5A),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
            button(text("P/STOP").size(11))
                .on_press(Message::RecorderAction("playstop"))
                .padding([4, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x3A, 0x3A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x5A, 0x5A),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(4),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Recorder", rec_col));

    // Mono Link
    let mono_link = param_bool(app, "/config/mono/link");
    let mono_col = column![param_toggle(
        "Mono Link",
        "/config/mono/link".to_owned(),
        mono_link
    ),]
    .spacing(6);
    panels = panels.push(detail_panel("Mono Link", mono_col));

    // Network / Clock
    let ip_dhcp = param_bool(app, "/-prefs/ip/dhcp");
    let clock_source = match app.parameter_values.get("/-prefs/clocksource") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let clock_rate = match app.parameter_values.get("/-prefs/clockrate") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let clock_mode = match app.parameter_values.get("/-prefs/clockmode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let net_col = column![
        param_toggle("DHCP", "/-prefs/ip/dhcp".to_owned(), ip_dhcp),
        cycle_button(
            "Clock Src",
            "/-prefs/clocksource".to_owned(),
            clock_source,
            &["Int", "AES50A", "AES50B", "Card"]
        ),
        cycle_button(
            "Clock Rate",
            "/-prefs/clockrate".to_owned(),
            clock_rate,
            &["44.1k", "48k"]
        ),
        cycle_button(
            "Clock Mode",
            "/-prefs/clockmode".to_owned(),
            clock_mode,
            &["Single", "Double", "Quad"]
        ),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Network", net_col));

    // Tape / USB
    let tape_autoplay = param_bool(app, "/config/tape/autoplay");
    let tape_col = column![param_toggle(
        "Autoplay",
        "/config/tape/autoplay".to_owned(),
        tape_autoplay
    ),]
    .spacing(6);
    panels = panels.push(detail_panel("Tape", tape_col));

    // Preferences
    let bright = param_float(app, "/-prefs/bright");
    let lcdcont = param_float(app, "/-prefs/lcdcont");
    let ledbright = param_float(app, "/-prefs/ledbright");
    let lamp = param_float(app, "/-prefs/lamp");
    let lampon = param_bool(app, "/-prefs/lampon");
    let confirm_general = param_bool(app, "/-prefs/confirm_general");
    let confirm_overwrite = param_bool(app, "/-prefs/confirm_overwrite");
    let confirm_sceneload = param_bool(app, "/-prefs/confirm_sceneload");
    let remote_enable = param_bool(app, "/-prefs/remote/enable");
    let remote_protocol = match app.parameter_values.get("/-prefs/remote/protocol") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let remote_port = match app.parameter_values.get("/-prefs/remote/port") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let card_ufifc = match app.parameter_values.get("/-prefs/card/UFifc") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let card_ufmode = match app.parameter_values.get("/-prefs/card/UFmode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let fast_faders = param_bool(app, "/-prefs/fastFaders");
    let hard_mute = param_bool(app, "/-prefs/hardmute");
    let dca_mute = param_bool(app, "/-prefs/dcamute");
    let invert_mutes = param_bool(app, "/-prefs/invertmutes");
    let safe_master = param_bool(app, "/-prefs/safe_masterlevels");
    let view_rtn = param_bool(app, "/-prefs/viewrtn");
    let scene_advance = param_bool(app, "/-prefs/scene_advance");
    let ha_flags = match app.parameter_values.get("/-prefs/haflags") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let show_control = match app.parameter_values.get("/-prefs/show_control") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let rec_control = match app.parameter_values.get("/-prefs/rec_control") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };

    let prefs_col = column![
        param_slider_labeled("Bright", "/-prefs/bright".to_owned(), bright, |v| format!(
            "{:.0}%",
            v * 100.0
        )),
        param_slider_labeled("LCD", "/-prefs/lcdcont".to_owned(), lcdcont, |v| format!(
            "{:.0}%",
            v * 100.0
        )),
        param_slider_labeled(
            "LED",
            "/-prefs/ledbright".to_owned(),
            ledbright,
            |v| format!("{:.0}%", v * 100.0)
        ),
        param_slider_labeled("Lamp", "/-prefs/lamp".to_owned(), lamp, |v| format!(
            "{:.0}%",
            v * 100.0
        )),
        param_toggle("Lamp On", "/-prefs/lampon".to_owned(), lampon),
        param_toggle(
            "Confirm Gen",
            "/-prefs/confirm_general".to_owned(),
            confirm_general
        ),
        param_toggle(
            "Confirm Ovw",
            "/-prefs/confirm_overwrite".to_owned(),
            confirm_overwrite
        ),
        param_toggle(
            "Confirm Scene",
            "/-prefs/confirm_sceneload".to_owned(),
            confirm_sceneload
        ),
        param_toggle(
            "Remote En",
            "/-prefs/remote/enable".to_owned(),
            remote_enable
        ),
        cycle_button(
            "Rem Proto",
            "/-prefs/remote/protocol".to_owned(),
            remote_protocol,
            &["MIDI", "OSC"]
        ),
        param_slider_labeled(
            "Rem Port",
            "/-prefs/remote/port".to_owned(),
            remote_port as f32 / 65535.0,
            |v| format!("{:.0}", v * 65535.0)
        ),
        cycle_button(
            "Card IF",
            "/-prefs/card/UFifc".to_owned(),
            card_ufifc,
            &["USB", "FW"]
        ),
        cycle_button(
            "Card Mode",
            "/-prefs/card/UFmode".to_owned(),
            card_ufmode,
            &["Player", "Rec", "Both"]
        ),
        param_toggle("Fast Faders", "/-prefs/fastFaders".to_owned(), fast_faders),
        param_toggle("Hard Mute", "/-prefs/hardmute".to_owned(), hard_mute),
        param_toggle("DCA Mute", "/-prefs/dcamute".to_owned(), dca_mute),
        param_toggle("Inv Mutes", "/-prefs/invertmutes".to_owned(), invert_mutes),
        param_toggle(
            "Safe Mst",
            "/-prefs/safe_masterlevels".to_owned(),
            safe_master
        ),
        param_toggle("View Rtn", "/-prefs/viewrtn".to_owned(), view_rtn),
        param_toggle(
            "Scene Adv",
            "/-prefs/scene_advance".to_owned(),
            scene_advance
        ),
        param_slider_labeled(
            "HA Flags",
            "/-prefs/haflags".to_owned(),
            ha_flags as f32 / 255.0,
            |v| format!("0x{:02X}", (v * 255.0) as u8)
        ),
        param_slider_labeled(
            "Show Ctrl",
            "/-prefs/show_control".to_owned(),
            show_control as f32 / 10.0,
            |v| format!("{:.0}", v * 10.0)
        ),
        param_slider_labeled(
            "Rec Ctrl",
            "/-prefs/rec_control".to_owned(),
            rec_control as f32 / 10.0,
            |v| format!("{:.0}", v * 10.0)
        ),
    ]
    .spacing(4);
    panels = panels.push(detail_panel(
        "Prefs",
        scrollable(prefs_col).height(Length::Fixed(220.0)),
    ));

    // User Assign
    let user_assign_col = ["A", "B", "C"]
        .iter()
        .fold(column!().spacing(6), |col, layer| {
            let color_path = format!("/config/userctrl/{layer}/color");
            let color_val = match app.parameter_values.get(&color_path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let layer_color = match color_val {
                1 => Color::from_rgb8(0xD0, 0x40, 0x40),
                2 => Color::from_rgb8(0x40, 0xD0, 0x40),
                3 => Color::from_rgb8(0xD0, 0xD0, 0x40),
                4 => Color::from_rgb8(0x40, 0x40, 0xD0),
                5 => Color::from_rgb8(0xD0, 0x40, 0xD0),
                6 => Color::from_rgb8(0x40, 0xD0, 0xD0),
                7 => Color::from_rgb8(0xD0, 0xD0, 0xD0),
                _ => Color::from_rgb8(0x80, 0x80, 0x80),
            };
            let enc_row = (1..=4).fold(row!().spacing(2), |r, n| {
                let path = format!("/config/userctrl/{layer}/enc/{n}");
                let val = match app.parameter_values.get(&path) {
                    Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                    _ => "—".to_owned(),
                };
                r.push(
                    column![
                        text(format!("E{n}")).size(8).color(layer_color),
                        text(val).size(8).color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                    ]
                    .spacing(1)
                    .width(Length::Fixed(32.0)),
                )
            });
            let btn_row = (5..=12).fold(row!().spacing(2), |r, n| {
                let path = format!("/config/userctrl/{layer}/btn/{n}");
                let val = match app.parameter_values.get(&path) {
                    Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                    _ => "—".to_owned(),
                };
                r.push(
                    column![
                        text(format!("B{n}")).size(8).color(layer_color),
                        text(val).size(8).color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                    ]
                    .spacing(1)
                    .width(Length::Fixed(32.0)),
                )
            });
            col.push(
                column![
                    text(format!("Layer {layer}")).size(10).color(layer_color),
                    enc_row,
                    btn_row,
                ]
                .spacing(2),
            )
        });
    panels = panels.push(detail_panel(
        "User Assign",
        scrollable(user_assign_col).height(Length::Fixed(180.0)),
    ));

    top_panel_shell(panels)
}

fn routing_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let mut panels = row!().spacing(8);

    // Channel linking
    let chlink_col: Element<'_, Message> = (1..=16)
        .fold(column!().spacing(3), |col, n| {
            let path = format!("/config/chlink/{n:02}");
            let val = match app.parameter_values.get(&path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let active = val != 0;
            let label = format!("Ch {:02}/{:02}", n * 2 - 1, n * 2);
            col.push(
                row![
                    text(label)
                        .size(10)
                        .width(Length::Fixed(60.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text(if active { "ON" } else { "OFF" }).size(10))
                        .on_press(Message::ParameterChanged(
                            path.clone(),
                            OscValue::Int(if active { 0 } else { 1 })
                        ))
                        .padding([2, 6])
                        .style(
                            move |_theme: &Theme, _status: button::Status| button::Style {
                                background: if active {
                                    Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                                } else {
                                    Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                                },
                                text_color: if active {
                                    Color::WHITE
                                } else {
                                    Color::from_rgb8(0x8E, 0x94, 0x9D)
                                },
                                border: Border {
                                    color: if active {
                                        Color::from_rgb8(0x5A, 0x8A, 0x5A)
                                    } else {
                                        Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                    },
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }
                        ),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel("Ch Link", chlink_col));

    // Bus / FX / Mtx / Aux linking
    let mut link_col = column!().spacing(3);
    for n in 1..=4 {
        let path = format!("/config/auxlink/{n:02}");
        let val = match app.parameter_values.get(&path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let active = val != 0;
        let label = format!("Aux {:02}/{:02}", n * 2 - 1, n * 2);
        link_col = link_col.push(
            row![
                text(label)
                    .size(10)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                button(text(if active { "ON" } else { "OFF" }).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(if active { 0 } else { 1 })
                    ))
                    .padding([2, 6])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if active {
                                Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: if active {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x8E, 0x94, 0x9D)
                            },
                            border: Border {
                                color: if active {
                                    Color::from_rgb8(0x5A, 0x8A, 0x5A)
                                } else {
                                    Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                },
                                width: 1.0,
                                radius: 2.0.into()
                            },
                            ..Default::default()
                        }
                    ),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        );
    }
    for n in 1..=8 {
        let path = format!("/config/buslink/{n:02}");
        let val = match app.parameter_values.get(&path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let active = val != 0;
        let label = format!("Bus {:02}/{:02}", n * 2 - 1, n * 2);
        link_col = link_col.push(
            row![
                text(label)
                    .size(10)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                button(text(if active { "ON" } else { "OFF" }).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(if active { 0 } else { 1 })
                    ))
                    .padding([2, 6])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if active {
                                Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: if active {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x8E, 0x94, 0x9D)
                            },
                            border: Border {
                                color: if active {
                                    Color::from_rgb8(0x5A, 0x8A, 0x5A)
                                } else {
                                    Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                },
                                width: 1.0,
                                radius: 2.0.into()
                            },
                            ..Default::default()
                        }
                    ),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        );
    }
    for n in 1..=4 {
        let path = format!("/config/fxlink/{n:02}");
        let val = match app.parameter_values.get(&path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let active = val != 0;
        let label = format!("FX {:02}/{:02}", n * 2 - 1, n * 2);
        link_col = link_col.push(
            row![
                text(label)
                    .size(10)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                button(text(if active { "ON" } else { "OFF" }).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(if active { 0 } else { 1 })
                    ))
                    .padding([2, 6])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if active {
                                Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: if active {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x8E, 0x94, 0x9D)
                            },
                            border: Border {
                                color: if active {
                                    Color::from_rgb8(0x5A, 0x8A, 0x5A)
                                } else {
                                    Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                },
                                width: 1.0,
                                radius: 2.0.into()
                            },
                            ..Default::default()
                        }
                    ),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        );
    }
    for n in 1..=3 {
        let path = format!("/config/mtxlink/{n:02}");
        let val = match app.parameter_values.get(&path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let active = val != 0;
        let label = format!("Mtx {:02}/{:02}", n * 2 - 1, n * 2);
        link_col = link_col.push(
            row![
                text(label)
                    .size(10)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                button(text(if active { "ON" } else { "OFF" }).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(if active { 0 } else { 1 })
                    ))
                    .padding([2, 6])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if active {
                                Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: if active {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x8E, 0x94, 0x9D)
                            },
                            border: Border {
                                color: if active {
                                    Color::from_rgb8(0x5A, 0x8A, 0x5A)
                                } else {
                                    Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                },
                                width: 1.0,
                                radius: 2.0.into()
                            },
                            ..Default::default()
                        }
                    ),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        );
    }
    panels = panels.push(detail_panel("Bus/FX/Mtx", link_col));

    // Link config
    let linkcfg_hadly = param_bool(app, "/config/linkcfg/hadly");
    let linkcfg_eq = param_bool(app, "/config/linkcfg/eq");
    let linkcfg_dyn = param_bool(app, "/config/linkcfg/dyn");
    let linkcfg_fdrmute = param_bool(app, "/config/linkcfg/fdrmute");

    let linkcfg_col = column![
        param_toggle("HA+Dly", "/config/linkcfg/hadly".to_owned(), linkcfg_hadly),
        param_toggle("EQ", "/config/linkcfg/eq".to_owned(), linkcfg_eq),
        param_toggle("Dyn", "/config/linkcfg/dyn".to_owned(), linkcfg_dyn),
        param_toggle(
            "Fdr+Mute",
            "/config/linkcfg/fdrmute".to_owned(),
            linkcfg_fdrmute
        ),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Link Cfg", linkcfg_col));

    // Channel source routing
    let src_col: Element<'_, Message> = (1..=32)
        .fold(column!().spacing(3), |col, ch| {
            let path = format!("/ch/{ch:02}/config/source");
            let val = match app.parameter_values.get(&path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            col.push(
                row![
                    text(format!("Ch {ch:02}"))
                        .size(10)
                        .width(Length::Fixed(40.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    param_slider_labeled("Src", path, val as f32 / 64.0, |v| format!(
                        "{:.0}",
                        v * 64.0
                    )),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel(
        "Ch Source",
        scrollable(src_col).height(Length::Fixed(200.0)),
    ));

    // Output Routing - individual source selectors
    let out_routing_col: Element<'_, Message> = [
        ("/config/routing/OUT/1-4", 1u8),
        ("/config/routing/OUT/5-8", 5u8),
        ("/config/routing/OUT/9-12", 9u8),
        ("/config/routing/OUT/13-16", 13u8),
    ]
    .into_iter()
    .fold(column!().spacing(2), |col, (path, offset)| {
        let val = match app.parameter_values.get(path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let mut group_col = column!().spacing(1);
        for i in 0..4 {
            let out_num = offset + i;
            let src = (val >> (i * 8)) & 0xFF;
            let path = path.to_owned();
            let prev = (src - 1).max(0);
            let next = (src + 1).min(255);
            group_col = group_col.push(
                row![
                    text(format!("Out {out_num:02}"))
                        .size(9)
                        .width(Length::Fixed(40.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text("-").size(9))
                        .on_press(output_routing_message(path.clone(), i, prev, app))
                        .padding([1, 4]),
                    text(format!("{:02}", src))
                        .size(9)
                        .width(Length::Fixed(20.0))
                        .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                    button(text("+").size(9))
                        .on_press(output_routing_message(path.clone(), i, next, app))
                        .padding([1, 4]),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center),
            );
        }
        col.push(group_col)
    })
    .into();
    panels = panels.push(detail_panel(
        "Out Route",
        scrollable(out_routing_col).height(Length::Fixed(200.0)),
    ));

    // Output Delay
    let out_delay_col: Element<'_, Message> = (1..=16)
        .fold(column!().spacing(2), |col, out| {
            let on_path = format!("/outputs/main/{out:02}/delay/on");
            let time_path = format!("/outputs/main/{out:02}/delay/time");
            let on = param_bool(app, &on_path);
            let time = param_float(app, &time_path);
            col.push(
                row![
                    text(format!("Out {out:02}"))
                        .size(9)
                        .width(Length::Fixed(36.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text(if on { "ON" } else { "OFF" }).size(9))
                        .on_press(Message::ParameterChanged(on_path, OscValue::Bool(!on)))
                        .padding([1, 4])
                        .style(
                            move |_theme: &Theme, _status: button::Status| button::Style {
                                background: if on {
                                    Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                                } else {
                                    Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                                },
                                text_color: if on {
                                    Color::WHITE
                                } else {
                                    Color::from_rgb8(0x8E, 0x94, 0x9D)
                                },
                                border: Border {
                                    color: if on {
                                        Color::from_rgb8(0x5A, 0x8A, 0x5A)
                                    } else {
                                        Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                    },
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }
                        ),
                    horizontal_slider(0.0..=1.0, time, move |v| {
                        Message::ParameterChanged(time_path.clone(), OscValue::Float(v))
                    })
                    .fill_from_start()
                    .step(0.01)
                    .width(Length::Fixed(50.0))
                    .height(Length::Fixed(10.0)),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel(
        "Out Delay",
        scrollable(out_delay_col).height(Length::Fixed(200.0)),
    ));

    // Output Source Patching
    let output_src_name = |src: i32| -> &'static str {
        match src {
            0 => "OFF",
            1 => "Main L",
            2 => "Main R",
            3 => "M/C",
            4..=19 => "MixBus",
            20..=25 => "Matrix",
            26..=57 => "DirOut",
            58..=65 => "DirAux",
            66..=73 => "DirFX",
            74 => "Mon L",
            75 => "Mon R",
            76 => "TB",
            _ => "?",
        }
    };

    let out_src_col: Element<'_, Message> = (1..=16)
        .fold(column!().spacing(2), |col, out| {
            let path = format!("/outputs/main/{out:02}/src");
            let val = match app.parameter_values.get(&path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let prev = (val - 1).max(0);
            let next = (val + 1).min(76);
            col.push(
                row![
                    text(format!("Out {out:02}"))
                        .size(9)
                        .width(Length::Fixed(40.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text("-").size(9))
                        .on_press(Message::ParameterChanged(path.clone(), OscValue::Int(prev)))
                        .padding([1, 4]),
                    text(output_src_name(val))
                        .size(9)
                        .width(Length::Fixed(44.0))
                        .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                    button(text("+").size(9))
                        .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
                        .padding([1, 4]),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel(
        "Out Src",
        scrollable(out_src_col).height(Length::Fixed(200.0)),
    ));

    let aux_src_col: Element<'_, Message> = (1..=6)
        .fold(column!().spacing(2), |col, out| {
            let path = format!("/outputs/aux/{out:02}/src");
            let val = match app.parameter_values.get(&path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let prev = (val - 1).max(0);
            let next = (val + 1).min(76);
            col.push(
                row![
                    text(format!("Aux {out:02}"))
                        .size(9)
                        .width(Length::Fixed(40.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text("-").size(9))
                        .on_press(Message::ParameterChanged(path.clone(), OscValue::Int(prev)))
                        .padding([1, 4]),
                    text(output_src_name(val))
                        .size(9)
                        .width(Length::Fixed(44.0))
                        .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                    button(text("+").size(9))
                        .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
                        .padding([1, 4]),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel(
        "Aux Src",
        scrollable(aux_src_col).height(Length::Fixed(120.0)),
    ));

    top_panel_shell(panels)
}

fn rta_source_name(source: i32) -> String {
    match source {
        0 => "None".to_owned(),
        1 => "Monitor".to_owned(),
        2..=33 => format!("Ch {:02}", source - 1),
        34..=41 => format!("Aux {:02}", source - 33),
        42..=49 => format!("FX {}", source - 41),
        50..=65 => format!("Bus {:02}", source - 49),
        66..=71 => format!("Mtx {}", source - 65),
        72 => "Main".to_owned(),
        73 => "Mono".to_owned(),
        _ => format!("Src {source}"),
    }
}

fn rta_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let max_db = 0.0f32;
    let min_db = -90.0f32;
    let range = max_db - min_db;

    let source = param_int(app, "/-prefs/rta/source");
    let source_label = rta_source_name(source);
    let prev_source = (source - 1).max(0);
    let next_source = (source + 1).min(73);

    let bars: Element<'_, Message> = app
        .rta_meters_db
        .iter()
        .enumerate()
        .fold(row!().spacing(1), |row, (_i, &db)| {
            let norm = ((db - min_db) / range).clamp(0.0, 1.0);
            let height = 180.0 * norm;
            let color = if norm > 0.75 {
                Color::from_rgb8(0xD0, 0x40, 0x40)
            } else if norm > 0.5 {
                Color::from_rgb8(0xD0, 0xA0, 0x30)
            } else {
                Color::from_rgb8(0x30, 0xA0, 0x50)
            };
            row.push(
                container(
                    Space::new()
                        .width(Length::Fixed(4.0))
                        .height(Length::Fixed(height)),
                )
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(color)),
                    ..Default::default()
                }),
            )
        })
        .into();

    let rta_gain = param_int(app, "/-prefs/rta/gain");
    let rta_autogain = param_bool(app, "/-prefs/rta/autogain");
    let rta_decay = param_float(app, "/-prefs/rta/decay");
    let rta_mode = param_int(app, "/-prefs/rta/mode");

    let controls = row![
        row![
            text("Source:")
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            button(text("-").size(11))
                .on_press(Message::ParameterChanged(
                    "/-prefs/rta/source".to_owned(),
                    OscValue::Int(prev_source),
                ))
                .padding([2, 6]),
            text(source_label)
                .size(11)
                .width(Length::Fixed(80.0))
                .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
            button(text("+").size(11))
                .on_press(Message::ParameterChanged(
                    "/-prefs/rta/source".to_owned(),
                    OscValue::Int(next_source),
                ))
                .padding([2, 6]),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
        Space::new().width(Length::Fixed(16.0)),
        row![
            text("Gain:")
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            button(text("-").size(11))
                .on_press(Message::ParameterChanged(
                    "/-prefs/rta/gain".to_owned(),
                    OscValue::Int((rta_gain - 6).max(0)),
                ))
                .padding([2, 6]),
            text(format!("{} dB", rta_gain))
                .size(11)
                .width(Length::Fixed(40.0))
                .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
            button(text("+").size(11))
                .on_press(Message::ParameterChanged(
                    "/-prefs/rta/gain".to_owned(),
                    OscValue::Int((rta_gain + 6).min(60)),
                ))
                .padding([2, 6]),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
        Space::new().width(Length::Fixed(12.0)),
        param_toggle("Auto", "/-prefs/rta/autogain".to_owned(), rta_autogain),
        Space::new().width(Length::Fixed(12.0)),
        row![
            text("Decay:")
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            horizontal_slider(0.0..=1.0, rta_decay, move |v| {
                Message::ParameterChanged("/-prefs/rta/decay".to_owned(), OscValue::Float(v))
            })
            .fill_from_start()
            .step(0.01)
            .width(Length::Fixed(60.0))
            .height(Length::Fixed(12.0)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
        Space::new().width(Length::Fixed(12.0)),
        cycle_button(
            "Mode",
            "/-prefs/rta/mode".to_owned(),
            rta_mode,
            &["Bar", "Spec"]
        ),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    let content = column![
        controls,
        row![
            text("20 Hz")
                .size(9)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
            Space::new().width(Length::Fill).height(Length::Fixed(1.0)),
            text("18.7 kHz")
                .size(9)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
        container(bars)
            .height(Length::Fixed(200.0))
            .align_y(iced::Alignment::End)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x1A, 0x1A, 0x1C))),
                ..Default::default()
            }),
        row![
            text("-90 dB")
                .size(9)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
            Space::new().width(Length::Fill).height(Length::Fixed(1.0)),
            text("0 dB")
                .size(9)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(6)
    .align_x(iced::Alignment::Center);

    top_panel_shell(content)
}

fn detail_panel<'a>(
    title: &'static str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![
            text(title)
                .size(14)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            content.into(),
        ]
        .spacing(10)
        .align_x(iced::Alignment::Center),
    )
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x1A, 0x1A, 0x1C))),
        border: Border {
            color: Color::from_rgb8(0x4B, 0x4B, 0x4B),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .padding([10, 10])
    .height(Length::Fixed(220.0))
    .width(Length::Fixed(160.0))
    .into()
}

fn top_panel_shell<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content.into())
        .padding([0, 0])
        .height(Length::Shrink)
        .width(Length::Fill)
        .into()
}

fn channel_send_row<'a>(
    strip_index: usize,
    bus_index: usize,
    bus: u8,
    send_value: f32,
) -> Element<'a, Message> {
    row![
        text(format!("{bus:02}"))
            .size(13)
            .width(Length::Fixed(22.0))
            .color(Color::from_rgb8(0x29, 0xE6, 0xF2)),
        horizontal_slider(0.0..=1.0, send_value, move |next| {
            Message::SendChanged(strip_index, bus_index, next)
        })
        .fill_from_start()
        .step(0.01)
        .double_click_reset(0.0)
        .width(Length::Fixed(110.0))
        .height(Length::Fixed(10.0)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn x32_color_to_rgb(value: u8) -> Color {
    match value {
        1 => Color::from_rgb8(0xFF, 0x45, 0x45),
        2 => Color::from_rgb8(0x32, 0xCD, 0x32),
        3 => Color::from_rgb8(0xFF, 0xD7, 0x00),
        4 => Color::from_rgb8(0x41, 0x69, 0xE1),
        5 => Color::from_rgb8(0xFF, 0x00, 0xFF),
        6 => Color::from_rgb8(0x00, 0xFF, 0xFF),
        7 => Color::from_rgb8(0xFF, 0xFF, 0xFF),
        9 => Color::from_rgb8(0xCC, 0x33, 0x33),
        10 => Color::from_rgb8(0x28, 0xA4, 0x28),
        11 => Color::from_rgb8(0xCC, 0xAC, 0x00),
        12 => Color::from_rgb8(0x33, 0x55, 0xB4),
        13 => Color::from_rgb8(0xCC, 0x00, 0xCC),
        14 => Color::from_rgb8(0x00, 0xCC, 0xCC),
        15 => Color::from_rgb8(0xDD, 0xDD, 0xDD),
        _ => Color::from_rgb8(0x3B, 0x42, 0x52),
    }
}

fn strip_label(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("CH {channel:02}"),
        FaderTarget::Aux(aux) => format!("AUX {aux:02}"),
        FaderTarget::Bus(bus) => format!("BUS {bus:02}"),
        FaderTarget::FxRtn(fx) => format!("FX {fx:02}"),
        FaderTarget::Mtx(mtx) => format!("MTX {mtx:02}"),
        FaderTarget::Dca(dca) => format!("DCA {dca}"),
        FaderTarget::Main => "LR".to_owned(),
    }
}

fn strip_name(app: &StatusApp, index: usize, target: FaderTarget) -> String {
    app.names[index]
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| strip_label(target))
}

fn format_fader_label(value: f32) -> String {
    if value <= 0.0 {
        return "-oo".to_owned();
    }

    format!("{:.1} dB", x32_fader_db(value))
}

fn format_pan_label(value: f32) -> String {
    let offset = ((value.clamp(0.0, 1.0) - 0.5) * 200.0).round() as i32;

    if offset == 0 {
        "C".to_owned()
    } else if offset < 0 {
        format!("L{}", -offset)
    } else {
        format!("R{offset}")
    }
}

fn gain_range(source: GainSource) -> std::ops::RangeInclusive<f32> {
    match source {
        GainSource::Headamp(_) => -12.0..=60.0,
        GainSource::Trim => -18.0..=18.0,
    }
}

fn gain_step(source: GainSource) -> f32 {
    match source {
        GainSource::Headamp(_) => 0.1,
        GainSource::Trim => 0.25,
    }
}

fn quantize_gain_value(value: f32, source: GainSource) -> f32 {
    let range = gain_range(source);
    let min = *range.start();
    let max = *range.end();
    let step = gain_step(source);
    let steps = ((value.clamp(min, max) - min) / step).round();
    (min + steps * step).clamp(min, max)
}

fn format_gain_label(value: f32, source: GainSource) -> String {
    match source {
        GainSource::Headamp(_) => format!("{value:+.1} dB"),
        GainSource::Trim => format!("T {value:+.1} dB"),
    }
}

fn linf_value(raw: f32, min: f32, max: f32) -> f32 {
    raw.clamp(0.0, 1.0) * (max - min) + min
}

fn logf_value(raw: f32, min: f32, max: f32) -> f32 {
    let raw = raw.clamp(0.0, 1.0);
    if min <= 0.0 || max <= 0.0 || min == max {
        return raw * (max - min) + min;
    }
    min * (max / min).powf(raw)
}

fn format_hz(value: f32) -> String {
    if value >= 1000.0 {
        format!("{:.2} kHz", value / 1000.0)
    } else {
        format!("{:.1} Hz", value)
    }
}

fn format_db(value: f32) -> String {
    format!("{value:+.2} dB")
}

fn format_db1(value: f32) -> String {
    format!("{value:+.1} dB")
}

fn format_ms(value: f32) -> String {
    if value < 1.0 {
        format!("{value:.2} ms")
    } else if value < 100.0 {
        format!("{value:.1} ms")
    } else {
        format!("{value:.0} ms")
    }
}

fn format_q(value: f32) -> String {
    format!("{value:.2}")
}

fn format_pct(value: f32) -> String {
    format!("{:.0}%", value * 100.0)
}

fn key_source_name(value: i32) -> String {
    match value {
        0 => "Self".to_owned(),
        1..=32 => format!("Ch {value:02}"),
        33..=40 => format!("Aux {:02}", value - 32),
        41 => "USB L".to_owned(),
        42 => "USB R".to_owned(),
        43..=50 => {
            let names = [
                "Fx1L", "Fx1R", "Fx2L", "Fx2R", "Fx3L", "Fx3R", "Fx4L", "Fx4R",
            ];
            names
                .get((value - 43) as usize)
                .copied()
                .unwrap_or("?")
                .to_owned()
        }
        51..=66 => format!("Bus {:02}", value - 50),
        _ => format!("Src {value}"),
    }
}

fn x32_fader_db(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);

    if value >= 0.5 {
        value * 40.0 - 30.0
    } else if value >= 0.25 {
        value * 80.0 - 50.0
    } else if value >= 0.0625 {
        value * 160.0 - 70.0
    } else {
        value * 480.0 - 90.0
    }
}

fn linear_meter_to_db(value: f32) -> f32 {
    let value = value.max(0.000_031_622_78);
    (20.0 * value.log10()).clamp(-90.0, 20.0)
}

fn strip_base_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(n) => format!("/ch/{n:02}"),
        FaderTarget::Aux(n) => format!("/auxin/{n:02}"),
        FaderTarget::Bus(n) => format!("/bus/{n:02}"),
        FaderTarget::FxRtn(n) => format!("/fxrtn/{n:02}"),
        FaderTarget::Mtx(n) => format!("/mtx/{n:02}"),
        FaderTarget::Dca(n) => format!("/dca/{n}"),
        FaderTarget::Main => "/main/st".to_owned(),
    }
}

fn param_float(app: &StatusApp, path: &str) -> f32 {
    match app.parameter_values.get(path) {
        Some(OscValue::Float(v)) => *v,
        Some(OscValue::Int(v)) => *v as f32,
        _ => 0.0,
    }
}

fn param_bool(app: &StatusApp, path: &str) -> bool {
    match app.parameter_values.get(path) {
        Some(OscValue::Bool(v)) => *v,
        Some(OscValue::Int(v)) => *v != 0,
        Some(OscValue::Float(v)) => *v != 0.0,
        _ => false,
    }
}

fn param_int(app: &StatusApp, path: &str) -> i32 {
    match app.parameter_values.get(path) {
        Some(OscValue::Int(v)) => *v,
        Some(OscValue::Float(v)) => *v as i32,
        _ => 0,
    }
}

fn param_slider_labeled<'a>(
    label: &'a str,
    path: String,
    value: f32,
    format_fn: fn(f32) -> String,
) -> Element<'a, Message> {
    column![
        row![
            text(label)
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            text(format_fn(value))
                .size(11)
                .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
        horizontal_slider(0.0..=1.0, value, move |next| {
            Message::ParameterChanged(path.clone(), OscValue::Float(next))
        })
        .fill_from_start()
        .step(0.01)
        .width(Length::Fill)
        .height(Length::Fixed(16.0)),
    ]
    .spacing(2)
    .into()
}

fn param_toggle<'a>(label: &'a str, path: String, active: bool) -> Element<'a, Message> {
    let color = if active {
        Color::from_rgb8(0x7D, 0xD3, 0xA7)
    } else {
        Color::from_rgb8(0xF0, 0x7C, 0x82)
    };
    button(text(label).size(12))
        .on_press(Message::ParameterChanged(
            path.clone(),
            OscValue::Bool(!active),
        ))
        .style(move |_theme: &Theme, _status: button::Status| toggle_button_style(active, color))
        .into()
}

fn toggle_button_style(active: bool, color: Color) -> button::Style {
    if active {
        button::Style {
            background: Some(Background::Color(color)),
            text_color: Color::from_rgb8(0x14, 0x18, 0x20),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color,
            },
            ..Default::default()
        }
    } else {
        button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: color,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color,
            },
            ..Default::default()
        }
    }
}

fn meter_subscription(mixer_addr: SocketAddr) -> Subscription<Message> {
    Subscription::run_with(mixer_addr, meter_worker).map(Message::MetersLoaded)
}

fn master_meter_subscription(mixer_addr: SocketAddr) -> Subscription<Message> {
    Subscription::run_with(mixer_addr, master_meter_worker)
        .map(|r| Message::MasterMetersLoaded(Box::new(r)))
}

fn rta_meter_subscription(mixer_addr: SocketAddr) -> Subscription<Message> {
    Subscription::run_with(mixer_addr, rta_meter_worker)
        .map(|r| Message::RtaMetersLoaded(Box::new(r)))
}

fn state_worker(mixer_addr: &SocketAddr) -> BoxStream<'static, Result<ConsoleUpdate, String>> {
    let mixer_addr = *mixer_addr;
    stream::channel(
        64,
        move |mut output: mpsc::Sender<Result<ConsoleUpdate, String>>| async move {
            let socket = match bind_meter_socket().await {
                Ok(socket) => socket,
                Err(error) => {
                    let _ = output.send(Err(error.to_string())).await;
                    return;
                }
            };

            if let Err(error) = socket.send_to(XREMOTE_REQUEST, mixer_addr).await {
                let _ = output
                    .send(Err(format!("failed to send /xremote: {error}")))
                    .await;
                return;
            }

            let mut last_xremote = Instant::now();
            let mut buffer = [0_u8; 4096];

            loop {
                if last_xremote.elapsed() >= Duration::from_secs(5) {
                    if let Err(error) = socket.send_to(XREMOTE_REQUEST, mixer_addr).await {
                        let _ = output
                            .send(Err(format!("failed to renew /xremote: {error}")))
                            .await;
                        return;
                    }
                    last_xremote = Instant::now();
                }

                match tokio::time::timeout(
                    Duration::from_millis(250),
                    socket.recv_from(&mut buffer),
                )
                .await
                {
                    Ok(Ok((received, _))) => {
                        if let Some(update) = parse_console_update(&buffer[..received]) {
                            let _ = output.send(Ok(update)).await;
                        }
                    }
                    Ok(Err(error)) => {
                        let _ = output
                            .send(Err(format!("failed while receiving state stream: {error}")))
                            .await;
                        return;
                    }
                    Err(_) => {}
                }
            }
        },
    )
    .boxed()
}

fn meter_worker(mixer_addr: &SocketAddr) -> BoxStream<'static, Result<Vec<StripMeter>, String>> {
    let mixer_addr = *mixer_addr;
    stream::channel(
        32,
        move |mut output: mpsc::Sender<Result<Vec<StripMeter>, String>>| async move {
            let socket = match bind_meter_socket().await {
                Ok(socket) => socket,
                Err(error) => {
                    let _ = output.send(Err(error.to_string())).await;
                    return;
                }
            };

            let subscribe = batchsubscribe_meter_request("meters/0", "/meters/0", 0, 0, 1);
            if let Err(error) = socket.send_to(XREMOTE_REQUEST, mixer_addr).await {
                let _ = output
                    .send(Err(format!("failed to send /xremote: {error}")))
                    .await;
                return;
            }
            if let Err(error) = socket.send_to(&subscribe, mixer_addr).await {
                let _ = output
                    .send(Err(format!(
                        "failed to send /batchsubscribe for meters/0: {error}"
                    )))
                    .await;
                return;
            }

            let renew = renew_request("meters/0");
            let mut last_xremote = Instant::now();
            let mut last_renew = Instant::now();
            let mut buffer = [0_u8; 4096];

            loop {
                if last_xremote.elapsed() >= Duration::from_secs(5) {
                    if let Err(error) = socket.send_to(XREMOTE_REQUEST, mixer_addr).await {
                        let _ = output
                            .send(Err(format!("failed to renew /xremote: {error}")))
                            .await;
                        return;
                    }
                    last_xremote = Instant::now();
                }

                if last_renew.elapsed() >= Duration::from_secs(5) {
                    if let Err(error) = socket.send_to(&renew, mixer_addr).await {
                        let _ = output
                            .send(Err(format!("failed to renew meter subscription: {error}")))
                            .await;
                        return;
                    }
                    last_renew = Instant::now();
                }

                match tokio::time::timeout(
                    Duration::from_millis(250),
                    socket.recv_from(&mut buffer),
                )
                .await
                {
                    Ok(Ok((received, _))) => {
                        if let Ok(meters) = parse_input_meter_packet(&buffer[..received]) {
                            let _ = output.send(Ok(meters)).await;
                        }
                    }
                    Ok(Err(error)) => {
                        let _ = output
                            .send(Err(format!("failed while receiving meter stream: {error}")))
                            .await;
                        return;
                    }
                    Err(_) => {}
                }

                sleep(Duration::from_millis(10)).await;
            }
        },
    )
    .boxed()
}

fn master_meter_worker(
    mixer_addr: &SocketAddr,
) -> BoxStream<'static, Result<MainMeterLevels, String>> {
    let mixer_addr = *mixer_addr;
    stream::channel(
        32,
        move |mut output: mpsc::Sender<Result<MainMeterLevels, String>>| async move {
            let socket = match bind_meter_socket().await {
                Ok(socket) => socket,
                Err(error) => {
                    let _ = output.send(Err(error.to_string())).await;
                    return;
                }
            };

            let subscribe = batchsubscribe_meter_request("meters/2", "/meters/2", 0, 0, 1);
            if let Err(error) = socket.send_to(XREMOTE_REQUEST, mixer_addr).await {
                let _ = output
                    .send(Err(format!("failed to send /xremote: {error}")))
                    .await;
                return;
            }
            if let Err(error) = socket.send_to(&subscribe, mixer_addr).await {
                let _ = output
                    .send(Err(format!(
                        "failed to send /batchsubscribe for meters/2: {error}"
                    )))
                    .await;
                return;
            }

            let mut last_renew = Instant::now();
            let mut buffer = [0_u8; 4096];

            loop {
                if last_renew.elapsed() >= Duration::from_secs(5) {
                    let renew = renew_request("meters/2");
                    if let Err(error) = socket.send_to(&renew, mixer_addr).await {
                        let _ = output
                            .send(Err(format!(
                                "failed to renew meter stream meters/2: {error}"
                            )))
                            .await;
                        return;
                    }
                    last_renew = Instant::now();
                }

                match tokio::time::timeout(
                    Duration::from_millis(250),
                    socket.recv_from(&mut buffer),
                )
                .await
                {
                    Ok(Ok((received, _))) => {
                        if let Ok(levels) = parse_main_meter_packet(&buffer[..received]) {
                            let _ = output.send(Ok(levels)).await;
                        }
                    }
                    Ok(Err(error)) => {
                        let _ = output
                            .send(Err(format!(
                                "failed while receiving main meter stream: {error}"
                            )))
                            .await;
                        return;
                    }
                    Err(_) => {}
                }

                sleep(Duration::from_millis(10)).await;
            }
        },
    )
    .boxed()
}

fn rta_meter_worker(mixer_addr: &SocketAddr) -> BoxStream<'static, Result<[f32; 100], String>> {
    let mixer_addr = *mixer_addr;
    stream::channel(
        32,
        move |mut output: mpsc::Sender<Result<[f32; 100], String>>| async move {
            let socket = match bind_meter_socket().await {
                Ok(socket) => socket,
                Err(error) => {
                    let _ = output.send(Err(error.to_string())).await;
                    return;
                }
            };

            let subscribe = batchsubscribe_meter_request("meters/15", "/meters/15", 0, 0, 1);
            if let Err(error) = socket.send_to(XREMOTE_REQUEST, mixer_addr).await {
                let _ = output
                    .send(Err(format!("failed to send /xremote: {error}")))
                    .await;
                return;
            }
            if let Err(error) = socket.send_to(&subscribe, mixer_addr).await {
                let _ = output
                    .send(Err(format!(
                        "failed to send /batchsubscribe for meters/15: {error}"
                    )))
                    .await;
                return;
            }

            let mut last_renew = Instant::now();
            let mut buffer = [0_u8; 4096];

            loop {
                if last_renew.elapsed() >= Duration::from_secs(5) {
                    let renew = renew_request("meters/15");
                    if let Err(error) = socket.send_to(&renew, mixer_addr).await {
                        let _ = output
                            .send(Err(format!(
                                "failed to renew meter stream meters/15: {error}"
                            )))
                            .await;
                        return;
                    }
                    last_renew = Instant::now();
                }

                match tokio::time::timeout(
                    Duration::from_millis(250),
                    socket.recv_from(&mut buffer),
                )
                .await
                {
                    Ok(Ok((received, _))) => {
                        if let Ok(levels) = parse_rta_meter_packet(&buffer[..received]) {
                            let _ = output.send(Ok(levels)).await;
                        }
                    }
                    Ok(Err(error)) => {
                        let _ = output
                            .send(Err(format!(
                                "failed while receiving rta meter stream: {error}"
                            )))
                            .await;
                        return;
                    }
                    Err(_) => {}
                }

                sleep(Duration::from_millis(10)).await;
            }
        },
    )
    .boxed()
}

async fn bind_meter_socket() -> std::io::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0))).await?;
    Ok(socket)
}
