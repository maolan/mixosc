use crate::common::{
    COLOR_RESPONSE_SUFFIX, ConsoleUpdate, FADER_RESPONSE_SUFFIX, FaderTarget, GAIN_RESPONSE_SUFFIX,
    HEADAMP_GAIN_RESPONSE_SUFFIX, MUTE_RESPONSE_SUFFIX, MainMeterLevels, NAME_RESPONSE_SUFFIX,
    PAN_RESPONSE_SUFFIX, ProbeError, SOLO_RESPONSE_PREFIX, StripColor, StripFader, StripGain,
    StripMeter, StripMute, StripName, StripPan, StripSend, StripSolo, osc_address, osc_padded_len,
    parse_color_value, parse_float_value, parse_string_value, parse_switch_value,
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

const XR18_INPUT_METERS_PATH: &str = "/meters/1";
const XR18_MAIN_METERS_PATH: &str = "/meters/1";
const XR18_METER_COUNT: usize = 20;

fn xr18_meter_blob<'a>(packet: &'a [u8], expected_path: &str) -> Result<&'a [u8], ProbeError> {
    let path = osc_address(packet)
        .ok_or_else(|| ProbeError::Protocol("meter reply missing OSC address".to_owned()))?;
    if path != expected_path {
        return Err(ProbeError::Protocol(format!(
            "unexpected XR18 meter reply path '{path}'"
        )));
    }

    let type_tag_offset = osc_padded_len(packet).ok_or_else(|| {
        ProbeError::Protocol("XR18 meter reply missing type tag offset".to_owned())
    })?;
    if packet.get(type_tag_offset..type_tag_offset + 4) != Some(b",b\0\0") {
        return Err(ProbeError::Protocol(
            "XR18 meter reply is not a blob".to_owned(),
        ));
    }

    let size_offset = type_tag_offset + 4;
    let size_bytes: [u8; 4] = packet
        .get(size_offset..size_offset + 4)
        .ok_or_else(|| ProbeError::Protocol("XR18 meter blob size truncated".to_owned()))?
        .try_into()
        .map_err(|_| ProbeError::Protocol("XR18 meter blob size slice mismatch".to_owned()))?;
    let size = u32::from_be_bytes(size_bytes) as usize;

    packet
        .get(size_offset + 4..size_offset + 4 + size)
        .ok_or_else(|| ProbeError::Protocol("XR18 meter blob data truncated".to_owned()))
}

fn int16_db_to_linear(value: i16) -> f32 {
    let db = f32::from(value) / 256.0;
    10.0_f32.powf(db / 20.0)
}

pub fn parse_input_meter_packet(packet: &[u8]) -> Result<Vec<StripMeter>, ProbeError> {
    let blob = xr18_meter_blob(packet, XR18_INPUT_METERS_PATH)?;
    if blob.len() < XR18_METER_COUNT * 2 {
        return Err(ProbeError::Protocol(
            "XR18 input meter blob is shorter than expected".to_owned(),
        ));
    }

    let mut strips = Vec::with_capacity(16);
    for index in 0..16 {
        let bytes: [u8; 2] = blob[index * 2..index * 2 + 2]
            .try_into()
            .map_err(|_| ProbeError::Protocol("XR18 meter int16 slice size mismatch".to_owned()))?;
        strips.push(StripMeter {
            target: FaderTarget::Channel((index + 1) as u8),
            level_linear: int16_db_to_linear(i16::from_le_bytes(bytes)),
        });
    }
    Ok(strips)
}

pub fn parse_main_meter_packet(packet: &[u8]) -> Result<MainMeterLevels, ProbeError> {
    let blob = xr18_meter_blob(packet, XR18_MAIN_METERS_PATH)?;
    if blob.len() < XR18_METER_COUNT * 2 {
        return Err(ProbeError::Protocol(
            "XR18 main meter blob is shorter than expected".to_owned(),
        ));
    }

    let read = |index: usize| {
        let bytes: [u8; 2] = blob[index * 2..index * 2 + 2].try_into().map_err(|_| {
            ProbeError::Protocol(format!("XR18 main meter int16 {index} slice size mismatch"))
        })?;
        Ok(int16_db_to_linear(i16::from_le_bytes(bytes)))
    };

    Ok(MainMeterLevels {
        mains: [0.0f32; 16],
        main_lr: [read(18)?, read(19)?],
        matrices: [0.0f32; 6],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::osc_string;

    fn xr18_meters_packet(values: &[i16]) -> Vec<u8> {
        let mut packet = osc_string("/meters/1");
        packet.extend_from_slice(b",b\0\0");
        packet.extend_from_slice(&(values.len() as u32 * 2).to_be_bytes());
        for value in values {
            packet.extend_from_slice(&value.to_le_bytes());
        }
        while !packet.len().is_multiple_of(4) {
            packet.push(0);
        }
        packet
    }

    #[test]
    fn parses_xr18_input_meters() {
        let mut values = [i16::MIN; 20];
        values[0] = 0; // 0 dB (clipping)
        values[1] = -5120; // -20 dB
        values[15] = -2560; // -10 dB
        let packet = xr18_meters_packet(&values);

        let meters = parse_input_meter_packet(&packet).expect("should parse XR18 input meters");
        assert_eq!(meters.len(), 16);
        assert_eq!(meters[0].target, FaderTarget::Channel(1));
        assert!((meters[0].level_linear - 1.0).abs() < f32::EPSILON);
        assert_eq!(meters[1].target, FaderTarget::Channel(2));
        assert!((meters[1].level_linear - 0.1).abs() < 0.001);
        assert_eq!(meters[15].target, FaderTarget::Channel(16));
        assert!((meters[15].level_linear - 0.316_227_76).abs() < 0.001);
    }

    #[test]
    fn parses_xr18_main_meters() {
        let mut values = [i16::MIN; 20];
        values[18] = -2560; // main L -10 dB
        values[19] = -5120; // main R -20 dB
        let packet = xr18_meters_packet(&values);

        let levels = parse_main_meter_packet(&packet).expect("should parse XR18 main meters");
        assert!((levels.main_lr[0] - 0.316_227_76).abs() < 0.001);
        assert!((levels.main_lr[1] - 0.1).abs() < 0.001);
    }

    #[test]
    fn rejects_wrong_xr18_meter_path() {
        let packet = osc_string("/meters/2");
        assert!(parse_input_meter_packet(&packet).is_err());
    }
}
