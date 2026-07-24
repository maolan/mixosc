use crate::common::{
    COLOR_RESPONSE_SUFFIX, ConsoleUpdate, FADER_RESPONSE_SUFFIX, FaderTarget, GAIN_RESPONSE_SUFFIX,
    HEADAMP_GAIN_RESPONSE_SUFFIX, HEADAMP_INDEX_RESPONSE_SUFFIX, INPUT_METERS_ALIAS,
    INPUT_METERS_REQUEST, MAIN_METERS_ALIAS, MAIN_METERS_REQUEST, MUTE_RESPONSE_SUFFIX,
    MainMeterLevels, NAME_RESPONSE_SUFFIX, PAN_RESPONSE_SUFFIX, ProbeError, SOLO_RESPONSE_PREFIX,
    StripColor, StripFader, StripGain, StripMeter, StripMute, StripName, StripPan, StripSend,
    StripSolo, osc_address, osc_padded_len, parse_color_value, parse_float_value, parse_int_value,
    parse_meter_blob, parse_string_value, parse_switch_value, quantize_gain_step, read_be_u32,
};

pub fn fader_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/mix/fader"),
        FaderTarget::Aux(aux) => format!("/auxin/{aux:02}/mix/fader"),
        FaderTarget::Bus(bus) => format!("/bus/{bus:02}/mix/fader"),
        FaderTarget::FxRtn(fx) => format!("/fxrtn/{fx:02}/mix/fader"),
        FaderTarget::Mtx(mtx) => format!("/mtx/{mtx:02}/mix/fader"),
        FaderTarget::Dca(dca) => format!("/dca/{dca}/fader"),
        FaderTarget::Main => "/main/st/mix/fader".to_owned(),
    }
}

pub fn pan_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/mix/pan"),
        FaderTarget::Aux(aux) => format!("/auxin/{aux:02}/mix/pan"),
        FaderTarget::Bus(bus) => format!("/bus/{bus:02}/mix/pan"),
        FaderTarget::FxRtn(fx) => format!("/fxrtn/{fx:02}/mix/pan"),
        FaderTarget::Mtx(mtx) => format!("/mtx/{mtx:02}/mix/pan"),
        FaderTarget::Dca(_) => String::new(),
        FaderTarget::Main => "/main/st/mix/pan".to_owned(),
    }
}

pub fn gain_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/preamp/trim"),
        FaderTarget::Aux(aux) => format!("/auxin/{aux:02}/preamp/trim"),
        FaderTarget::Bus(_) => "/bus/01/preamp/trim".to_owned(),
        FaderTarget::FxRtn(_) => "/fxrtn/01/preamp/trim".to_owned(),
        FaderTarget::Mtx(_) | FaderTarget::Dca(_) => String::new(),
        FaderTarget::Main => "/main/st/preamp/trim".to_owned(),
    }
}

pub fn headamp_index_path(target: FaderTarget) -> String {
    let index = match target {
        FaderTarget::Channel(channel) => channel - 1,
        FaderTarget::Aux(aux) => 31 + aux,
        FaderTarget::Bus(_) => 255,
        FaderTarget::FxRtn(_) => 255,
        FaderTarget::Mtx(_) | FaderTarget::Dca(_) => 255,
        FaderTarget::Main => 255,
    };
    format!("/-ha/{index:02}/index")
}

pub fn headamp_gain_path(index: u8) -> String {
    format!("/headamp/{index:03}/gain")
}

pub fn headamp_index_from_gain_path(path: &str) -> Option<u8> {
    path.strip_prefix("/headamp/")
        .and_then(|rest| rest.strip_suffix("/gain"))
        .and_then(|index| index.parse::<u8>().ok())
}

pub fn send_level_path(target: FaderTarget, bus: u8) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/mix/{bus:02}/level"),
        FaderTarget::Aux(aux) => format!("/auxin/{aux:02}/mix/{bus:02}/level"),
        FaderTarget::Bus(bus_target) => format!("/bus/{bus_target:02}/mix/{bus:02}/level"),
        FaderTarget::FxRtn(fx) => format!("/fxrtn/{fx:02}/mix/{bus:02}/level"),
        FaderTarget::Mtx(_) | FaderTarget::Dca(_) => String::new(),
        FaderTarget::Main => format!("/main/st/mix/{bus:02}/level"),
    }
}

pub fn mute_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/mix/on"),
        FaderTarget::Aux(aux) => format!("/auxin/{aux:02}/mix/on"),
        FaderTarget::Bus(bus) => format!("/bus/{bus:02}/mix/on"),
        FaderTarget::FxRtn(fx) => format!("/fxrtn/{fx:02}/mix/on"),
        FaderTarget::Mtx(mtx) => format!("/mtx/{mtx:02}/mix/on"),
        FaderTarget::Dca(dca) => format!("/dca/{dca}/on"),
        FaderTarget::Main => "/main/st/mix/on".to_owned(),
    }
}

pub fn solo_path(target: FaderTarget) -> String {
    let id = match target {
        FaderTarget::Channel(channel) => channel,
        FaderTarget::Aux(aux) => 32 + aux,
        FaderTarget::FxRtn(fx) => 40 + fx,
        FaderTarget::Bus(bus) => 48 + bus,
        FaderTarget::Mtx(_) | FaderTarget::Dca(_) | FaderTarget::Main => 0,
    };
    format!("/-stat/solosw/{id:02}")
}

pub fn name_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/config/name"),
        FaderTarget::Aux(aux) => format!("/auxin/{aux:02}/config/name"),
        FaderTarget::Bus(bus) => format!("/bus/{bus:02}/config/name"),
        FaderTarget::FxRtn(fx) => format!("/fxrtn/{fx:02}/config/name"),
        FaderTarget::Mtx(mtx) => format!("/mtx/{mtx:02}/config/name"),
        FaderTarget::Dca(dca) => format!("/dca/{dca}/config/name"),
        FaderTarget::Main => "/main/st/config/name".to_owned(),
    }
}

pub fn color_path(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("/ch/{channel:02}/config/color"),
        FaderTarget::Aux(aux) => format!("/auxin/{aux:02}/config/color"),
        FaderTarget::Bus(bus) => format!("/bus/{bus:02}/config/color"),
        FaderTarget::FxRtn(fx) => format!("/fxrtn/{fx:02}/config/color"),
        FaderTarget::Mtx(mtx) => format!("/mtx/{mtx:02}/config/color"),
        FaderTarget::Dca(dca) => format!("/dca/{dca}/config/color"),
        FaderTarget::Main => "/main/st/config/color".to_owned(),
    }
}

pub fn parse_console_update(packet: &[u8]) -> Option<ConsoleUpdate> {
    if let Some((path, value)) = parse_gain_value(packet)
        && let Some(target) = target_from_channel_path(&path, GAIN_RESPONSE_SUFFIX)
    {
        return Some(ConsoleUpdate::Gain(StripGain {
            target,
            value: decode_trim_gain(value),
            source: crate::common::GainSource::Trim,
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
        .strip_prefix("/auxin/")
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        return index.parse::<u8>().ok().map(FaderTarget::Aux);
    }

    if let Some(index) = path
        .strip_prefix("/bus/")
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        return index.parse::<u8>().ok().map(FaderTarget::Bus);
    }

    if let Some(index) = path
        .strip_prefix("/fxrtn/")
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        return index.parse::<u8>().ok().map(FaderTarget::FxRtn);
    }

    if let Some(index) = path
        .strip_prefix("/mtx/")
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        return index.parse::<u8>().ok().map(FaderTarget::Mtx);
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

    if path == format!("/main/st{suffix}") {
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
        1..=32 => Some(FaderTarget::Channel(id)),
        33..=40 => Some(FaderTarget::Aux(id - 32)),
        41..=48 => Some(FaderTarget::FxRtn(id - 40)),
        49..=64 => Some(FaderTarget::Bus(id - 48)),
        _ => None,
    }
}

fn target_and_bus_from_send_path(path: &str) -> Option<(FaderTarget, u8)> {
    let (target, rest) = if let Some(rest) = path.strip_prefix("/ch/") {
        let (channel, rest) = rest.split_once('/')?;
        (FaderTarget::Channel(channel.parse::<u8>().ok()?), rest)
    } else if let Some(rest) = path.strip_prefix("/auxin/") {
        let (aux, rest) = rest.split_once('/')?;
        (FaderTarget::Aux(aux.parse::<u8>().ok()?), rest)
    } else if let Some(rest) = path.strip_prefix("/bus/") {
        let (bus, rest) = rest.split_once('/')?;
        (FaderTarget::Bus(bus.parse::<u8>().ok()?), rest)
    } else if let Some(rest) = path.strip_prefix("/fxrtn/") {
        let (fx, rest) = rest.split_once('/')?;
        (FaderTarget::FxRtn(fx.parse::<u8>().ok()?), rest)
    } else {
        let rest = path.strip_prefix("/main/st/")?;
        (FaderTarget::Main, rest)
    };

    let rest = rest.strip_prefix("mix/")?;
    let (bus, tail) = rest.split_once('/')?;
    if tail != "level" {
        return None;
    }

    let bus = bus.parse::<u8>().ok()?;
    if !(1..=16).contains(&bus) {
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

fn _parse_headamp_index_value(packet: &[u8]) -> Option<(String, i32)> {
    let (path, value) = parse_int_value(packet)?;
    if path.starts_with("/-ha/") && path.ends_with(HEADAMP_INDEX_RESPONSE_SUFFIX) {
        Some((path, value))
    } else {
        None
    }
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
    quantize_gain_step(raw.clamp(0.0, 1.0) * 72.0 - 12.0, -12.0, 0.1)
}

pub fn encode_headamp_gain(db: f32) -> f32 {
    ((quantize_gain_step(db, -12.0, 0.1) + 12.0) / 72.0).clamp(0.0, 1.0)
}

fn decode_trim_gain(raw: f32) -> f32 {
    quantize_gain_step(raw.clamp(0.0, 1.0) * 36.0 - 18.0, -18.0, 0.25)
}

fn _encode_trim_gain(db: f32) -> f32 {
    ((quantize_gain_step(db, -18.0, 0.25) + 18.0) / 36.0).clamp(0.0, 1.0)
}

pub fn parse_input_meter_packet(packet: &[u8]) -> Result<Vec<StripMeter>, ProbeError> {
    let floats = parse_meter_blob(packet, INPUT_METERS_REQUEST, INPUT_METERS_ALIAS)?;

    let mut strips = Vec::with_capacity(48);
    for index in 0..48 {
        let target = if index < 32 {
            FaderTarget::Channel((index + 1) as u8)
        } else if index < 40 {
            FaderTarget::Aux((index - 31) as u8)
        } else {
            FaderTarget::FxRtn((index - 39) as u8)
        };
        let start = index * 4;
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
    let floats = parse_meter_blob(packet, MAIN_METERS_REQUEST, MAIN_METERS_ALIAS)?;
    if floats.len() < 24 * 4 {
        return Err(ProbeError::Protocol(
            "main meter blob is shorter than expected".to_owned(),
        ));
    }

    let mut mains = [0.0f32; 16];
    for i in 0..16 {
        mains[i] = f32::from_le_bytes(floats[i * 4..i * 4 + 4].try_into().map_err(|_| {
            ProbeError::Protocol(format!("main meter {i} float slice size mismatch"))
        })?);
    }

    let mut matrices = [0.0f32; 6];
    for i in 0..6 {
        matrices[i] =
            f32::from_le_bytes(floats[(16 + i) * 4..(16 + i) * 4 + 4].try_into().map_err(
                |_| ProbeError::Protocol(format!("matrix meter {i} float slice size mismatch")),
            )?);
    }

    Ok(MainMeterLevels {
        mains,
        main_lr: [
            f32::from_le_bytes(floats[22 * 4..22 * 4 + 4].try_into().map_err(|_| {
                ProbeError::Protocol("main L meter float slice size mismatch".to_owned())
            })?),
            f32::from_le_bytes(floats[23 * 4..23 * 4 + 4].try_into().map_err(|_| {
                ProbeError::Protocol("main R meter float slice size mismatch".to_owned())
            })?),
        ],
        matrices,
    })
}

pub fn parse_rta_meter_packet(packet: &[u8]) -> Result<[f32; 100], ProbeError> {
    let path = osc_address(packet)
        .ok_or_else(|| ProbeError::Protocol("rta meter reply missing OSC address".to_owned()))?;
    if path != "/meters/15" {
        return Err(ProbeError::Protocol(format!(
            "unexpected rta meter reply path '{path}'"
        )));
    }

    let mut offset = osc_padded_len(packet).ok_or_else(|| {
        ProbeError::Protocol("rta meter reply has invalid OSC address".to_owned())
    })?;
    let type_tag_end = packet[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| ProbeError::Protocol("rta meter reply missing OSC type tag".to_owned()))?;
    let type_tag = std::str::from_utf8(&packet[offset..offset + type_tag_end])
        .map_err(|_| ProbeError::Protocol("rta meter reply type tag is not UTF-8".to_owned()))?;
    if type_tag != ",b" {
        return Err(ProbeError::Protocol(format!(
            "unexpected rta meter reply type tag '{type_tag}'"
        )));
    }
    offset += osc_padded_len(&packet[offset..])
        .ok_or_else(|| ProbeError::Protocol("rta meter reply has invalid type tag".to_owned()))?;

    let blob_len = read_be_u32(packet, offset)? as usize;
    offset += 4;
    let blob = packet.get(offset..offset + blob_len).ok_or_else(|| {
        ProbeError::Protocol("rta meter blob length exceeds packet size".to_owned())
    })?;

    let mut values = [-128.0f32; 100];
    let mut idx = 0;
    for chunk in blob.as_chunks::<4>().0 {
        if idx >= 100 {
            break;
        }
        let u = u32::from_le_bytes(*chunk);
        let low = (u & 0xFFFF) as i16;
        let high = ((u >> 16) & 0xFFFF) as i16;
        values[idx] = low as f32 / 256.0;
        idx += 1;
        if idx < 100 {
            values[idx] = high as f32 / 256.0;
            idx += 1;
        }
    }

    Ok(values)
}
