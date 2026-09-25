use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MixerModel {
    #[default]
    X32,
    XR18,
}

impl MixerModel {
    pub fn from_model_string(s: &str) -> Option<Self> {
        match s {
            "X32" | "X32C" | "X32P" | "X32Rack" => Some(Self::X32),
            "XR18" | "XR16" | "XR12" | "X18" | "X16" | "X12" => Some(Self::XR18),
            _ => None,
        }
    }
}

impl std::fmt::Display for MixerModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X32 => write!(f, "X32"),
            Self::XR18 => write!(f, "XR18"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaderTarget {
    Channel(u8),
    Aux(u8),
    Bus(u8),
    FxRtn(u8),
    Mtx(u8),
    Dca(u8),
    Main,
}

impl std::fmt::Display for FaderTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Channel(channel) => write!(f, "channel {channel:02}"),
            Self::Aux(aux) => write!(f, "aux {aux:02}"),
            Self::Bus(bus) => write!(f, "bus {bus:02}"),
            Self::FxRtn(fx) => write!(f, "fxrtn {fx:02}"),
            Self::Mtx(mtx) => write!(f, "mtx {mtx:02}"),
            Self::Dca(dca) => write!(f, "dca {dca}"),
            Self::Main => write!(f, "main lr"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StripFader {
    pub target: FaderTarget,
    pub value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StripPan {
    pub target: FaderTarget,
    pub value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StripGain {
    pub target: FaderTarget,
    pub value: f32,
    pub source: GainSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GainSource {
    Headamp(u8),
    Trim,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StripSend {
    pub target: FaderTarget,
    pub bus: u8,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripName {
    pub target: FaderTarget,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripColor {
    pub target: FaderTarget,
    pub value: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripMute {
    pub target: FaderTarget,
    pub on: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripSolo {
    pub target: FaderTarget,
    pub on: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StripMeter {
    pub target: FaderTarget,
    pub level_linear: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleUpdate {
    Gain(StripGain),
    HeadampGain {
        index: u8,
        value: f32,
    },
    Fader(StripFader),
    Pan(StripPan),
    Send(StripSend),
    Mute(StripMute),
    Solo(StripSolo),
    Name(StripName),
    Color(StripColor),
    Parameter {
        path: String,
        value: crate::parameters::OscValue,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredMixer {
    pub addr: SocketAddr,
    pub network_address: Option<String>,
    pub name: Option<String>,
    pub model: MixerModel,
    pub firmware: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct MainMeterLevels {
    pub mains: [f32; 16],
    pub main_lr: [f32; 2],
    pub matrices: [f32; 6],
}

pub struct MixerConfig {
    pub strip_count: usize,
    pub send_bus_count: usize,
    pub visible_strips: &'static [FaderTarget],
    pub send_buses: &'static [u8],
    pub matrix_sends: &'static [u8],
    pub dca_count: usize,
    pub mute_group_count: usize,
}

const X32_VISIBLE_STRIPS: &[FaderTarget] = &[
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

const X32_SEND_BUSES: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const X32_MATRIX_SENDS: &[u8] = &[1, 2, 3, 4, 5, 6];

const XR18_VISIBLE_STRIPS: &[FaderTarget] = &[
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
    FaderTarget::Bus(1),
    FaderTarget::Bus(2),
    FaderTarget::Bus(3),
    FaderTarget::Bus(4),
    FaderTarget::Bus(5),
    FaderTarget::Bus(6),
    FaderTarget::FxRtn(1),
    FaderTarget::FxRtn(2),
    FaderTarget::FxRtn(3),
    FaderTarget::FxRtn(4),
    FaderTarget::FxRtn(5),
    FaderTarget::Dca(1),
    FaderTarget::Dca(2),
    FaderTarget::Dca(3),
    FaderTarget::Dca(4),
];

const XR18_SEND_BUSES: &[u8] = &[1, 2, 3, 4, 5, 6];
const XR18_MATRIX_SENDS: &[u8] = &[];

pub(crate) fn config_for_model(model: MixerModel) -> MixerConfig {
    match model {
        MixerModel::X32 => MixerConfig {
            strip_count: 74,
            send_bus_count: 16,
            visible_strips: X32_VISIBLE_STRIPS,
            send_buses: X32_SEND_BUSES,
            matrix_sends: X32_MATRIX_SENDS,
            dca_count: 8,
            mute_group_count: 6,
        },
        MixerModel::XR18 => MixerConfig {
            strip_count: 31,
            send_bus_count: 6,
            visible_strips: XR18_VISIBLE_STRIPS,
            send_buses: XR18_SEND_BUSES,
            matrix_sends: XR18_MATRIX_SENDS,
            dca_count: 4,
            mute_group_count: 4,
        },
    }
}
