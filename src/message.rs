use std::net::SocketAddr;

use crate::model::{
    ConsoleUpdate, DiscoveredMixer, MainMeterLevels, StripColor, StripFader, StripGain, StripMeter,
    StripMute, StripName, StripPan, StripSend, StripSolo,
};
use crate::net::ProbeOutcome;
use crate::net::ProbeResponse;
use crate::parameters::OscValue;

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
    MixerEvent(usize, Box<Message>),
    TabSelected(usize),
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
