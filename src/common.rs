use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

pub const X32_DEFAULT_PORT: u16 = 10023;
pub const X32_BROADCAST_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), X32_DEFAULT_PORT);
pub const XR18_DEFAULT_PORT: u16 = 10024;
pub const XR18_BROADCAST_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), XR18_DEFAULT_PORT);
const INFO_REQUEST: &[u8] = b"/info\0\0\0,\0\0\0";
const STATUS_REQUEST: &[u8] = b"/status\0,\0\0\0";
const XINFO_REQUEST: &[u8] = b"/xinfo\0\0,\0\0\0";
pub const XREMOTE_REQUEST: &[u8] = b"/xremote\0\0\0,\0\0\0";
pub const XREMOTENFB_REQUEST: &[u8] = b"/xremotenfb\0";
const XINFO_RESPONSE: &str = "/xinfo";
const INFO_RESPONSE: &str = "/info";
const STATUS_RESPONSE: &str = "/status";
pub(crate) const FADER_RESPONSE_SUFFIX: &str = "/mix/fader";
pub(crate) const PAN_RESPONSE_SUFFIX: &str = "/mix/pan";
pub(crate) const GAIN_RESPONSE_SUFFIX: &str = "/preamp/trim";
pub(crate) const HEADAMP_GAIN_RESPONSE_SUFFIX: &str = "/gain";
pub(crate) const HEADAMP_INDEX_RESPONSE_SUFFIX: &str = "/index";
pub(crate) const MUTE_RESPONSE_SUFFIX: &str = "/mix/on";
pub(crate) const SOLO_RESPONSE_PREFIX: &str = "/-stat/solosw/";
pub(crate) const NAME_RESPONSE_SUFFIX: &str = "/config/name";
pub(crate) const COLOR_RESPONSE_SUFFIX: &str = "/config/color";
pub(crate) const INPUT_METERS_REQUEST: &str = "/meters/0";
pub(crate) const INPUT_METERS_ALIAS: &str = "/meters/0";
pub(crate) const MAIN_METERS_REQUEST: &str = "/meters/2";
pub(crate) const MAIN_METERS_ALIAS: &str = "/meters/2";

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

#[derive(Debug, Clone)]
pub struct ConnectionProbe {
    target: SocketAddr,
    timeout: Duration,
    bind_addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct DiscoveryProbe {
    bind_addr: SocketAddr,
    broadcast_addr: SocketAddr,
    timeout: Duration,
}

impl Default for DiscoveryProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryProbe {
    pub fn new() -> Self {
        Self {
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            broadcast_addr: X32_BROADCAST_ADDR,
            timeout: Duration::from_millis(1200),
        }
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn with_broadcast_addr(mut self, broadcast_addr: SocketAddr) -> Self {
        self.broadcast_addr = broadcast_addr;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn discover(&self) -> Result<Vec<DiscoveredMixer>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket.set_broadcast(true).map_err(ProbeError::Configure)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let _ = socket.send_to(XINFO_REQUEST, X32_BROADCAST_ADDR);
        let _ = socket.send_to(XINFO_REQUEST, XR18_BROADCAST_ADDR);

        let start = Instant::now();
        let mut mixers: Vec<DiscoveredMixer> = Vec::new();
        let mut buffer = [0_u8; 2048];

        loop {
            match socket.recv_from(&mut buffer) {
                Ok((received, responder)) => {
                    if let Some(mixer) = parse_discovered_mixer(&buffer[..received], responder)
                        && mixers.iter().all(|known| known.addr != mixer.addr)
                    {
                        mixers.push(mixer);
                    }

                    if start.elapsed() >= self.timeout {
                        break;
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(error) => return Err(ProbeError::Receive(error)),
            }
        }

        Ok(mixers)
    }
}

fn fallback_addr(addr: SocketAddr) -> Option<SocketAddr> {
    let fallback_port = match addr.port() {
        X32_DEFAULT_PORT => XR18_DEFAULT_PORT,
        XR18_DEFAULT_PORT => X32_DEFAULT_PORT,
        _ => return None,
    };
    let mut fallback = addr;
    fallback.set_port(fallback_port);
    Some(fallback)
}

impl ConnectionProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            timeout: Duration::from_millis(750),
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn target(&self) -> SocketAddr {
        self.target
    }

    pub fn probe(&self) -> Result<ProbeOutcome, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut buffer = [0_u8; 2048];

        match self.try_target(&socket, self.target, &mut buffer) {
            Ok(outcome @ ProbeOutcome::Connected { .. }) => return Ok(outcome),
            Ok(ProbeOutcome::Disconnected) => {}
            Err(error) => return Err(error),
        }

        let fallback = fallback_addr(self.target);
        if let Some(fallback) = fallback {
            match self.try_target(&socket, fallback, &mut buffer) {
                Ok(outcome @ ProbeOutcome::Connected { .. }) => return Ok(outcome),
                Ok(ProbeOutcome::Disconnected) => {}
                Err(error) => return Err(error),
            }
        }

        Ok(ProbeOutcome::Disconnected)
    }

    fn try_target(
        &self,
        socket: &UdpSocket,
        target: SocketAddr,
        buffer: &mut [u8],
    ) -> Result<ProbeOutcome, ProbeError> {
        socket
            .send_to(INFO_REQUEST, target)
            .map_err(ProbeError::Send)?;

        match socket.recv_from(buffer) {
            Ok((received, responder)) => {
                let response = parse_response(&buffer[..received]);
                let model = if response == ProbeResponse::Info {
                    osc_strings(&buffer[..received])
                        .get(2)
                        .and_then(|s| MixerModel::from_model_string(s))
                } else {
                    None
                };
                Ok(ProbeOutcome::Connected {
                    responder,
                    response,
                    model,
                })
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                socket
                    .send_to(STATUS_REQUEST, target)
                    .map_err(ProbeError::Send)?;
                match socket.recv_from(buffer) {
                    Ok((received, responder)) => Ok(ProbeOutcome::Connected {
                        responder,
                        response: parse_response(&buffer[..received]),
                        model: None,
                    }),
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) =>
                    {
                        Ok(ProbeOutcome::Disconnected)
                    }
                    Err(error) => Err(ProbeError::Receive(error)),
                }
            }
            Err(error) => Err(ProbeError::Receive(error)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FaderBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

impl FaderBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load(&self, targets: &[FaderTarget]) -> Result<Vec<StripFader>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut faders = Vec::with_capacity(targets.len());

        for &target in targets {
            let path = fader_path(target, self.model);
            let request = osc_query(&path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((path, value)) = parse_fader_value(packet) else {
                return Err(ProbeError::Protocol(format!(
                    "unexpected OSC reply while reading {target}"
                )));
            };

            if path != fader_path(target, self.model) {
                return Err(ProbeError::Protocol(format!(
                    "received fader reply for '{path}' while reading {target}"
                )));
            }

            faders.push(StripFader { target, value });
        }

        Ok(faders)
    }

    pub fn set(&self, target: FaderTarget, value: f32) -> Result<(), ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let packet = osc_float_message(&fader_path(target, self.model), value.clamp(0.0, 1.0));
        socket
            .send_to(&packet, self.target)
            .map_err(ProbeError::Send)?;
        Ok(())
    }
}

impl PanBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load(&self, targets: &[FaderTarget]) -> Result<Vec<StripPan>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut pans = Vec::with_capacity(targets.len());

        for &target in targets {
            let path = pan_path(target, self.model);
            let request = osc_query(&path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((reply_path, value)) = parse_pan_value(packet) else {
                return Err(ProbeError::Protocol(format!(
                    "unexpected OSC reply while reading pan for {target}"
                )));
            };

            if reply_path != path {
                return Err(ProbeError::Protocol(format!(
                    "received pan reply for '{reply_path}' while reading {target}"
                )));
            }

            pans.push(StripPan { target, value });
        }

        Ok(pans)
    }

    pub fn set(&self, target: FaderTarget, value: f32) -> Result<(), ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let packet = osc_float_message(&pan_path(target, self.model), value.clamp(0.0, 1.0));
        socket
            .send_to(&packet, self.target)
            .map_err(ProbeError::Send)?;
        Ok(())
    }
}

impl SendBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load(
        &self,
        targets: &[FaderTarget],
        buses: &[u8],
    ) -> Result<Vec<StripSend>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut sends = Vec::with_capacity(targets.len() * buses.len());

        for &target in targets {
            for &bus in buses {
                let path = send_level_path(target, bus, self.model);
                let request = osc_query(&path);
                socket
                    .send_to(&request, self.target)
                    .map_err(ProbeError::Send)?;

                let mut buffer = [0_u8; 2048];
                let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
                let packet = &buffer[..received];
                let Some((reply_path, value)) = parse_send_value(packet) else {
                    return Err(ProbeError::Protocol(format!(
                        "unexpected OSC reply while reading send {bus:02} for {target}"
                    )));
                };

                if reply_path != path {
                    return Err(ProbeError::Protocol(format!(
                        "received send reply for '{reply_path}' while reading bus {bus:02} for {target}"
                    )));
                }

                sends.push(StripSend { target, bus, value });
            }
        }

        Ok(sends)
    }

    pub fn set(&self, target: FaderTarget, bus: u8, value: f32) -> Result<(), ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let packet = osc_float_message(
            &send_level_path(target, bus, self.model),
            value.clamp(0.0, 1.0),
        );
        socket
            .send_to(&packet, self.target)
            .map_err(ProbeError::Send)?;
        Ok(())
    }
}

impl GainBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load(&self, targets: &[FaderTarget]) -> Result<Vec<StripGain>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut gains = Vec::with_capacity(targets.len());

        for &target in targets {
            gains.push(self.read_gain(&socket, target)?);
        }

        Ok(gains)
    }

    pub fn set(
        &self,
        target: FaderTarget,
        source: GainSource,
        value: f32,
    ) -> Result<(), ProbeError> {
        if matches!(
            target,
            FaderTarget::Bus(_) | FaderTarget::FxRtn(_) | FaderTarget::Mtx(_) | FaderTarget::Dca(_)
        ) {
            return Ok(());
        }

        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let packet = match (source, self.model) {
            (GainSource::Headamp(index), MixerModel::X32) => osc_float_message(
                &headamp_gain_path(index, self.model),
                crate::x32::encode_headamp_gain(value),
            ),
            (GainSource::Headamp(index), MixerModel::XR18) => osc_float_message(
                &headamp_gain_path(index, self.model),
                crate::xr18::encode_headamp_gain(value),
            ),
            (GainSource::Trim, _) => {
                osc_float_message(&gain_path(target, self.model), encode_trim_gain(value))
            }
        };
        socket
            .send_to(&packet, self.target)
            .map_err(ProbeError::Send)?;
        Ok(())
    }

    fn read_gain(&self, socket: &UdpSocket, target: FaderTarget) -> Result<StripGain, ProbeError> {
        if matches!(
            target,
            FaderTarget::Bus(_) | FaderTarget::FxRtn(_) | FaderTarget::Mtx(_) | FaderTarget::Dca(_)
        ) {
            return Ok(StripGain {
                target,
                value: 0.0,
                source: GainSource::Trim,
            });
        }

        if gain_uses_headamp(target)
            && let Some(index) = self.read_headamp_index(socket, target)?
        {
            let path = headamp_gain_path(index, self.model);
            let request = osc_query(&path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((reply_path, value)) = parse_headamp_gain_value(packet) else {
                return Err(ProbeError::Protocol(format!(
                    "unexpected OSC reply while reading headamp gain for {target}"
                )));
            };

            if reply_path != path {
                return Err(ProbeError::Protocol(format!(
                    "received headamp gain reply for '{reply_path}' while reading {target}"
                )));
            }

            Ok(StripGain {
                target,
                value: match self.model {
                    MixerModel::X32 => crate::x32::decode_headamp_gain(value),
                    MixerModel::XR18 => crate::xr18::decode_headamp_gain(value),
                },
                source: GainSource::Headamp(index),
            })
        } else {
            let path = gain_path(target, self.model);
            let request = osc_query(&path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((reply_path, value)) = parse_gain_value(packet) else {
                return Err(ProbeError::Protocol(format!(
                    "unexpected OSC reply while reading trim gain for {target}"
                )));
            };

            if reply_path != path {
                return Err(ProbeError::Protocol(format!(
                    "received trim gain reply for '{reply_path}' while reading {target}"
                )));
            }

            Ok(StripGain {
                target,
                value: decode_trim_gain(value),
                source: GainSource::Trim,
            })
        }
    }

    fn read_headamp_index(
        &self,
        socket: &UdpSocket,
        target: FaderTarget,
    ) -> Result<Option<u8>, ProbeError> {
        let path = headamp_index_path(target, self.model);
        let request = osc_query(&path);
        socket
            .send_to(&request, self.target)
            .map_err(ProbeError::Send)?;

        let mut buffer = [0_u8; 2048];
        let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
        let packet = &buffer[..received];
        let Some((reply_path, value)) = parse_headamp_index_value(packet) else {
            return Err(ProbeError::Protocol(format!(
                "unexpected OSC reply while reading headamp index for {target}"
            )));
        };

        if reply_path != path {
            return Err(ProbeError::Protocol(format!(
                "received headamp index reply for '{reply_path}' while reading {target}"
            )));
        }

        if value < 0 {
            Ok(None)
        } else {
            Ok(Some(value as u8))
        }
    }
}

fn gain_uses_headamp(target: FaderTarget) -> bool {
    !matches!(target, FaderTarget::Channel(17..=32))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    Connected {
        responder: SocketAddr,
        response: ProbeResponse,
        model: Option<MixerModel>,
    },
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeResponse {
    Info,
    Status,
    XInfo,
    Unknown,
}

#[derive(Debug)]
pub enum ProbeError {
    Bind(io::Error),
    Configure(io::Error),
    Send(io::Error),
    Receive(io::Error),
    Protocol(String),
}

#[derive(Debug, Clone)]
pub struct MuteBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

#[derive(Debug, Clone)]
pub struct PanBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

#[derive(Debug, Clone)]
pub struct GainBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

#[derive(Debug, Clone)]
pub struct SendBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

#[derive(Debug, Clone)]
pub struct NameBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

#[derive(Debug, Clone)]
pub struct ColorBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

#[derive(Debug, Clone)]
pub struct SoloBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

#[derive(Debug, Clone)]
pub struct ParameterProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
}

impl ParameterProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn get(&self, path: &str) -> Result<crate::parameters::OscValue, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let request = crate::parameters::build_get(path);
        socket
            .send_to(&request, self.target)
            .map_err(ProbeError::Send)?;

        let mut buffer = [0_u8; 2048];
        let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
        let packet = &buffer[..received];
        let Some((reply_path, value)) = crate::parameters::parse_osc_value(packet) else {
            return Err(ProbeError::Protocol(format!(
                "unexpected OSC reply while reading {path}"
            )));
        };

        if reply_path != path {
            return Err(ProbeError::Protocol(format!(
                "received reply for '{reply_path}' while reading {path}"
            )));
        }

        Ok(value)
    }

    pub fn set(&self, path: &str, value: crate::parameters::OscValue) -> Result<(), ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let packet = crate::parameters::build_set(path, value);
        socket
            .send_to(&packet, self.target)
            .map_err(ProbeError::Send)?;
        Ok(())
    }

    pub fn set_multi(
        &self,
        path: &str,
        values: &[crate::parameters::OscValue],
    ) -> Result<(), ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let packet = crate::parameters::build_set_multi(path, values);
        socket
            .send_to(&packet, self.target)
            .map_err(ProbeError::Send)?;
        Ok(())
    }

    pub fn load_batch(
        &self,
        paths: &[String],
    ) -> Result<Vec<(String, crate::parameters::OscValue)>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            let request = crate::parameters::build_get(path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((reply_path, value)) = crate::parameters::parse_osc_value(packet) else {
                continue;
            };

            if reply_path == *path {
                results.push((reply_path, value));
            }
        }
        Ok(results)
    }
}

impl SoloBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load(&self, targets: &[FaderTarget]) -> Result<Vec<StripSolo>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut solos = Vec::with_capacity(targets.len());

        for &target in targets {
            let path = solo_path(target, self.model);
            let request = osc_query(&path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((reply_path, on)) = parse_switch_value(packet) else {
                return Err(ProbeError::Protocol(format!(
                    "unexpected OSC reply while reading solo for {target}"
                )));
            };

            if reply_path != path {
                return Err(ProbeError::Protocol(format!(
                    "received solo reply for '{reply_path}' while reading {target}"
                )));
            }

            solos.push(StripSolo { target, on });
        }

        Ok(solos)
    }

    pub fn set(&self, target: FaderTarget, on: bool) -> Result<(), ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let packet = osc_int_message(&solo_path(target, self.model), i32::from(on));
        socket
            .send_to(&packet, self.target)
            .map_err(ProbeError::Send)?;
        Ok(())
    }
}

impl MuteBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load(&self, targets: &[FaderTarget]) -> Result<Vec<StripMute>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut mutes = Vec::with_capacity(targets.len());

        for &target in targets {
            let path = mute_path(target, self.model);
            let request = osc_query(&path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((reply_path, on)) = parse_switch_value(packet) else {
                return Err(ProbeError::Protocol(format!(
                    "unexpected OSC reply while reading mute for {target}"
                )));
            };

            if reply_path != path {
                return Err(ProbeError::Protocol(format!(
                    "received mute reply for '{reply_path}' while reading {target}"
                )));
            }

            mutes.push(StripMute { target, on });
        }

        Ok(mutes)
    }

    pub fn set(&self, target: FaderTarget, on: bool) -> Result<(), ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let packet = osc_int_message(&mute_path(target, self.model), i32::from(on));
        socket
            .send_to(&packet, self.target)
            .map_err(ProbeError::Send)?;
        Ok(())
    }
}

impl NameBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load(&self, targets: &[FaderTarget]) -> Result<Vec<StripName>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut names = Vec::with_capacity(targets.len());

        for &target in targets {
            let path = name_path(target, self.model);
            let request = osc_query(&path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((reply_path, value)) = parse_string_value(packet) else {
                return Err(ProbeError::Protocol(format!(
                    "unexpected OSC reply while reading name for {target}"
                )));
            };

            if reply_path != path {
                return Err(ProbeError::Protocol(format!(
                    "received name reply for '{reply_path}' while reading {target}"
                )));
            }

            names.push(StripName { target, value });
        }

        Ok(names)
    }
}

impl ColorBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load(&self, targets: &[FaderTarget]) -> Result<Vec<StripColor>, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;

        let mut colors = Vec::with_capacity(targets.len());

        for &target in targets {
            let path = color_path(target, self.model);
            let request = osc_query(&path);
            socket
                .send_to(&request, self.target)
                .map_err(ProbeError::Send)?;

            let mut buffer = [0_u8; 2048];
            let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
            let packet = &buffer[..received];
            let Some((reply_path, value)) = parse_color_value(packet) else {
                return Err(ProbeError::Protocol(format!(
                    "unexpected OSC reply while reading color for {target}"
                )));
            };

            if reply_path != path {
                return Err(ProbeError::Protocol(format!(
                    "received color reply for '{reply_path}' while reading {target}"
                )));
            }

            colors.push(StripColor { target, value });
        }

        Ok(colors)
    }
}

#[derive(Debug, Clone)]
pub struct MeterBankProbe {
    target: SocketAddr,
    bind_addr: SocketAddr,
    timeout: Duration,
    model: MixerModel,
}

impl MeterBankProbe {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            timeout: Duration::from_millis(400),
            model: MixerModel::X32,
        }
    }

    pub fn with_model(mut self, model: MixerModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn load_inputs(&self) -> Result<Vec<StripMeter>, ProbeError> {
        let socket = self.bind_socket()?;
        let request = osc_meter_group_request(INPUT_METERS_REQUEST);
        socket
            .send_to(&request, self.target)
            .map_err(ProbeError::Send)?;
        let mut buffer = [0_u8; 4096];
        let (received, _) = socket.recv_from(&mut buffer).map_err(ProbeError::Receive)?;
        parse_input_meter_packet(&buffer[..received], self.model)
    }

    fn bind_socket(&self) -> Result<UdpSocket, ProbeError> {
        let socket = UdpSocket::bind(self.bind_addr).map_err(ProbeError::Bind)?;
        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        socket
            .set_write_timeout(Some(self.timeout))
            .map_err(ProbeError::Configure)?;
        Ok(socket)
    }
}

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bind(error) => write!(f, "failed to bind UDP socket: {error}"),
            Self::Configure(error) => write!(f, "failed to configure UDP socket: {error}"),
            Self::Send(error) => write!(f, "failed to send probe to mixer: {error}"),
            Self::Receive(error) => write!(f, "failed to receive mixer response: {error}"),
            Self::Protocol(error) => write!(f, "invalid mixer protocol data: {error}"),
        }
    }
}

impl std::error::Error for ProbeError {}

pub fn parse_target(input: &str) -> Result<SocketAddr, ParseTargetError> {
    if let Ok(addr) = input.parse::<SocketAddr>() {
        return Ok(addr);
    }

    let candidate = format!("{input}:{X32_DEFAULT_PORT}");
    let mut resolved = candidate.to_socket_addrs()?;
    resolved.next().ok_or(ParseTargetError::NoResolvedAddress)
}

#[derive(Debug)]
pub enum ParseTargetError {
    Resolve(io::Error),
    NoResolvedAddress,
}

impl std::fmt::Display for ParseTargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve(error) => write!(f, "failed to resolve mixer address: {error}"),
            Self::NoResolvedAddress => write!(f, "mixer address did not resolve to a socket"),
        }
    }
}

impl std::error::Error for ParseTargetError {}

impl From<io::Error> for ParseTargetError {
    fn from(value: io::Error) -> Self {
        Self::Resolve(value)
    }
}

fn parse_response(packet: &[u8]) -> ProbeResponse {
    match osc_address(packet) {
        Some(INFO_RESPONSE) => ProbeResponse::Info,
        Some(STATUS_RESPONSE) => ProbeResponse::Status,
        Some(XINFO_RESPONSE) => ProbeResponse::XInfo,
        _ => ProbeResponse::Unknown,
    }
}

fn parse_discovered_mixer(packet: &[u8], responder: SocketAddr) -> Option<DiscoveredMixer> {
    if !matches!(parse_response(packet), ProbeResponse::XInfo) {
        return None;
    }

    let strings = osc_strings(packet);

    Some(DiscoveredMixer {
        addr: responder,
        network_address: strings.first().cloned(),
        name: strings.get(1).cloned(),
        model: strings
            .get(2)
            .and_then(|s| MixerModel::from_model_string(s))
            .unwrap_or(MixerModel::X32),
        firmware: strings.get(3).cloned(),
    })
}

fn fader_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::fader_path(target),
        MixerModel::XR18 => crate::xr18::fader_path(target),
    }
}

fn pan_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::pan_path(target),
        MixerModel::XR18 => crate::xr18::pan_path(target),
    }
}

fn gain_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::gain_path(target),
        MixerModel::XR18 => crate::xr18::gain_path(target),
    }
}

fn headamp_index_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::headamp_index_path(target),
        MixerModel::XR18 => crate::xr18::headamp_index_path(target),
    }
}

fn headamp_gain_path(index: u8, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::headamp_gain_path(index),
        MixerModel::XR18 => crate::xr18::headamp_gain_path(index),
    }
}

fn _headamp_index_from_gain_path(path: &str, model: MixerModel) -> Option<u8> {
    match model {
        MixerModel::X32 => crate::x32::headamp_index_from_gain_path(path),
        MixerModel::XR18 => crate::xr18::headamp_index_from_gain_path(path),
    }
}

fn send_level_path(target: FaderTarget, bus: u8, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::send_level_path(target, bus),
        MixerModel::XR18 => crate::xr18::send_level_path(target, bus),
    }
}

fn mute_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::mute_path(target),
        MixerModel::XR18 => crate::xr18::mute_path(target),
    }
}

fn solo_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::solo_path(target),
        MixerModel::XR18 => crate::xr18::solo_path(target),
    }
}

fn name_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::name_path(target),
        MixerModel::XR18 => crate::xr18::name_path(target),
    }
}

fn color_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::x32::color_path(target),
        MixerModel::XR18 => crate::xr18::color_path(target),
    }
}

pub fn osc_address(packet: &[u8]) -> Option<&str> {
    let end = packet.iter().position(|byte| *byte == 0)?;
    std::str::from_utf8(&packet[..end]).ok()
}

pub fn osc_meter_group_request(meter_id: &str) -> Vec<u8> {
    let mut packet = osc_string("/meters");
    packet.extend_from_slice(b",s\0\0");
    packet.extend_from_slice(&osc_string(meter_id));
    packet
}

pub fn batchsubscribe_meter_request(
    alias: &str,
    meter_id: &str,
    arg0: i32,
    arg1: i32,
    time_factor: i32,
) -> Vec<u8> {
    let mut packet = osc_string("/batchsubscribe");
    packet.extend_from_slice(b",ssiii\0\0");
    packet.extend_from_slice(&osc_string(alias));
    packet.extend_from_slice(&osc_string(meter_id));
    packet.extend_from_slice(&arg0.to_be_bytes());
    packet.extend_from_slice(&arg1.to_be_bytes());
    packet.extend_from_slice(&time_factor.to_be_bytes());
    packet
}

pub fn renew_request(alias: &str) -> Vec<u8> {
    let mut packet = osc_string("/renew");
    packet.extend_from_slice(b",s\0\0");
    packet.extend_from_slice(&osc_string(alias));
    packet
}

pub fn parse_console_update(packet: &[u8], model: MixerModel) -> Option<ConsoleUpdate> {
    match model {
        MixerModel::X32 => crate::x32::parse_console_update(packet),
        MixerModel::XR18 => crate::xr18::parse_console_update(packet),
    }
}

fn osc_query(address: &str) -> Vec<u8> {
    osc_string(address)
}

pub fn osc_float_message(address: &str, value: f32) -> Vec<u8> {
    let mut packet = osc_string(address);
    packet.extend_from_slice(b",f\0\0");
    packet.extend_from_slice(&value.to_bits().to_be_bytes());
    packet
}

pub fn osc_int_message(address: &str, value: i32) -> Vec<u8> {
    let mut packet = osc_string(address);
    packet.extend_from_slice(b",i\0\0");
    packet.extend_from_slice(&value.to_be_bytes());
    packet
}

pub fn osc_string_message(address: &str, value: &str) -> Vec<u8> {
    let mut packet = osc_string(address);
    packet.extend_from_slice(b",s\0\0");
    packet.extend_from_slice(&osc_string(value));
    packet
}

pub fn osc_string(value: &str) -> Vec<u8> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    bytes
}

pub(crate) fn decode_trim_gain(raw: f32) -> f32 {
    quantize_gain_step(raw.clamp(0.0, 1.0) * 36.0 - 18.0, -18.0, 0.25)
}

pub(crate) fn encode_trim_gain(db: f32) -> f32 {
    ((quantize_gain_step(db, -18.0, 0.25) + 18.0) / 36.0).clamp(0.0, 1.0)
}

pub(crate) fn quantize_gain_step(value: f32, min: f32, step: f32) -> f32 {
    let steps = ((value - min) / step).round();
    min + steps * step
}

#[derive(Debug, Clone, Copy)]
pub struct MainMeterLevels {
    pub mains: [f32; 16],
    pub main_lr: [f32; 2],
    pub matrices: [f32; 6],
}

pub(crate) fn parse_float_value(packet: &[u8], suffix: &str) -> Option<(String, f32)> {
    let path = osc_address(packet)?;
    if !path.ends_with(suffix) {
        return None;
    }

    let mut offset = osc_padded_len(packet)?;
    let type_tag_end = packet.get(offset..)?.iter().position(|byte| *byte == 0)?;
    let type_tag = std::str::from_utf8(packet.get(offset..offset + type_tag_end)?).ok()?;
    let type_tag_len = osc_padded_len(packet.get(offset..)?)?;
    offset += type_tag_len;

    if type_tag != ",f" {
        return None;
    }

    let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
    Some((
        path.to_owned(),
        f32::from_bits(u32::from_be_bytes(value_bytes)),
    ))
}

pub(crate) fn parse_int_value(packet: &[u8]) -> Option<(String, i32)> {
    let path = osc_address(packet)?;

    let mut offset = osc_padded_len(packet)?;
    let type_tag_end = packet.get(offset..)?.iter().position(|byte| *byte == 0)?;
    let type_tag = std::str::from_utf8(packet.get(offset..offset + type_tag_end)?).ok()?;
    let type_tag_len = osc_padded_len(packet.get(offset..)?)?;
    offset += type_tag_len;

    if type_tag != ",i" {
        return None;
    }

    let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
    Some((path.to_owned(), i32::from_be_bytes(value_bytes)))
}

fn parse_fader_value(packet: &[u8]) -> Option<(String, f32)> {
    if let Some(result) = parse_float_value(packet, FADER_RESPONSE_SUFFIX) {
        return Some(result);
    }
    let (path, value) = parse_float_value(packet, "/fader")?;
    if !path.starts_with("/dca/") {
        return None;
    }
    Some((path, value))
}

fn parse_pan_value(packet: &[u8]) -> Option<(String, f32)> {
    parse_float_value(packet, PAN_RESPONSE_SUFFIX)
}

fn parse_gain_value(packet: &[u8]) -> Option<(String, f32)> {
    parse_float_value(packet, GAIN_RESPONSE_SUFFIX)
}

fn parse_headamp_gain_value(packet: &[u8]) -> Option<(String, f32)> {
    let (path, value) = parse_float_value(packet, HEADAMP_GAIN_RESPONSE_SUFFIX)?;
    path.starts_with("/headamp/").then_some((path, value))
}

fn parse_headamp_index_value(packet: &[u8]) -> Option<(String, i32)> {
    let (path, value) = parse_int_value(packet)?;
    if path.starts_with("/-ha/") && path.ends_with(HEADAMP_INDEX_RESPONSE_SUFFIX) {
        Some((path, value))
    } else {
        None
    }
}

fn parse_send_value(packet: &[u8]) -> Option<(String, f32)> {
    let (path, value) = parse_float_value(packet, "/level")?;

    if !path.contains("/mix/") || !path.ends_with("/level") {
        return None;
    }
    Some((path, value))
}

pub(crate) fn parse_switch_value(packet: &[u8]) -> Option<(String, bool)> {
    let path = osc_address(packet)?;
    if !path.ends_with(MUTE_RESPONSE_SUFFIX)
        && !path.starts_with(SOLO_RESPONSE_PREFIX)
        && !is_dca_mute_path(path)
    {
        return None;
    }

    let mut offset = osc_padded_len(packet)?;
    let type_tag_end = packet.get(offset..)?.iter().position(|byte| *byte == 0)?;
    let type_tag = std::str::from_utf8(packet.get(offset..offset + type_tag_end)?).ok()?;
    let type_tag_len = osc_padded_len(packet.get(offset..)?)?;
    offset += type_tag_len;

    match type_tag {
        ",i" => {
            let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
            Some((path.to_owned(), i32::from_be_bytes(value_bytes) != 0))
        }
        ",f" => {
            let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
            Some((path.to_owned(), f32::from_be_bytes(value_bytes) != 0.0))
        }
        _ => None,
    }
}

fn is_dca_mute_path(path: &str) -> bool {
    path.strip_prefix("/dca/")
        .and_then(|rest| {
            rest.strip_suffix("/mix/on")
                .or_else(|| rest.strip_suffix("/on"))
        })
        .and_then(|index| index.parse::<u8>().ok())
        .is_some()
}

pub(crate) fn parse_string_value(packet: &[u8]) -> Option<(String, String)> {
    let path = osc_address(packet)?;
    if !path.ends_with(NAME_RESPONSE_SUFFIX) {
        return None;
    }

    let mut offset = osc_padded_len(packet)?;
    let type_tag_end = packet.get(offset..)?.iter().position(|byte| *byte == 0)?;
    let type_tag = std::str::from_utf8(packet.get(offset..offset + type_tag_end)?).ok()?;
    let type_tag_len = osc_padded_len(packet.get(offset..)?)?;
    offset += type_tag_len;

    if type_tag != ",s" {
        return None;
    }

    let value_bytes = packet.get(offset..)?;
    let value_end = value_bytes.iter().position(|byte| *byte == 0)?;
    let value = std::str::from_utf8(&value_bytes[..value_end]).ok()?;
    Some((path.to_owned(), value.to_owned()))
}

pub(crate) fn parse_color_value(packet: &[u8]) -> Option<(String, u8)> {
    let path = osc_address(packet)?;
    if !path.ends_with(COLOR_RESPONSE_SUFFIX) {
        return None;
    }

    let mut offset = osc_padded_len(packet)?;
    let type_tag_end = packet.get(offset..)?.iter().position(|byte| *byte == 0)?;
    let type_tag = std::str::from_utf8(packet.get(offset..offset + type_tag_end)?).ok()?;
    let type_tag_len = osc_padded_len(packet.get(offset..)?)?;
    offset += type_tag_len;

    if type_tag != ",i" {
        return None;
    }

    let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
    let value = i32::from_be_bytes(value_bytes).clamp(0, 15) as u8;
    Some((path.to_owned(), value))
}

pub fn parse_input_meter_packet(
    packet: &[u8],
    model: MixerModel,
) -> Result<Vec<StripMeter>, ProbeError> {
    match model {
        MixerModel::X32 => crate::x32::parse_input_meter_packet(packet),
        MixerModel::XR18 => crate::xr18::parse_input_meter_packet(packet),
    }
}

pub fn parse_main_meter_packet(
    packet: &[u8],
    model: MixerModel,
) -> Result<MainMeterLevels, ProbeError> {
    match model {
        MixerModel::X32 => crate::x32::parse_main_meter_packet(packet),
        MixerModel::XR18 => crate::xr18::parse_main_meter_packet(packet),
    }
}

pub fn parse_rta_meter_packet(packet: &[u8], model: MixerModel) -> Result<[f32; 100], ProbeError> {
    match model {
        MixerModel::X32 => crate::x32::parse_rta_meter_packet(packet),
        MixerModel::XR18 => Err(ProbeError::Protocol("RTA not supported on XR18".to_owned())),
    }
}

pub(crate) fn parse_meter_blob<'a>(
    packet: &'a [u8],
    expected_path: &str,
    expected_alias: &str,
) -> Result<&'a [u8], ProbeError> {
    let path = osc_address(packet)
        .ok_or_else(|| ProbeError::Protocol("meter reply missing OSC address".to_owned()))?;
    if path != expected_path && path != expected_alias {
        return Err(ProbeError::Protocol(format!(
            "unexpected meter reply path '{path}'"
        )));
    }

    let mut offset = osc_padded_len(packet)
        .ok_or_else(|| ProbeError::Protocol("meter reply has invalid OSC address".to_owned()))?;
    let type_tag_end = packet[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| ProbeError::Protocol("meter reply missing OSC type tag".to_owned()))?;
    let type_tag = std::str::from_utf8(&packet[offset..offset + type_tag_end])
        .map_err(|_| ProbeError::Protocol("meter reply type tag is not UTF-8".to_owned()))?;
    if type_tag != ",b" {
        return Err(ProbeError::Protocol(format!(
            "unexpected meter reply type tag '{type_tag}'"
        )));
    }
    offset += osc_padded_len(&packet[offset..])
        .ok_or_else(|| ProbeError::Protocol("meter reply has invalid type tag".to_owned()))?;

    let blob_len = read_be_u32(packet, offset)? as usize;
    offset += 4;
    let blob = packet
        .get(offset..offset + blob_len)
        .ok_or_else(|| ProbeError::Protocol("meter blob length exceeds packet size".to_owned()))?;
    if blob.len() < 4 {
        return Err(ProbeError::Protocol(
            "meter blob is missing float-count header".to_owned(),
        ));
    }

    let float_count = u32::from_le_bytes(
        blob[0..4]
            .try_into()
            .map_err(|_| ProbeError::Protocol("meter float-count size mismatch".to_owned()))?,
    ) as usize;
    let floats = &blob[4..];

    if floats.len() < float_count * 4 {
        return Err(ProbeError::Protocol(
            "meter blob is shorter than advertised float count".to_owned(),
        ));
    }

    Ok(floats)
}

pub(crate) fn read_be_u32(packet: &[u8], offset: usize) -> Result<u32, ProbeError> {
    let bytes: [u8; 4] = packet
        .get(offset..offset + 4)
        .ok_or_else(|| ProbeError::Protocol("packet truncated while reading u32".to_owned()))?
        .try_into()
        .map_err(|_| ProbeError::Protocol("u32 slice size mismatch".to_owned()))?;
    Ok(u32::from_be_bytes(bytes))
}

fn osc_strings(packet: &[u8]) -> Vec<String> {
    let Some(mut offset) = osc_padded_len(packet) else {
        return Vec::new();
    };
    let Some(type_tag_len) = packet.get(offset..).and_then(osc_padded_len) else {
        return Vec::new();
    };
    offset += type_tag_len;

    let mut values = Vec::new();

    while offset < packet.len() {
        let bytes = &packet[offset..];
        let Some(end) = bytes.iter().position(|byte| *byte == 0) else {
            break;
        };

        let Some(value) = std::str::from_utf8(&bytes[..end]).ok() else {
            break;
        };
        let Some(padded_len) = osc_padded_len(bytes) else {
            break;
        };

        let value = value.to_owned();
        values.push(value);
        offset += padded_len;
    }

    values
}

pub fn osc_padded_len(bytes: &[u8]) -> Option<usize> {
    let end = bytes.iter().position(|byte| *byte == 0)?;
    let raw = end + 1;
    Some((raw + 3) & !3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_osc_address_from_packet() {
        assert_eq!(osc_address(b"/info\0\0\0,\0\0\0"), Some("/info"));
    }

    #[test]
    fn identifies_known_probe_responses() {
        assert_eq!(parse_response(b"/status\0,\0\0\0"), ProbeResponse::Status);
        assert_eq!(parse_response(b"/xinfo\0\0,\0\0\0"), ProbeResponse::XInfo);
    }

    #[test]
    fn applies_default_port_to_bare_host() {
        let target = parse_target("127.0.0.1").expect("should parse localhost");
        assert_eq!(target.port(), X32_DEFAULT_PORT);
    }

    #[test]
    fn parses_xinfo_discovery_payload() {
        let packet = concat!(
            "/xinfo\0\0,\0\0\0",
            "192.168.1.62\0\0\0\0",
            "X32-024A-53\0",
            "X32\0",
            "3.04\0\0\0"
        )
        .as_bytes();
        let responder = SocketAddr::from(([192, 168, 1, 62], X32_DEFAULT_PORT));

        let mixer = parse_discovered_mixer(packet, responder).expect("xinfo should parse");

        assert_eq!(mixer.addr, responder);
        assert_eq!(mixer.network_address.as_deref(), Some("192.168.1.62"));
        assert_eq!(mixer.name.as_deref(), Some("X32-024A-53"));
        assert_eq!(mixer.model, MixerModel::X32);
        assert_eq!(mixer.firmware.as_deref(), Some("3.04"));
    }

    #[test]
    fn builds_query_packet_for_channel_fader() {
        assert_eq!(
            osc_query(&crate::x32::fader_path(FaderTarget::Channel(1))),
            b"/ch/01/mix/fader\0\0\0\0".to_vec()
        );
    }

    #[test]
    fn parses_float_fader_reply() {
        let packet = [
            b"/ch/01/mix/fader\0\0\0\0".as_slice(),
            b",f\0\0".as_slice(),
            0.75_f32.to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, value) = parse_fader_value(&packet).expect("should parse fader reply");
        assert_eq!(path, "/ch/01/mix/fader");
        assert!((value - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_float_pan_reply() {
        let packet = [
            osc_string("/auxin/05/mix/pan").as_slice(),
            b",f\0\0".as_slice(),
            0.25_f32.to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, value) = parse_pan_value(&packet).expect("should parse pan reply");
        assert_eq!(path, "/auxin/05/mix/pan");
        assert!((value - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_float_gain_reply() {
        let packet = [
            osc_string("/ch/02/preamp/trim").as_slice(),
            b",f\0\0".as_slice(),
            (-6.0_f32).to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, value) = parse_gain_value(&packet).expect("should parse gain reply");
        assert_eq!(path, "/ch/02/preamp/trim");
        assert!((value + 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_float_send_reply() {
        let packet = [
            osc_string("/ch/02/mix/16/level").as_slice(),
            b",f\0\0".as_slice(),
            0.5_f32.to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, value) = parse_send_value(&packet).expect("should parse send reply");
        assert_eq!(path, "/ch/02/mix/16/level");
        assert!((value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_int_mute_reply() {
        let packet = osc_int_message("/auxin/05/mix/on", 0);

        let (path, on) = parse_switch_value(&packet).expect("should parse mute reply");
        assert_eq!(path, "/auxin/05/mix/on");
        assert!(!on);
    }

    #[test]
    fn parses_int_solo_reply() {
        let packet = osc_int_message("/-stat/solosw/37", 1);

        let (path, on) = parse_switch_value(&packet).expect("should parse solo reply");
        assert_eq!(path, "/-stat/solosw/37");
        assert!(on);
    }

    #[test]
    fn parses_string_name_reply() {
        let packet = [
            osc_string("/auxin/05/config/name").as_slice(),
            b",s\0\0".as_slice(),
            osc_string("Lead Vox").as_slice(),
        ]
        .concat();

        let (path, value) = parse_string_value(&packet).expect("should parse name reply");
        assert_eq!(path, "/auxin/05/config/name");
        assert_eq!(value, "Lead Vox");
    }

    #[test]
    fn builds_meter_request_packet() {
        let packet = osc_meter_group_request(INPUT_METERS_REQUEST);
        assert_eq!(&packet[..8], b"/meters\0");
        assert_eq!(&packet[8..12], b",s\0\0");
        assert_eq!(&packet[12..24], b"/meters/0\0\0\0");
    }

    #[test]
    fn parses_input_meter_blob() {
        let mut floats = Vec::new();
        for i in 0..82 {
            floats.extend_from_slice(&((i as f32) / 10.0).to_le_bytes());
        }
        let mut blob = Vec::new();
        blob.extend_from_slice(&(82_u32).to_le_bytes());
        blob.extend_from_slice(&floats);

        let mut packet = osc_string(INPUT_METERS_ALIAS);
        packet.extend_from_slice(b",b\0\0");
        packet.extend_from_slice(&(blob.len() as u32).to_be_bytes());
        packet.extend_from_slice(&blob);

        let meters = parse_input_meter_packet(&packet, MixerModel::X32)
            .expect("should parse input meter blob");
        assert_eq!(meters.len(), 48);
        assert_eq!(meters[0].target, FaderTarget::Channel(1));
        assert_eq!(meters[31].target, FaderTarget::Channel(32));
        assert_eq!(meters[32].target, FaderTarget::Aux(1));
        assert_eq!(meters[39].target, FaderTarget::Aux(8));
        assert_eq!(meters[40].target, FaderTarget::FxRtn(1));
        assert_eq!(meters[47].target, FaderTarget::FxRtn(8));
        assert!((meters[5].level_linear - 0.5).abs() < f32::EPSILON);
        assert!((meters[35].level_linear - 3.5).abs() < f32::EPSILON);
    }

    #[test]
    fn builds_batchsubscribe_meter_request_packet() {
        let packet = batchsubscribe_meter_request("meters/0", "/meters/0", 0, 0, 1);
        assert_eq!(&packet[..16], b"/batchsubscribe\0");
        assert_eq!(&packet[16..24], b",ssiii\0\0");
    }

    #[test]
    fn builds_renew_request_packet() {
        let packet = renew_request("meters/0");
        assert_eq!(&packet[..8], b"/renew\0\0");
    }

    #[test]
    fn builds_query_packet_for_bus_fader() {
        assert_eq!(
            osc_query(&crate::x32::fader_path(FaderTarget::Bus(1))),
            b"/bus/01/mix/fader\0\0\0".to_vec()
        );
    }

    #[test]
    fn parses_bus_fader_reply() {
        let packet = [
            osc_string("/bus/05/mix/fader").as_slice(),
            b",f\0\0".as_slice(),
            0.75_f32.to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, value) = parse_fader_value(&packet).expect("should parse bus fader reply");
        assert_eq!(path, "/bus/05/mix/fader");
        assert!((value - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_bus_pan_reply() {
        let packet = [
            osc_string("/bus/03/mix/pan").as_slice(),
            b",f\0\0".as_slice(),
            0.25_f32.to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, value) = parse_pan_value(&packet).expect("should parse bus pan reply");
        assert_eq!(path, "/bus/03/mix/pan");
        assert!((value - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_bus_send_reply() {
        let packet = [
            osc_string("/bus/02/mix/06/level").as_slice(),
            b",f\0\0".as_slice(),
            0.5_f32.to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, value) = parse_send_value(&packet).expect("should parse bus send reply");
        assert_eq!(path, "/bus/02/mix/06/level");
        assert!((value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_bus_mute_reply() {
        let packet = osc_int_message("/bus/07/mix/on", 0);

        let (path, on) = parse_switch_value(&packet).expect("should parse bus mute reply");
        assert_eq!(path, "/bus/07/mix/on");
        assert!(!on);
    }

    #[test]
    fn parses_bus_solo_reply() {
        let packet = osc_int_message("/-stat/solosw/52", 1);

        let (path, on) = parse_switch_value(&packet).expect("should parse bus solo reply");
        assert_eq!(path, "/-stat/solosw/52");
        assert!(on);
    }

    #[test]
    fn parses_bus_name_reply() {
        let packet = [
            osc_string("/bus/08/config/name").as_slice(),
            b",s\0\0".as_slice(),
            osc_string("Drums").as_slice(),
        ]
        .concat();

        let (path, value) = parse_string_value(&packet).expect("should parse bus name reply");
        assert_eq!(path, "/bus/08/config/name");
        assert_eq!(value, "Drums");
    }

    #[test]
    fn parses_dca_mute_reply_with_on_suffix() {
        let packet = osc_int_message("/dca/3/on", 0);

        let (path, on) = parse_switch_value(&packet).expect("should parse DCA mute reply");
        assert_eq!(path, "/dca/3/on");
        assert!(!on);
    }

    #[test]
    fn parses_dca_mute_reply_with_mix_on_suffix() {
        let packet = osc_int_message("/dca/5/mix/on", 1);

        let (path, on) = parse_switch_value(&packet).expect("should parse DCA /mix/on mute reply");
        assert_eq!(path, "/dca/5/mix/on");
        assert!(on);
    }

    #[test]
    fn parses_dca_mute_reply_as_float() {
        let packet = [
            osc_string("/dca/2/on").as_slice(),
            b",f\0\0".as_slice(),
            1.0_f32.to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, on) = parse_switch_value(&packet).expect("should parse DCA float mute reply");
        assert_eq!(path, "/dca/2/on");
        assert!(on);
    }

    #[test]
    fn parses_fxrtn_mute_reply_as_float() {
        let packet = [
            osc_string("/fxrtn/03/mix/on").as_slice(),
            b",f\0\0".as_slice(),
            0.0_f32.to_bits().to_be_bytes().as_slice(),
        ]
        .concat();

        let (path, on) =
            parse_switch_value(&packet).expect("should parse FX return float mute reply");
        assert_eq!(path, "/fxrtn/03/mix/on");
        assert!(!on);
    }

    #[test]
    fn parses_dca_mix_on_mute_console_update() {
        let packet = osc_int_message("/dca/6/mix/on", 0);
        let update = parse_console_update(&packet, MixerModel::X32)
            .expect("should parse DCA /mix/on mute update");
        assert_eq!(
            update,
            ConsoleUpdate::Mute(StripMute {
                target: FaderTarget::Dca(6),
                on: false,
            })
        );
    }
}
