pub mod app;
mod codec;
mod message;
mod model;
mod net;
mod osc;
mod panel_paths;
pub mod parameters;
mod state;
mod ui;
mod workers;

pub use codec::{
    parse_console_update, parse_input_meter_packet, parse_main_meter_packet, parse_rta_meter_packet,
};
pub use model::MixerConfig;
pub use model::{
    ConsoleUpdate, DiscoveredMixer, FaderTarget, GainSource, MainMeterLevels, MixerModel,
    StripColor, StripFader, StripGain, StripMeter, StripMute, StripName, StripPan, StripSend,
    StripSolo,
};
pub use net::{
    ColorBankProbe, ConnectionProbe, DiscoveryProbe, FaderBankProbe, GainBankProbe, MeterBankProbe,
    MuteBankProbe, NameBankProbe, PanBankProbe, ParameterProbe, ParseTargetError, ProbeError,
    ProbeOutcome, ProbeResponse, SendBankProbe, SoloBankProbe, X32_BROADCAST_ADDR,
    X32_DEFAULT_PORT, XR18_BROADCAST_ADDR, XR18_DEFAULT_PORT, XREMOTE_REQUEST, XREMOTENFB_REQUEST,
    parse_target,
};
pub use osc::{batchsubscribe_meter_request, osc_meter_group_request, renew_request};
pub use parameters::{OscValue, build_get, build_set, parse_osc_value, path};
