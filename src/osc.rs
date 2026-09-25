use crate::net::ProbeError;

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

pub(crate) fn osc_query(address: &str) -> Vec<u8> {
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

pub(crate) fn parse_fader_value(packet: &[u8]) -> Option<(String, f32)> {
    if let Some(result) = parse_float_value(packet, FADER_RESPONSE_SUFFIX) {
        return Some(result);
    }
    let (path, value) = parse_float_value(packet, "/fader")?;
    if !path.starts_with("/dca/") {
        return None;
    }
    Some((path, value))
}

pub(crate) fn parse_pan_value(packet: &[u8]) -> Option<(String, f32)> {
    parse_float_value(packet, PAN_RESPONSE_SUFFIX)
}

pub(crate) fn parse_gain_value(packet: &[u8]) -> Option<(String, f32)> {
    parse_float_value(packet, GAIN_RESPONSE_SUFFIX)
}

pub(crate) fn parse_headamp_gain_value(packet: &[u8]) -> Option<(String, f32)> {
    let (path, value) = parse_float_value(packet, HEADAMP_GAIN_RESPONSE_SUFFIX)?;
    path.starts_with("/headamp/").then_some((path, value))
}

pub(crate) fn parse_headamp_index_value(packet: &[u8]) -> Option<(String, i32)> {
    let (path, value) = parse_int_value(packet)?;
    if path.starts_with("/-ha/") && path.ends_with(HEADAMP_INDEX_RESPONSE_SUFFIX) {
        Some((path, value))
    } else {
        None
    }
}

pub(crate) fn parse_send_value(packet: &[u8]) -> Option<(String, f32)> {
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

pub(crate) fn is_dca_mute_path(path: &str) -> bool {
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

pub(crate) fn osc_strings(packet: &[u8]) -> Vec<String> {
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
}
