use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

use crate::codec::parse_input_meter_packet;
use crate::codec::{
    color_path, fader_path, gain_path, headamp_gain_path, headamp_index_path, mute_path, name_path,
    pan_path, send_level_path, solo_path,
};
use crate::model::{
    DiscoveredMixer, FaderTarget, GainSource, MixerModel, StripColor, StripFader, StripGain,
    StripMeter, StripMute, StripName, StripPan, StripSend, StripSolo,
};
use crate::osc::{
    INPUT_METERS_REQUEST, decode_trim_gain, encode_trim_gain, osc_address, osc_float_message,
    osc_int_message, osc_meter_group_request, osc_query, osc_strings, parse_color_value,
    parse_fader_value, parse_gain_value, parse_headamp_gain_value, parse_headamp_index_value,
    parse_pan_value, parse_send_value, parse_string_value, parse_switch_value,
};

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
                crate::codec::x32::encode_headamp_gain(value),
            ),
            (GainSource::Headamp(index), MixerModel::XR18) => osc_float_message(
                &headamp_gain_path(index, self.model),
                crate::codec::xr18::encode_headamp_gain(value),
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
                    MixerModel::X32 => crate::codec::x32::decode_headamp_gain(value),
                    MixerModel::XR18 => crate::codec::xr18::decode_headamp_gain(value),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{parse_console_update, parse_input_meter_packet};
    use crate::model::ConsoleUpdate;
    use crate::osc::{INPUT_METERS_ALIAS, osc_string};

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
            osc_query(&crate::codec::x32::fader_path(FaderTarget::Channel(1))),
            b"/ch/01/mix/fader\0\0\0\0".to_vec()
        );
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
    fn builds_query_packet_for_bus_fader() {
        assert_eq!(
            osc_query(&crate::codec::x32::fader_path(FaderTarget::Bus(1))),
            b"/bus/01/mix/fader\0\0\0".to_vec()
        );
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
