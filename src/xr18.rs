use crate::common::{
    COLOR_RESPONSE_SUFFIX, ConsoleUpdate, FADER_RESPONSE_SUFFIX, FaderTarget, GAIN_RESPONSE_SUFFIX,
    HEADAMP_GAIN_RESPONSE_SUFFIX, MUTE_RESPONSE_SUFFIX, MainMeterLevels, NAME_RESPONSE_SUFFIX,
    PAN_RESPONSE_SUFFIX, ProbeError, SOLO_RESPONSE_PREFIX, StripColor, StripFader, StripGain,
    StripMeter, StripMute, StripName, StripPan, StripSend, StripSolo, parse_color_value,
    parse_float_value, parse_meter_blob, parse_string_value, parse_switch_value,
    quantize_gain_step,
};

pub fn fader_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/mix/fader"),
        FaderTarget::Bus(bus) => format!("/bus/{bus}/mix/fader"),
        FaderTarget::FxRtn(5) => "/rtn/aux/mix/fader".to_owned(),
        FaderTarget::FxRtn(fx) => format!("/rtn/{fx}/mix/fader"),
        FaderTarget::Dca(dca) => format!("/dca/{dca}/fader"),
        FaderTarget::Main => "/lr/mix/fader".to_owned(),
        _ => String::new(),
    }
}

pub fn pan_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/mix/pan"),
        FaderTarget::Bus(bus) => format!("/bus/{bus}/mix/pan"),
        FaderTarget::FxRtn(5) => "/rtn/aux/mix/pan".to_owned(),
        FaderTarget::FxRtn(fx) => format!("/rtn/{fx}/mix/pan"),
        FaderTarget::Dca(_) => String::new(),
        FaderTarget::Main => "/lr/mix/pan".to_owned(),
        _ => String::new(),
    }
}

pub fn gain_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/headamp/{channel:02}/gain"),
        _ => String::new(),
    }
}

pub fn headamp_index_path(_target: FaderTarget) -> String {
    String::new()
}

pub fn headamp_gain_path(index: u8) -> String {
    format!("/headamp/{index:02}/gain")
}

pub fn headamp_index_from_gain_path(path: &str) -> Option<u8> {
    path.strip_prefix("/headamp/")
        .and_then(|rest| rest.strip_suffix("/gain"))
        .and_then(|index| index.parse::<u8>().ok())
}

pub fn send_level_path(target: FaderTarget, bus: u8) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/mix/{bus:02}/level"),
        FaderTarget::FxRtn(5) => format!("/rtn/aux/mix/{bus:02}/level"),
        FaderTarget::FxRtn(fx) => format!("/rtn/{fx}/mix/{bus:02}/level"),
        _ => String::new(),
    }
}

pub fn mute_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/mix/on"),
        FaderTarget::Bus(bus) => format!("/bus/{bus}/mix/on"),
        FaderTarget::FxRtn(5) => "/rtn/aux/mix/on".to_owned(),
        FaderTarget::FxRtn(fx) => format!("/rtn/{fx}/mix/on"),
        FaderTarget::Dca(dca) => format!("/dca/{dca}/on"),
        FaderTarget::Main => "/lr/mix/on".to_owned(),
        _ => String::new(),
    }
}

pub fn solo_path(target: FaderTarget) -> String {
    let id = match target {
        FaderTarget::Channel(channel) => channel,
        FaderTarget::FxRtn(fx) if fx <= 4 => 16 + fx,
        FaderTarget::FxRtn(5) => 27,
        FaderTarget::Bus(bus) => 20 + bus,
        FaderTarget::Main => 27,
        FaderTarget::Dca(dca) => 50 + dca,
        _ => 0,
    };
    if id == 0 {
        return String::new();
    }
    format!("/-stat/solosw/{id:02}")
}

pub fn name_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/config/name"),
        FaderTarget::Bus(bus) => format!("/bus/{bus}/config/name"),
        FaderTarget::FxRtn(5) => "/rtn/aux/config/name".to_owned(),
        FaderTarget::FxRtn(fx) => format!("/rtn/{fx}/config/name"),
        FaderTarget::Dca(dca) => format!("/dca/{dca}/config/name"),
        FaderTarget::Main => "/lr/config/name".to_owned(),
        _ => String::new(),
    }
}

pub fn color_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/config/color"),
        FaderTarget::Bus(bus) => format!("/bus/{bus}/config/color"),
        FaderTarget::FxRtn(5) => "/rtn/aux/config/color".to_owned(),
        FaderTarget::FxRtn(fx) => format!("/rtn/{fx}/config/color"),
        FaderTarget::Dca(dca) => format!("/dca/{dca}/config/color"),
        FaderTarget::Main => "/lr/config/color".to_owned(),
        _ => String::new(),
    }
}

pub fn parse_console_update(packet: &[u8]) -> Option<ConsoleUpdate> {
    if let Some((path, value)) = parse_gain_value(packet)
        && let Some(target) = target_from_channel_path(&path, GAIN_RESPONSE_SUFFIX)
    {
        return Some(ConsoleUpdate::Gain(StripGain {
            target,
            value: decode_headamp_gain(value),
            source: crate::common::GainSource::Headamp(match target {
                FaderTarget::Channel(n) => n,
                _ => 0,
            }),
        }));
    }

    if let Some((path, value)) = parse_headamp_gain_value(packet)
        && let Some(index) = headamp_index_from_gain_path(&path)
    {
        return Some(ConsoleUpdate::HeadampGain {
            index,
            value: decode_headamp_gain(value),
        });
    }

    if let Some((path, value)) = parse_fader_value(packet)
        && let Some(target) = target_from_channel_path(&path, FADER_RESPONSE_SUFFIX)
    {
        return Some(ConsoleUpdate::Fader(StripFader { target, value }));
    }

    if let Some((path, value)) = parse_pan_value(packet)
        && let Some(target) = target_from_channel_path(&path, PAN_RESPONSE_SUFFIX)
    {
        return Some(ConsoleUpdate::Pan(StripPan { target, value }));
    }

    if let Some((target, bus, value)) = parse_send_update(packet) {
        return Some(ConsoleUpdate::Send(StripSend { target, bus, value }));
    }

    if let Some((path, on)) = parse_switch_value(packet) {
        if let Some(target) = target_from_channel_path(&path, MUTE_RESPONSE_SUFFIX) {
            return Some(ConsoleUpdate::Mute(StripMute { target, on }));
        }
        if let Some(target) = target_from_solo_path(&path) {
            return Some(ConsoleUpdate::Solo(StripSolo { target, on }));
        }
    }

    if let Some((path, value)) = parse_string_value(packet)
        && let Some(target) = target_from_channel_path(&path, NAME_RESPONSE_SUFFIX)
    {
        return Some(ConsoleUpdate::Name(StripName { target, value }));
    }

    if let Some((path, value)) = parse_color_value(packet)
        && let Some(target) = target_from_channel_path(&path, COLOR_RESPONSE_SUFFIX)
    {
        return Some(ConsoleUpdate::Color(StripColor { target, value }));
    }

    if let Some((path, value)) = crate::parameters::parse_osc_value(packet) {
        return Some(ConsoleUpdate::Parameter { path, value });
    }

    None
}

fn target_from_channel_path(path: &str, suffix: &str) -> Option<FaderTarget> {
    if let Some(index) = path
        .strip_prefix("/ch/")
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        return index.parse::<u8>().ok().map(FaderTarget::Channel);
    }

    if let Some(index) = path
        .strip_prefix("/bus/")
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        return index.parse::<u8>().ok().map(FaderTarget::Bus);
    }

    if let Some(index) = path
        .strip_prefix("/rtn/")
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        if index == "aux" {
            return Some(FaderTarget::FxRtn(5));
        }
        return index.parse::<u8>().ok().map(FaderTarget::FxRtn);
    }

    if suffix == FADER_RESPONSE_SUFFIX
        && let Some(index) = path
            .strip_prefix("/dca/")
            .and_then(|rest| rest.strip_suffix("/fader"))
    {
        return index.parse::<u8>().ok().map(FaderTarget::Dca);
    }
    if suffix == MUTE_RESPONSE_SUFFIX {
        if let Some(index) = path
            .strip_prefix("/dca/")
            .and_then(|rest| rest.strip_suffix("/mix/on"))
        {
            return index.parse::<u8>().ok().map(FaderTarget::Dca);
        }
        if let Some(index) = path
            .strip_prefix("/dca/")
            .and_then(|rest| rest.strip_suffix("/on"))
        {
            return index.parse::<u8>().ok().map(FaderTarget::Dca);
        }
    }
    if suffix == NAME_RESPONSE_SUFFIX
        && let Some(index) = path
            .strip_prefix("/dca/")
            .and_then(|rest| rest.strip_suffix("/config/name"))
    {
        return index.parse::<u8>().ok().map(FaderTarget::Dca);
    }

    if path == format!("/lr{suffix}") {
        return Some(FaderTarget::Main);
    }

    None
}

fn target_from_solo_path(path: &str) -> Option<FaderTarget> {
    let id = path
        .strip_prefix(SOLO_RESPONSE_PREFIX)?
        .parse::<u8>()
        .ok()?;
    match id {
        1..=16 => Some(FaderTarget::Channel(id)),
        17..=20 => Some(FaderTarget::FxRtn(id - 16)),
        21..=26 => Some(FaderTarget::Bus(id - 20)),
        27 => Some(FaderTarget::Main),
        51..=54 => Some(FaderTarget::Dca(id - 50)),
        _ => None,
    }
}

fn target_and_bus_from_send_path(path: &str) -> Option<(FaderTarget, u8)> {
    let (target, rest) = if let Some(rest) = path.strip_prefix("/ch/") {
        let (channel, rest) = rest.split_once('/')?;
        (FaderTarget::Channel(channel.parse::<u8>().ok()?), rest)
    } else if let Some(rest) = path.strip_prefix("/bus/") {
        let (bus, rest) = rest.split_once('/')?;
        (FaderTarget::Bus(bus.parse::<u8>().ok()?), rest)
    } else if let Some(rest) = path.strip_prefix("/rtn/aux/") {
        (FaderTarget::FxRtn(5), rest)
    } else if let Some(rest) = path.strip_prefix("/rtn/") {
        let (fx, rest) = rest.split_once('/')?;
        (FaderTarget::FxRtn(fx.parse::<u8>().ok()?), rest)
    } else if let Some(rest) = path.strip_prefix("/lr/") {
        (FaderTarget::Main, rest)
    } else {
        return None;
    };

    let rest = rest.strip_prefix("mix/")?;
    let (bus, tail) = rest.split_once('/')?;
    if tail != "level" {
        return None;
    }

    let bus = bus.parse::<u8>().ok()?;
    if !(1..=10).contains(&bus) {
        return None;
    }

    Some((target, bus))
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

fn parse_send_value(packet: &[u8]) -> Option<(String, f32)> {
    let (path, value) = parse_float_value(packet, "/level")?;
    target_and_bus_from_send_path(&path)?;
    Some((path, value))
}

fn parse_send_update(packet: &[u8]) -> Option<(FaderTarget, u8, f32)> {
    let (path, value) = parse_send_value(packet)?;
    let (target, bus) = target_and_bus_from_send_path(&path)?;
    Some((target, bus, value))
}

pub fn decode_headamp_gain(raw: f32) -> f32 {
    quantize_gain_step(raw.clamp(0.0, 1.0) * 32.0 - 12.0, -12.0, 0.1)
}

pub fn encode_headamp_gain(db: f32) -> f32 {
    ((quantize_gain_step(db, -12.0, 0.1) + 12.0) / 32.0).clamp(0.0, 1.0)
}

pub fn parse_input_meter_packet(packet: &[u8]) -> Result<Vec<StripMeter>, ProbeError> {
    let floats = parse_meter_blob(packet, "/meters/0", "meters/0")?;

    let mut strips = Vec::with_capacity(21);
    for index in 0..21 {
        let target = if index < 16 {
            FaderTarget::Channel((index + 1) as u8)
        } else if index < 20 {
            FaderTarget::FxRtn((index - 15) as u8)
        } else {
            FaderTarget::FxRtn(5)
        };
        let start = index * 4;
        if start + 4 > floats.len() {
            break;
        }
        let bytes: [u8; 4] = floats[start..start + 4]
            .try_into()
            .map_err(|_| ProbeError::Protocol("meter float slice size mismatch".to_owned()))?;
        strips.push(StripMeter {
            target,
            level_linear: f32::from_le_bytes(bytes),
        });
    }
    Ok(strips)
}

pub fn parse_main_meter_packet(packet: &[u8]) -> Result<MainMeterLevels, ProbeError> {
    let floats = parse_meter_blob(packet, "/meters/2", "meters/2")?;
    if floats.len() < 8 * 4 {
        return Err(ProbeError::Protocol(
            "main meter blob is shorter than expected".to_owned(),
        ));
    }

    let mut mains = [0.0f32; 16];
    for i in 0..6 {
        mains[i] = f32::from_le_bytes(floats[i * 4..i * 4 + 4].try_into().map_err(|_| {
            ProbeError::Protocol(format!("main meter {i} float slice size mismatch"))
        })?);
    }

    Ok(MainMeterLevels {
        mains,
        main_lr: [
            f32::from_le_bytes(floats[6 * 4..6 * 4 + 4].try_into().map_err(|_| {
                ProbeError::Protocol("main L meter float slice size mismatch".to_owned())
            })?),
            f32::from_le_bytes(floats[7 * 4..7 * 4 + 4].try_into().map_err(|_| {
                ProbeError::Protocol("main R meter float slice size mismatch".to_owned())
            })?),
        ],
        matrices: [0.0f32; 6],
    })
}
