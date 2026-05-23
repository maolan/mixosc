// Auto-generated from X32 OSC specification.
// Do not edit manually.

use crate::x32::{
    osc_address, osc_float_message, osc_int_message, osc_padded_len, osc_string, osc_string_message,
};

/// A typed OSC value.
#[derive(Debug, Clone, PartialEq)]
pub enum OscValue {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
}

impl OscValue {
    /// Create a float value.
    pub fn float(v: f32) -> Self {
        Self::Float(v)
    }

    /// Create an int value.
    pub fn int(v: i32) -> Self {
        Self::Int(v)
    }

    /// Create a string value.
    pub fn string(v: impl Into<String>) -> Self {
        Self::String(v.into())
    }

    /// Create a bool value.
    pub fn bool(v: bool) -> Self {
        Self::Bool(v)
    }
}

/// Build an OSC query packet (get value).
pub fn build_get(path: &str) -> Vec<u8> {
    osc_string(path)
}

/// Build an OSC set packet with a typed value.
pub fn build_set(path: &str, value: OscValue) -> Vec<u8> {
    match value {
        OscValue::Float(v) => osc_float_message(path, v),
        OscValue::Int(v) => osc_int_message(path, v),
        OscValue::String(v) => osc_string_message(path, &v),
        OscValue::Bool(v) => osc_int_message(path, i32::from(v)),
    }
}

/// Build an OSC set packet with multiple typed values.
pub fn build_set_multi(path: &str, values: &[OscValue]) -> Vec<u8> {
    let mut type_tag = String::from(",");
    let mut payload = Vec::new();
    for value in values {
        match value {
            OscValue::Float(v) => {
                type_tag.push('f');
                payload.extend_from_slice(&v.to_bits().to_be_bytes());
            }
            OscValue::Int(v) => {
                type_tag.push('i');
                payload.extend_from_slice(&v.to_be_bytes());
            }
            OscValue::String(v) => {
                type_tag.push('s');
                payload.extend_from_slice(&osc_string(v));
            }
            OscValue::Bool(v) => {
                type_tag.push('i');
                payload.extend_from_slice(&i32::from(*v).to_be_bytes());
            }
        }
    }
    let mut packet = osc_string(path);
    packet.extend_from_slice(&osc_string(&type_tag));
    packet.extend_from_slice(&payload);
    packet
}

/// Parse an OSC response packet into (path, value).
pub fn parse_osc_value(packet: &[u8]) -> Option<(String, OscValue)> {
    let path = osc_address(packet)?;
    let mut offset = osc_padded_len(packet)?;
    let type_tag_end = packet.get(offset..)?.iter().position(|byte| *byte == 0)?;
    let type_tag = std::str::from_utf8(packet.get(offset..offset + type_tag_end)?).ok()?;
    let type_tag_len = osc_padded_len(packet.get(offset..)?)?;
    offset += type_tag_len;

    match type_tag {
        ",f" => {
            let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
            Some((
                path.to_owned(),
                OscValue::Float(f32::from_bits(u32::from_be_bytes(value_bytes))),
            ))
        }
        ",i" => {
            let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
            Some((
                path.to_owned(),
                OscValue::Int(i32::from_be_bytes(value_bytes)),
            ))
        }
        ",s" => {
            let value_bytes = packet.get(offset..)?;
            let value_end = value_bytes.iter().position(|byte| *byte == 0)?;
            let value = std::str::from_utf8(&value_bytes[..value_end]).ok()?;
            Some((path.to_owned(), OscValue::String(value.to_owned())))
        }
        _ => None,
    }
}

/// Path builder functions for all X32 OSC parameters.
pub mod path {
    // -stat
    /// /-stat/solosw/N (unknown)
    pub fn _stat_solosw(n1: u8) -> String {
        format!("/-stat/solosw/{n1:02}")
    }

    /// /-stat/talk/A (unknown)
    pub fn _stat_talk_a() -> String {
        String::from("/-stat/talk/A")
    }

    /// /-stat/talk/B (unknown)
    pub fn _stat_talk_b() -> String {
        String::from("/-stat/talk/B")
    }

    // auxin
    /// /auxin/N/config/color (int)
    pub fn auxin_config_color(n1: u8) -> String {
        format!("/auxin/{n1:02}/config/color")
    }

    /// /auxin/N/config/icon (int)
    pub fn auxin_config_icon(n1: u8) -> String {
        format!("/auxin/{n1:02}/config/icon")
    }

    /// /auxin/N/config/name (string)
    pub fn auxin_config_name(n1: u8) -> String {
        format!("/auxin/{n1:02}/config/name")
    }

    /// /auxin/N/config/source (int)
    pub fn auxin_config_source(n1: u8) -> String {
        format!("/auxin/{n1:02}/config/source")
    }

    /// /auxin/N/eq/N/f (float)
    pub fn auxin_eq_f(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/f")
    }

    /// /auxin/N/eq/N/g (float)
    pub fn auxin_eq_g(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/g")
    }

    /// /auxin/N/eq/N/on (bool)
    pub fn auxin_eq_on(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/on")
    }

    /// /auxin/N/eq/N/q (float)
    pub fn auxin_eq_q(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/q")
    }

    /// /auxin/N/eq/N/type (int)
    pub fn auxin_eq_type(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/type")
    }

    /// /auxin/N/eq/on (bool)
    pub fn auxin_eq_on_2(n1: u8) -> String {
        format!("/auxin/{n1:02}/eq/on")
    }

    /// /auxin/N/grp/dca (unknown)
    pub fn auxin_grp_dca(n1: u8) -> String {
        format!("/auxin/{n1:02}/grp/dca")
    }

    /// /auxin/N/grp/mute (unknown)
    pub fn auxin_grp_mute(n1: u8) -> String {
        format!("/auxin/{n1:02}/grp/mute")
    }

    /// /auxin/N/mix/N/mevel (unknown)
    pub fn auxin_mix_mevel(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/mevel")
    }

    /// /auxin/N/mix/N/on (bool)
    pub fn auxin_mix_on(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/on")
    }

    /// /auxin/N/mix/N/pan (float)
    pub fn auxin_mix_pan(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/pan")
    }

    /// /auxin/N/mix/N/panFollow (unknown)
    pub fn auxin_mix_pan_follow(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/panFollow")
    }

    /// /auxin/N/mix/N/type (int)
    pub fn auxin_mix_type(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/type")
    }

    /// /auxin/N/mix/fader (float)
    pub fn auxin_mix_fader(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/fader")
    }

    /// /auxin/N/mix/mlevel (unknown)
    pub fn auxin_mix_mlevel(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/mlevel")
    }

    /// /auxin/N/mix/mono (bool)
    pub fn auxin_mix_mono(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/mono")
    }

    /// /auxin/N/mix/on (bool)
    pub fn auxin_mix_on_2(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/on")
    }

    /// /auxin/N/mix/pan (float)
    pub fn auxin_mix_pan_2(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/pan")
    }

    /// /auxin/N/mix/st (bool)
    pub fn auxin_mix_st(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/st")
    }

    /// /auxin/N/preamp/invert (bool)
    pub fn auxin_preamp_invert(n1: u8) -> String {
        format!("/auxin/{n1:02}/preamp/invert")
    }

    /// /auxin/N/preamp/trim (float)
    pub fn auxin_preamp_trim(n1: u8) -> String {
        format!("/auxin/{n1:02}/preamp/trim")
    }

    // bus
    /// /bus/N/config/color (int)
    pub fn bus_config_color(n1: u8) -> String {
        format!("/bus/{n1:02}/config/color")
    }

    /// /bus/N/config/icon (int)
    pub fn bus_config_icon(n1: u8) -> String {
        format!("/bus/{n1:02}/config/icon")
    }

    /// /bus/N/config/name (string)
    pub fn bus_config_name(n1: u8) -> String {
        format!("/bus/{n1:02}/config/name")
    }

    /// /bus/N/dyn/attack (float)
    pub fn bus_dyn_attack(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/attack")
    }

    /// /bus/N/dyn/auto (bool)
    pub fn bus_dyn_auto(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/auto")
    }

    /// /bus/N/dyn/det (int)
    pub fn bus_dyn_det(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/det")
    }

    /// /bus/N/dyn/env (int)
    pub fn bus_dyn_env(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/env")
    }

    /// /bus/N/dyn/filter/f (float)
    pub fn bus_dyn_filter_f(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/filter/f")
    }

    /// /bus/N/dyn/filter/on (bool)
    pub fn bus_dyn_filter_on(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/filter/on")
    }

    /// /bus/N/dyn/filter/type (int)
    pub fn bus_dyn_filter_type(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/filter/type")
    }

    /// /bus/N/dyn/hold (float)
    pub fn bus_dyn_hold(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/hold")
    }

    /// /bus/N/dyn/keysrc (int)
    pub fn bus_dyn_keysrc(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/keysrc")
    }

    /// /bus/N/dyn/knee (float)
    pub fn bus_dyn_knee(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/knee")
    }

    /// /bus/N/dyn/mgain (float)
    pub fn bus_dyn_mgain(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/mgain")
    }

    /// /bus/N/dyn/mix (float)
    pub fn bus_dyn_mix(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/mix")
    }

    /// /bus/N/dyn/mode (int)
    pub fn bus_dyn_mode(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/mode")
    }

    /// /bus/N/dyn/on (bool)
    pub fn bus_dyn_on(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/on")
    }

    /// /bus/N/dyn/pos (int)
    pub fn bus_dyn_pos(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/pos")
    }

    /// /bus/N/dyn/ratio (unknown)
    pub fn bus_dyn_ratio(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/ratio")
    }

    /// /bus/N/dyn/release (float)
    pub fn bus_dyn_release(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/release")
    }

    /// /bus/N/dyn/thr (float)
    pub fn bus_dyn_thr(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/thr")
    }

    /// /bus/N/eq/N/f (float)
    pub fn bus_eq_f(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/f")
    }

    /// /bus/N/eq/N/g (float)
    pub fn bus_eq_g(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/g")
    }

    /// /bus/N/eq/N/on (bool)
    pub fn bus_eq_on(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/on")
    }

    /// /bus/N/eq/N/q (float)
    pub fn bus_eq_q(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/q")
    }

    /// /bus/N/eq/N/type (int)
    pub fn bus_eq_type(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/type")
    }

    /// /bus/N/eq/on (bool)
    pub fn bus_eq_on_2(n1: u8) -> String {
        format!("/bus/{n1:02}/eq/on")
    }

    /// /bus/N/grp/dca (unknown)
    pub fn bus_grp_dca(n1: u8) -> String {
        format!("/bus/{n1:02}/grp/dca")
    }

    /// /bus/N/grp/mute (unknown)
    pub fn bus_grp_mute(n1: u8) -> String {
        format!("/bus/{n1:02}/grp/mute")
    }

    /// /bus/N/insert/on (bool)
    pub fn bus_insert_on(n1: u8) -> String {
        format!("/bus/{n1:02}/insert/on")
    }

    /// /bus/N/insert/pos (int)
    pub fn bus_insert_pos(n1: u8) -> String {
        format!("/bus/{n1:02}/insert/pos")
    }

    /// /bus/N/insert/sel (int)
    pub fn bus_insert_sel(n1: u8) -> String {
        format!("/bus/{n1:02}/insert/sel")
    }

    /// /bus/N/mix/N/level (float)
    pub fn bus_mix_level(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/level")
    }

    /// /bus/N/mix/N/on (bool)
    pub fn bus_mix_on(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/on")
    }

    /// /bus/N/mix/N/pan (float)
    pub fn bus_mix_pan(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/pan")
    }

    /// /bus/N/mix/N/panFollow (unknown)
    pub fn bus_mix_pan_follow(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/panFollow")
    }

    /// /bus/N/mix/N/type (int)
    pub fn bus_mix_type(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/type")
    }

    /// /bus/N/mix/fader (float)
    pub fn bus_mix_fader(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/fader")
    }

    /// /bus/N/mix/mlevel (unknown)
    pub fn bus_mix_mlevel(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/mlevel")
    }

    /// /bus/N/mix/mono (bool)
    pub fn bus_mix_mono(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/mono")
    }

    /// /bus/N/mix/on (bool)
    pub fn bus_mix_on_2(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/on")
    }

    /// /bus/N/mix/pan (float)
    pub fn bus_mix_pan_2(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/pan")
    }

    /// /bus/N/mix/st (bool)
    pub fn bus_mix_st(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/st")
    }

    // ch
    /// /ch/N/automix/group (int)
    pub fn ch_automix_group(n1: u8) -> String {
        format!("/ch/{n1:02}/automix/group")
    }

    /// /ch/N/automix/weight (float)
    pub fn ch_automix_weight(n1: u8) -> String {
        format!("/ch/{n1:02}/automix/weight")
    }

    /// /ch/N/config/color (enum)
    pub fn ch_config_color(n1: u8) -> String {
        format!("/ch/{n1:02}/config/color")
    }

    /// /ch/N/config/icon (int)
    pub fn ch_config_icon(n1: u8) -> String {
        format!("/ch/{n1:02}/config/icon")
    }

    /// /ch/N/config/name (string)
    pub fn ch_config_name(n1: u8) -> String {
        format!("/ch/{n1:02}/config/name")
    }

    /// /ch/N/config/source (int)
    pub fn ch_config_source(n1: u8) -> String {
        format!("/ch/{n1:02}/config/source")
    }

    /// /ch/N/delay/on (bool)
    pub fn ch_delay_on(n1: u8) -> String {
        format!("/ch/{n1:02}/delay/on")
    }

    /// /ch/N/delay/time (float)
    pub fn ch_delay_time(n1: u8) -> String {
        format!("/ch/{n1:02}/delay/time")
    }

    /// /ch/N/dyn/attack (float)
    pub fn ch_dyn_attack(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/attack")
    }

    /// /ch/N/dyn/auto (bool)
    pub fn ch_dyn_auto(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/auto")
    }

    /// /ch/N/dyn/det (int)
    pub fn ch_dyn_det(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/det")
    }

    /// /ch/N/dyn/env (int)
    pub fn ch_dyn_env(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/env")
    }

    /// /ch/N/dyn/filter/f (float)
    pub fn ch_dyn_filter_f(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/filter/f")
    }

    /// /ch/N/dyn/filter/on (bool)
    pub fn ch_dyn_filter_on(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/filter/on")
    }

    /// /ch/N/dyn/filter/type (int)
    pub fn ch_dyn_filter_type(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/filter/type")
    }

    /// /ch/N/dyn/hold (float)
    pub fn ch_dyn_hold(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/hold")
    }

    /// /ch/N/dyn/keysrc (int)
    pub fn ch_dyn_keysrc(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/keysrc")
    }

    /// /ch/N/dyn/knee (float)
    pub fn ch_dyn_knee(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/knee")
    }

    /// /ch/N/dyn/mgain (float)
    pub fn ch_dyn_mgain(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/mgain")
    }

    /// /ch/N/dyn/mix (float)
    pub fn ch_dyn_mix(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/mix")
    }

    /// /ch/N/dyn/mode (int)
    pub fn ch_dyn_mode(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/mode")
    }

    /// /ch/N/dyn/on (bool)
    pub fn ch_dyn_on(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/on")
    }

    /// /ch/N/dyn/pos (int)
    pub fn ch_dyn_pos(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/pos")
    }

    /// /ch/N/dyn/ratio (float)
    pub fn ch_dyn_ratio(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/ratio")
    }

    /// /ch/N/dyn/release (float)
    pub fn ch_dyn_release(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/release")
    }

    /// /ch/N/dyn/thr (float)
    pub fn ch_dyn_thr(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/thr")
    }

    /// /ch/N/eq/N/f (float)
    pub fn ch_eq_f(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/f")
    }

    /// /ch/N/eq/N/g (float)
    pub fn ch_eq_g(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/g")
    }

    /// /ch/N/eq/N/on (bool)
    pub fn ch_eq_on(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/on")
    }

    /// /ch/N/eq/N/q (float)
    pub fn ch_eq_q(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/q")
    }

    /// /ch/N/eq/N/type (enum)
    pub fn ch_eq_type(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/type")
    }

    /// /ch/N/eq/on (bool)
    pub fn ch_eq_on_2(n1: u8) -> String {
        format!("/ch/{n1:02}/eq/on")
    }

    /// /ch/N/gate/attack (float)
    pub fn ch_gate_attack(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/attack")
    }

    /// /ch/N/gate/filter/f (float)
    pub fn ch_gate_filter_f(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/filter/f")
    }

    /// /ch/N/gate/filter/on (bool)
    pub fn ch_gate_filter_on(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/filter/on")
    }

    /// /ch/N/gate/filter/type (int)
    pub fn ch_gate_filter_type(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/filter/type")
    }

    /// /ch/N/gate/hold (float)
    pub fn ch_gate_hold(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/hold")
    }

    /// /ch/N/gate/keysrc (int)
    pub fn ch_gate_keysrc(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/keysrc")
    }

    /// /ch/N/gate/mode (enum)
    pub fn ch_gate_mode(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/mode")
    }

    /// /ch/N/gate/on (bool)
    pub fn ch_gate_on(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/on")
    }

    /// /ch/N/gate/range (float)
    pub fn ch_gate_range(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/range")
    }

    /// /ch/N/gate/release (float)
    pub fn ch_gate_release(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/release")
    }

    /// /ch/N/gate/thr (float)
    pub fn ch_gate_thr(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/thr")
    }

    /// /ch/N/grp/dca (unknown)
    pub fn ch_grp_dca(n1: u8) -> String {
        format!("/ch/{n1:02}/grp/dca")
    }

    /// /ch/N/grp/mute (unknown)
    pub fn ch_grp_mute(n1: u8) -> String {
        format!("/ch/{n1:02}/grp/mute")
    }

    /// /ch/N/insert/on (bool)
    pub fn ch_insert_on(n1: u8) -> String {
        format!("/ch/{n1:02}/insert/on")
    }

    /// /ch/N/insert/pos (int)
    pub fn ch_insert_pos(n1: u8) -> String {
        format!("/ch/{n1:02}/insert/pos")
    }

    /// /ch/N/insert/sel (int)
    pub fn ch_insert_sel(n1: u8) -> String {
        format!("/ch/{n1:02}/insert/sel")
    }

    /// /ch/N/mix/N/level (float)
    pub fn ch_mix_level(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/mix/{n2:02}/level")
    }

    /// /ch/N/mix/N/pan (float)
    pub fn ch_mix_pan(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/mix/{n2:02}/pan")
    }

    /// /ch/N/mix/N/panFollow (unknown)
    pub fn ch_mix_pan_follow(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/mix/{n2:02}/panFollow")
    }

    /// /ch/N/mix/N/type (int)
    pub fn ch_mix_type(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/mix/{n2:02}/type")
    }

    /// /ch/N/mix/fader (float)
    pub fn ch_mix_fader(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/fader")
    }

    /// /ch/N/mix/mlevel (unknown)
    pub fn ch_mix_mlevel(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/mlevel")
    }

    /// /ch/N/mix/mono (bool)
    pub fn ch_mix_mono(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/mono")
    }

    /// /ch/N/mix/on (bool)
    pub fn ch_mix_on(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/on")
    }

    /// /ch/N/mix/pan (float)
    pub fn ch_mix_pan_2(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/pan")
    }

    /// /ch/N/mix/st (bool)
    pub fn ch_mix_st(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/st")
    }

    /// /ch/N/pream/hpon (bool)
    pub fn ch_pream_hpon(n1: u8) -> String {
        format!("/ch/{n1:02}/pream/hpon")
    }

    /// /ch/N/preamp/hpf (float)
    pub fn ch_preamp_hpf(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/hpf")
    }

    /// /ch/N/preamp/hpon (bool)
    pub fn ch_preamp_hpon(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/hpon")
    }

    /// /ch/N/preamp/hpslope (enum)
    pub fn ch_preamp_hpslope(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/hpslope")
    }

    /// /ch/N/preamp/invert (bool)
    pub fn ch_preamp_invert(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/invert")
    }

    /// /ch/N/preamp/trim (float)
    pub fn ch_preamp_trim(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/trim")
    }

    // config
    /// /config/auxlink/N (unknown)
    pub fn config_auxlink(n1: u8) -> String {
        format!("/config/auxlink/{n1:02}")
    }

    /// /config/buslink/N (unknown)
    pub fn config_buslink(n1: u8) -> String {
        format!("/config/buslink/{n1:02}")
    }

    /// /config/chlink/N (unknown)
    pub fn config_chlink(n1: u8) -> String {
        format!("/config/chlink/{n1:02}")
    }

    /// /config/dp48/broadcast (unknown)
    pub fn config_dp48_broadcast() -> String {
        String::from("/config/dp48/broadcast")
    }

    /// /config/dp48/scope (unknown)
    pub fn config_dp48_scope() -> String {
        String::from("/config/dp48/scope")
    }

    /// /config/fxlink/N (unknown)
    pub fn config_fxlink(n1: u8) -> String {
        format!("/config/fxlink/{n1:02}")
    }

    /// /config/linkcfg/dyn (unknown)
    pub fn config_linkcfg_dyn() -> String {
        String::from("/config/linkcfg/dyn")
    }

    /// /config/linkcfg/eq (unknown)
    pub fn config_linkcfg_eq() -> String {
        String::from("/config/linkcfg/eq")
    }

    /// /config/linkcfg/fdrmute (unknown)
    pub fn config_linkcfg_fdrmute() -> String {
        String::from("/config/linkcfg/fdrmute")
    }

    /// /config/linkcfg/hadly (unknown)
    pub fn config_linkcfg_hadly() -> String {
        String::from("/config/linkcfg/hadly")
    }

    /// /config/mono/link (unknown)
    pub fn config_mono_link() -> String {
        String::from("/config/mono/link")
    }

    /// /config/mono/mode (int)
    pub fn config_mono_mode() -> String {
        String::from("/config/mono/mode")
    }

    /// /config/mtxlink/N (unknown)
    pub fn config_mtxlink(n1: u8) -> String {
        format!("/config/mtxlink/{n1:02}")
    }

    /// /config/mute/N (unknown)
    pub fn config_mute(n1: u8) -> String {
        format!("/config/mute/{n1:02}")
    }

    /// /config/osc/dest (unknown)
    pub fn config_osc_dest() -> String {
        String::from("/config/osc/dest")
    }

    /// /config/osc/f (float)
    pub fn config_osc_f() -> String {
        String::from("/config/osc/f")
    }

    /// /config/osc/fsel (unknown)
    pub fn config_osc_fsel() -> String {
        String::from("/config/osc/fsel")
    }

    /// /config/osc/level (float)
    pub fn config_osc_level() -> String {
        String::from("/config/osc/level")
    }

    /// /config/osc/type (int)
    pub fn config_osc_type() -> String {
        String::from("/config/osc/type")
    }

    /// /config/routing/AES50A/N (unknown)
    pub fn config_routing_aes50_a(n1: u8) -> String {
        format!("/config/routing/AES50A/{n1:02}")
    }

    /// /config/routing/AES50B/N (unknown)
    pub fn config_routing_aes50_b(n1: u8) -> String {
        format!("/config/routing/AES50B/{n1:02}")
    }

    /// /config/routing/CARD/N (unknown)
    pub fn config_routing_card(n1: u8) -> String {
        format!("/config/routing/CARD/{n1:02}")
    }

    /// /config/routing/IN/AUX (unknown)
    pub fn config_routing_in_aux() -> String {
        String::from("/config/routing/IN/AUX")
    }

    /// /config/routing/IN/N (unknown)
    pub fn config_routing_in(n1: u8) -> String {
        format!("/config/routing/IN/{n1:02}")
    }

    /// /config/routing/OUT/N (unknown)
    pub fn config_routing_out(n1: u8) -> String {
        format!("/config/routing/OUT/{n1:02}")
    }

    /// /config/routing/PLAY/AUX (unknown)
    pub fn config_routing_play_aux() -> String {
        String::from("/config/routing/PLAY/AUX")
    }

    /// /config/routing/PLAY/N (unknown)
    pub fn config_routing_play(n1: u8) -> String {
        format!("/config/routing/PLAY/{n1:02}")
    }

    /// /config/routing/routswitch (unknown)
    pub fn config_routing_routswitch() -> String {
        String::from("/config/routing/routswitch")
    }

    /// /config/solo/busmode (unknown)
    pub fn config_solo_busmode() -> String {
        String::from("/config/solo/busmode")
    }

    /// /config/solo/chmode (unknown)
    pub fn config_solo_chmode() -> String {
        String::from("/config/solo/chmode")
    }

    /// /config/solo/dcamode (unknown)
    pub fn config_solo_dcamode() -> String {
        String::from("/config/solo/dcamode")
    }

    /// /config/solo/delay (int)
    pub fn config_solo_delay() -> String {
        String::from("/config/solo/delay")
    }

    /// /config/solo/delaytime (unknown)
    pub fn config_solo_delaytime() -> String {
        String::from("/config/solo/delaytime")
    }

    /// /config/solo/dim (unknown)
    pub fn config_solo_dim() -> String {
        String::from("/config/solo/dim")
    }

    /// /config/solo/dimatt (unknown)
    pub fn config_solo_dimatt() -> String {
        String::from("/config/solo/dimatt")
    }

    /// /config/solo/dimpfl (unknown)
    pub fn config_solo_dimpfl() -> String {
        String::from("/config/solo/dimpfl")
    }

    /// /config/solo/exclusive (unknown)
    pub fn config_solo_exclusive() -> String {
        String::from("/config/solo/exclusive")
    }

    /// /config/solo/followsel (unknown)
    pub fn config_solo_followsel() -> String {
        String::from("/config/solo/followsel")
    }

    /// /config/solo/followsolo (unknown)
    pub fn config_solo_followsolo() -> String {
        String::from("/config/solo/followsolo")
    }

    /// /config/solo/level (float)
    pub fn config_solo_level() -> String {
        String::from("/config/solo/level")
    }

    /// /config/solo/masterctrl (unknown)
    pub fn config_solo_masterctrl() -> String {
        String::from("/config/solo/masterctrl")
    }

    /// /config/solo/mono (bool)
    pub fn config_solo_mono() -> String {
        String::from("/config/solo/mono")
    }

    /// /config/solo/mute (unknown)
    pub fn config_solo_mute() -> String {
        String::from("/config/solo/mute")
    }

    /// /config/solo/source (int)
    pub fn config_solo_source() -> String {
        String::from("/config/solo/source")
    }

    /// /config/solo/sourcetrim (unknown)
    pub fn config_solo_sourcetrim() -> String {
        String::from("/config/solo/sourcetrim")
    }

    /// /config/talk/A/destmap (unknown)
    pub fn config_talk_a_destmap() -> String {
        String::from("/config/talk/A/destmap")
    }

    /// /config/talk/A/dim (unknown)
    pub fn config_talk_a_dim() -> String {
        String::from("/config/talk/A/dim")
    }

    /// /config/talk/A/latch (unknown)
    pub fn config_talk_a_latch() -> String {
        String::from("/config/talk/A/latch")
    }

    /// /config/talk/A/level (float)
    pub fn config_talk_a_level() -> String {
        String::from("/config/talk/A/level")
    }

    /// /config/talk/B/destmap (unknown)
    pub fn config_talk_b_destmap() -> String {
        String::from("/config/talk/B/destmap")
    }

    /// /config/talk/B/dim (unknown)
    pub fn config_talk_b_dim() -> String {
        String::from("/config/talk/B/dim")
    }

    /// /config/talk/B/latch (unknown)
    pub fn config_talk_b_latch() -> String {
        String::from("/config/talk/B/latch")
    }

    /// /config/talk/B/level (float)
    pub fn config_talk_b_level() -> String {
        String::from("/config/talk/B/level")
    }

    /// /config/talk/enable (unknown)
    pub fn config_talk_enable() -> String {
        String::from("/config/talk/enable")
    }

    /// /config/talk/source (int)
    pub fn config_talk_source() -> String {
        String::from("/config/talk/source")
    }

    /// /config/tape/autoplay (unknown)
    pub fn config_tape_autoplay() -> String {
        String::from("/config/tape/autoplay")
    }

    /// /config/tape/gainL (unknown)
    pub fn config_tape_gain_l() -> String {
        String::from("/config/tape/gainL")
    }

    /// /config/tape/gainR (unknown)
    pub fn config_tape_gain_r() -> String {
        String::from("/config/tape/gainR")
    }

    /// /config/userctrl/A/btn/N (unknown)
    pub fn config_userctrl_a_btn(n1: u8) -> String {
        format!("/config/userctrl/A/btn/{n1:02}")
    }

    /// /config/userctrl/A/color (int)
    pub fn config_userctrl_a_color() -> String {
        String::from("/config/userctrl/A/color")
    }

    /// /config/userctrl/A/enc/N (unknown)
    pub fn config_userctrl_a_enc(n1: u8) -> String {
        format!("/config/userctrl/A/enc/{n1:02}")
    }

    /// /config/userctrl/B/btn/N (unknown)
    pub fn config_userctrl_b_btn(n1: u8) -> String {
        format!("/config/userctrl/B/btn/{n1:02}")
    }

    /// /config/userctrl/B/color (int)
    pub fn config_userctrl_b_color() -> String {
        String::from("/config/userctrl/B/color")
    }

    /// /config/userctrl/B/enc/N (unknown)
    pub fn config_userctrl_b_enc(n1: u8) -> String {
        format!("/config/userctrl/B/enc/{n1:02}")
    }

    /// /config/userctrl/C/btn/N (unknown)
    pub fn config_userctrl_c_btn(n1: u8) -> String {
        format!("/config/userctrl/C/btn/{n1:02}")
    }

    /// /config/userctrl/C/color (int)
    pub fn config_userctrl_c_color() -> String {
        String::from("/config/userctrl/C/color")
    }

    /// /config/userctrl/C/enc/N (unknown)
    pub fn config_userctrl_c_enc(n1: u8) -> String {
        format!("/config/userctrl/C/enc/{n1:02}")
    }

    // dca
    /// /dca/N/config/color (int)
    pub fn dca_config_color(n1: u8) -> String {
        format!("/dca/{n1:02}/config/color")
    }

    /// /dca/N/config/icon (int)
    pub fn dca_config_icon(n1: u8) -> String {
        format!("/dca/{n1:02}/config/icon")
    }

    /// /dca/N/config/name (string)
    pub fn dca_config_name(n1: u8) -> String {
        format!("/dca/{n1:02}/config/name")
    }

    /// /dca/N/fader (float)
    pub fn dca_fader(n1: u8) -> String {
        format!("/dca/{n1:02}/fader")
    }

    /// /dca/N/mix/fader (float)
    pub fn dca_mix_fader(n1: u8) -> String {
        format!("/dca/{n1:02}/mix/fader")
    }

    /// /dca/N/mix/on (bool)
    pub fn dca_mix_on(n1: u8) -> String {
        format!("/dca/{n1:02}/mix/on")
    }

    /// /dca/N/on (bool)
    pub fn dca_on(n1: u8) -> String {
        format!("/dca/{n1:02}/on")
    }

    // fx
    /// /fx/N/par/N (float)
    pub fn fx_par(n1: u8, n2: u8) -> String {
        format!("/fx/{n1:02}/par/{n2:02}")
    }

    /// /fx/N/source/l (unknown)
    pub fn fx_source_l(n1: u8) -> String {
        format!("/fx/{n1:02}/source/l")
    }

    /// /fx/N/source/r (unknown)
    pub fn fx_source_r(n1: u8) -> String {
        format!("/fx/{n1:02}/source/r")
    }

    /// /fx/N/type (enum)
    pub fn fx_type(n1: u8) -> String {
        format!("/fx/{n1:02}/type")
    }

    // fxrtn
    /// /fxrtn/N/config/color (int)
    pub fn fxrtn_config_color(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/config/color")
    }

    /// /fxrtn/N/config/icon (int)
    pub fn fxrtn_config_icon(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/config/icon")
    }

    /// /fxrtn/N/config/name (string)
    pub fn fxrtn_config_name(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/config/name")
    }

    /// /fxrtn/N/eq/N/f (float)
    pub fn fxrtn_eq_f(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/f")
    }

    /// /fxrtn/N/eq/N/g (float)
    pub fn fxrtn_eq_g(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/g")
    }

    /// /fxrtn/N/eq/N/on (bool)
    pub fn fxrtn_eq_on(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/on")
    }

    /// /fxrtn/N/eq/N/q (float)
    pub fn fxrtn_eq_q(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/q")
    }

    /// /fxrtn/N/eq/N/type (int)
    pub fn fxrtn_eq_type(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/type")
    }

    /// /fxrtn/N/eq/on (bool)
    pub fn fxrtn_eq_on_2(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/on")
    }

    /// /fxrtn/N/grp/dca (unknown)
    pub fn fxrtn_grp_dca(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/grp/dca")
    }

    /// /fxrtn/N/grp/mute (unknown)
    pub fn fxrtn_grp_mute(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/grp/mute")
    }

    /// /fxrtn/N/mix/N/level (float)
    pub fn fxrtn_mix_level(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/{n2:02}/level")
    }

    /// /fxrtn/N/mix/N/on (bool)
    pub fn fxrtn_mix_on(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/{n2:02}/on")
    }

    /// /fxrtn/N/mix/fader (float)
    pub fn fxrtn_mix_fader(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/fader")
    }

    /// /fxrtn/N/mix/mlevel (unknown)
    pub fn fxrtn_mix_mlevel(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/mlevel")
    }

    /// /fxrtn/N/mix/mono (bool)
    pub fn fxrtn_mix_mono(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/mono")
    }

    /// /fxrtn/N/mix/on (bool)
    pub fn fxrtn_mix_on_2(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/on")
    }

    /// /fxrtn/N/mix/pan (float)
    pub fn fxrtn_mix_pan(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/pan")
    }

    /// /fxrtn/N/mix/st (bool)
    pub fn fxrtn_mix_st(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/st")
    }

    // headamp
    /// /headamp/N/gain (unknown)
    pub fn headamp_gain(n1: u8) -> String {
        format!("/headamp/{n1:02}/gain")
    }

    /// /headamp/N/phatom (unknown)
    pub fn headamp_phatom(n1: u8) -> String {
        format!("/headamp/{n1:02}/phatom")
    }

    // main
    /// /main/m/config/color (int)
    pub fn main_m_config_color() -> String {
        String::from("/main/m/config/color")
    }

    /// /main/m/config/icon (int)
    pub fn main_m_config_icon() -> String {
        String::from("/main/m/config/icon")
    }

    /// /main/m/config/name (string)
    pub fn main_m_config_name() -> String {
        String::from("/main/m/config/name")
    }

    /// /main/m/dyn/attack (float)
    pub fn main_m_dyn_attack() -> String {
        String::from("/main/m/dyn/attack")
    }

    /// /main/m/dyn/auto (bool)
    pub fn main_m_dyn_auto() -> String {
        String::from("/main/m/dyn/auto")
    }

    /// /main/m/dyn/det (int)
    pub fn main_m_dyn_det() -> String {
        String::from("/main/m/dyn/det")
    }

    /// /main/m/dyn/env (int)
    pub fn main_m_dyn_env() -> String {
        String::from("/main/m/dyn/env")
    }

    /// /main/m/dyn/filter/f (float)
    pub fn main_m_dyn_filter_f() -> String {
        String::from("/main/m/dyn/filter/f")
    }

    /// /main/m/dyn/filter/on (bool)
    pub fn main_m_dyn_filter_on() -> String {
        String::from("/main/m/dyn/filter/on")
    }

    /// /main/m/dyn/filter/type (int)
    pub fn main_m_dyn_filter_type() -> String {
        String::from("/main/m/dyn/filter/type")
    }

    /// /main/m/dyn/hold (float)
    pub fn main_m_dyn_hold() -> String {
        String::from("/main/m/dyn/hold")
    }

    /// /main/m/dyn/knee (float)
    pub fn main_m_dyn_knee() -> String {
        String::from("/main/m/dyn/knee")
    }

    /// /main/m/dyn/mgain (float)
    pub fn main_m_dyn_mgain() -> String {
        String::from("/main/m/dyn/mgain")
    }

    /// /main/m/dyn/mix (float)
    pub fn main_m_dyn_mix() -> String {
        String::from("/main/m/dyn/mix")
    }

    /// /main/m/dyn/mode (int)
    pub fn main_m_dyn_mode() -> String {
        String::from("/main/m/dyn/mode")
    }

    /// /main/m/dyn/on (bool)
    pub fn main_m_dyn_on() -> String {
        String::from("/main/m/dyn/on")
    }

    /// /main/m/dyn/pos (int)
    pub fn main_m_dyn_pos() -> String {
        String::from("/main/m/dyn/pos")
    }

    /// /main/m/dyn/ratio (unknown)
    pub fn main_m_dyn_ratio() -> String {
        String::from("/main/m/dyn/ratio")
    }

    /// /main/m/dyn/release (float)
    pub fn main_m_dyn_release() -> String {
        String::from("/main/m/dyn/release")
    }

    /// /main/m/dyn/thr (float)
    pub fn main_m_dyn_thr() -> String {
        String::from("/main/m/dyn/thr")
    }

    /// /main/m/eq/N/f (float)
    pub fn main_m_eq_f(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/f")
    }

    /// /main/m/eq/N/g (float)
    pub fn main_m_eq_g(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/g")
    }

    /// /main/m/eq/N/on (bool)
    pub fn main_m_eq_on(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/on")
    }

    /// /main/m/eq/N/q (float)
    pub fn main_m_eq_q(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/q")
    }

    /// /main/m/eq/N/type (int)
    pub fn main_m_eq_type(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/type")
    }

    /// /main/m/eq/on (bool)
    pub fn main_m_eq_on_2() -> String {
        String::from("/main/m/eq/on")
    }

    /// /main/m/insert/on (bool)
    pub fn main_m_insert_on() -> String {
        String::from("/main/m/insert/on")
    }

    /// /main/m/insert/pos (int)
    pub fn main_m_insert_pos() -> String {
        String::from("/main/m/insert/pos")
    }

    /// /main/m/insert/sel (int)
    pub fn main_m_insert_sel() -> String {
        String::from("/main/m/insert/sel")
    }

    /// /main/m/mix/N/level (float)
    pub fn main_m_mix_level(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/level")
    }

    /// /main/m/mix/N/on (bool)
    pub fn main_m_mix_on(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/on")
    }

    /// /main/m/mix/N/pan (float)
    pub fn main_m_mix_pan(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/pan")
    }

    /// /main/m/mix/N/panFollow (unknown)
    pub fn main_m_mix_pan_follow(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/panFollow")
    }

    /// /main/m/mix/N/type (int)
    pub fn main_m_mix_type(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/type")
    }

    /// /main/m/mix/fader (mixed)
    pub fn main_m_mix_fader() -> String {
        String::from("/main/m/mix/fader")
    }

    /// /main/m/mix/on (bool)
    pub fn main_m_mix_on_2() -> String {
        String::from("/main/m/mix/on")
    }

    /// /main/st/config/color (int)
    pub fn main_st_config_color() -> String {
        String::from("/main/st/config/color")
    }

    /// /main/st/config/icon (int)
    pub fn main_st_config_icon() -> String {
        String::from("/main/st/config/icon")
    }

    /// /main/st/config/name (string)
    pub fn main_st_config_name() -> String {
        String::from("/main/st/config/name")
    }

    /// /main/st/dyn/attack (float)
    pub fn main_st_dyn_attack() -> String {
        String::from("/main/st/dyn/attack")
    }

    /// /main/st/dyn/auto (bool)
    pub fn main_st_dyn_auto() -> String {
        String::from("/main/st/dyn/auto")
    }

    /// /main/st/dyn/det (int)
    pub fn main_st_dyn_det() -> String {
        String::from("/main/st/dyn/det")
    }

    /// /main/st/dyn/env (int)
    pub fn main_st_dyn_env() -> String {
        String::from("/main/st/dyn/env")
    }

    /// /main/st/dyn/filter/f (float)
    pub fn main_st_dyn_filter_f() -> String {
        String::from("/main/st/dyn/filter/f")
    }

    /// /main/st/dyn/filter/on (bool)
    pub fn main_st_dyn_filter_on() -> String {
        String::from("/main/st/dyn/filter/on")
    }

    /// /main/st/dyn/filter/type (int)
    pub fn main_st_dyn_filter_type() -> String {
        String::from("/main/st/dyn/filter/type")
    }

    /// /main/st/dyn/hold (float)
    pub fn main_st_dyn_hold() -> String {
        String::from("/main/st/dyn/hold")
    }

    /// /main/st/dyn/knee (float)
    pub fn main_st_dyn_knee() -> String {
        String::from("/main/st/dyn/knee")
    }

    /// /main/st/dyn/mgain (float)
    pub fn main_st_dyn_mgain() -> String {
        String::from("/main/st/dyn/mgain")
    }

    /// /main/st/dyn/mix (float)
    pub fn main_st_dyn_mix() -> String {
        String::from("/main/st/dyn/mix")
    }

    /// /main/st/dyn/mode (int)
    pub fn main_st_dyn_mode() -> String {
        String::from("/main/st/dyn/mode")
    }

    /// /main/st/dyn/on (bool)
    pub fn main_st_dyn_on() -> String {
        String::from("/main/st/dyn/on")
    }

    /// /main/st/dyn/pos (int)
    pub fn main_st_dyn_pos() -> String {
        String::from("/main/st/dyn/pos")
    }

    /// /main/st/dyn/ratio (unknown)
    pub fn main_st_dyn_ratio() -> String {
        String::from("/main/st/dyn/ratio")
    }

    /// /main/st/dyn/release (float)
    pub fn main_st_dyn_release() -> String {
        String::from("/main/st/dyn/release")
    }

    /// /main/st/dyn/thr (float)
    pub fn main_st_dyn_thr() -> String {
        String::from("/main/st/dyn/thr")
    }

    /// /main/st/eq/N/f (float)
    pub fn main_st_eq_f(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/f")
    }

    /// /main/st/eq/N/g (float)
    pub fn main_st_eq_g(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/g")
    }

    /// /main/st/eq/N/on (bool)
    pub fn main_st_eq_on(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/on")
    }

    /// /main/st/eq/N/q (float)
    pub fn main_st_eq_q(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/q")
    }

    /// /main/st/eq/N/type (int)
    pub fn main_st_eq_type(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/type")
    }

    /// /main/st/eq/on (bool)
    pub fn main_st_eq_on_2() -> String {
        String::from("/main/st/eq/on")
    }

    /// /main/st/insert/on (bool)
    pub fn main_st_insert_on() -> String {
        String::from("/main/st/insert/on")
    }

    /// /main/st/insert/pos (int)
    pub fn main_st_insert_pos() -> String {
        String::from("/main/st/insert/pos")
    }

    /// /main/st/insert/sel (int)
    pub fn main_st_insert_sel() -> String {
        String::from("/main/st/insert/sel")
    }

    /// /main/st/mix/N/level (float)
    pub fn main_st_mix_level(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/level")
    }

    /// /main/st/mix/N/on (bool)
    pub fn main_st_mix_on(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/on")
    }

    /// /main/st/mix/N/pan (float)
    pub fn main_st_mix_pan(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/pan")
    }

    /// /main/st/mix/N/panFollow (unknown)
    pub fn main_st_mix_pan_follow(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/panFollow")
    }

    /// /main/st/mix/N/type (int)
    pub fn main_st_mix_type(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/type")
    }

    /// /main/st/mix/fader (mixed)
    pub fn main_st_mix_fader() -> String {
        String::from("/main/st/mix/fader")
    }

    /// /main/st/mix/on (bool)
    pub fn main_st_mix_on_2() -> String {
        String::from("/main/st/mix/on")
    }

    /// /main/st/mix/pan (mixed)
    pub fn main_st_mix_pan_2() -> String {
        String::from("/main/st/mix/pan")
    }

    // mtx
    /// /mtx/N/config/color (int)
    pub fn mtx_config_color(n1: u8) -> String {
        format!("/mtx/{n1:02}/config/color")
    }

    /// /mtx/N/config/icon (int)
    pub fn mtx_config_icon(n1: u8) -> String {
        format!("/mtx/{n1:02}/config/icon")
    }

    /// /mtx/N/config/name (string)
    pub fn mtx_config_name(n1: u8) -> String {
        format!("/mtx/{n1:02}/config/name")
    }

    /// /mtx/N/dyn/attack (float)
    pub fn mtx_dyn_attack(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/attack")
    }

    /// /mtx/N/dyn/auto (bool)
    pub fn mtx_dyn_auto(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/auto")
    }

    /// /mtx/N/dyn/det (int)
    pub fn mtx_dyn_det(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/det")
    }

    /// /mtx/N/dyn/env (int)
    pub fn mtx_dyn_env(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/env")
    }

    /// /mtx/N/dyn/filter/f (float)
    pub fn mtx_dyn_filter_f(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/filter/f")
    }

    /// /mtx/N/dyn/filter/on (bool)
    pub fn mtx_dyn_filter_on(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/filter/on")
    }

    /// /mtx/N/dyn/filter/type (int)
    pub fn mtx_dyn_filter_type(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/filter/type")
    }

    /// /mtx/N/dyn/hold (float)
    pub fn mtx_dyn_hold(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/hold")
    }

    /// /mtx/N/dyn/knee (float)
    pub fn mtx_dyn_knee(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/knee")
    }

    /// /mtx/N/dyn/mgain (float)
    pub fn mtx_dyn_mgain(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/mgain")
    }

    /// /mtx/N/dyn/mix (float)
    pub fn mtx_dyn_mix(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/mix")
    }

    /// /mtx/N/dyn/mode (int)
    pub fn mtx_dyn_mode(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/mode")
    }

    /// /mtx/N/dyn/on (bool)
    pub fn mtx_dyn_on(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/on")
    }

    /// /mtx/N/dyn/pos (int)
    pub fn mtx_dyn_pos(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/pos")
    }

    /// /mtx/N/dyn/ratio (unknown)
    pub fn mtx_dyn_ratio(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/ratio")
    }

    /// /mtx/N/dyn/release (float)
    pub fn mtx_dyn_release(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/release")
    }

    /// /mtx/N/dyn/thr (float)
    pub fn mtx_dyn_thr(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/thr")
    }

    /// /mtx/N/eq/N/f (float)
    pub fn mtx_eq_f(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/f")
    }

    /// /mtx/N/eq/N/g (float)
    pub fn mtx_eq_g(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/g")
    }

    /// /mtx/N/eq/N/on (bool)
    pub fn mtx_eq_on(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/on")
    }

    /// /mtx/N/eq/N/q (float)
    pub fn mtx_eq_q(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/q")
    }

    /// /mtx/N/eq/N/type (int)
    pub fn mtx_eq_type(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/type")
    }

    /// /mtx/N/eq/on (bool)
    pub fn mtx_eq_on_2(n1: u8) -> String {
        format!("/mtx/{n1:02}/eq/on")
    }

    /// /mtx/N/insert/on (bool)
    pub fn mtx_insert_on(n1: u8) -> String {
        format!("/mtx/{n1:02}/insert/on")
    }

    /// /mtx/N/insert/pos (int)
    pub fn mtx_insert_pos(n1: u8) -> String {
        format!("/mtx/{n1:02}/insert/pos")
    }

    /// /mtx/N/insert/sel (int)
    pub fn mtx_insert_sel(n1: u8) -> String {
        format!("/mtx/{n1:02}/insert/sel")
    }

    /// /mtx/N/mix/fader (float)
    pub fn mtx_mix_fader(n1: u8) -> String {
        format!("/mtx/{n1:02}/mix/fader")
    }

    /// /mtx/N/mix/on (bool)
    pub fn mtx_mix_on(n1: u8) -> String {
        format!("/mtx/{n1:02}/mix/on")
    }

    /// /mtx/N/preamp (unknown)
    pub fn mtx_preamp(n1: u8) -> String {
        format!("/mtx/{n1:02}/preamp")
    }

    // outputs
    /// /outputs/aes/N/invert (bool)
    pub fn outputs_aes_invert(n1: u8) -> String {
        format!("/outputs/aes/{n1:02}/invert")
    }

    /// /outputs/aes/N/pos (int)
    pub fn outputs_aes_pos(n1: u8) -> String {
        format!("/outputs/aes/{n1:02}/pos")
    }

    /// /outputs/aes/N/src (int)
    pub fn outputs_aes_src(n1: u8) -> String {
        format!("/outputs/aes/{n1:02}/src")
    }

    /// /outputs/aux/N/invert (bool)
    pub fn outputs_aux_invert(n1: u8) -> String {
        format!("/outputs/aux/{n1:02}/invert")
    }

    /// /outputs/aux/N/pos (int)
    pub fn outputs_aux_pos(n1: u8) -> String {
        format!("/outputs/aux/{n1:02}/pos")
    }

    /// /outputs/aux/N/src (int)
    pub fn outputs_aux_src(n1: u8) -> String {
        format!("/outputs/aux/{n1:02}/src")
    }

    /// /outputs/main/N/delay/on (bool)
    pub fn outputs_main_delay_on(n1: u8) -> String {
        format!("/outputs/main/{n1:02}/delay/on")
    }

    /// /outputs/main/N/invert (bool)
    pub fn outputs_main_invert(n1: u8) -> String {
        format!("/outputs/main/{n1:02}/invert")
    }

    /// /outputs/main/N/pos (int)
    pub fn outputs_main_pos(n1: u8) -> String {
        format!("/outputs/main/{n1:02}/pos")
    }

    /// /outputs/main/N/src (int)
    pub fn outputs_main_src(n1: u8) -> String {
        format!("/outputs/main/{n1:02}/src")
    }

    /// /outputs/p16/N/iQ/eq (unknown)
    pub fn outputs_p16_i_q_eq(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/iQ/eq")
    }

    /// /outputs/p16/N/iQ/group (int)
    pub fn outputs_p16_i_q_group(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/iQ/group")
    }

    /// /outputs/p16/N/iQ/model (unknown)
    pub fn outputs_p16_i_q_model(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/iQ/model")
    }

    /// /outputs/p16/N/iQ/speaker (unknown)
    pub fn outputs_p16_i_q_speaker(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/iQ/speaker")
    }

    /// /outputs/p16/N/invert (bool)
    pub fn outputs_p16_invert(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/invert")
    }

    /// /outputs/p16/N/pos (int)
    pub fn outputs_p16_pos(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/pos")
    }

    /// /outputs/p16/N/src (int)
    pub fn outputs_p16_src(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/src")
    }

    /// /outputs/rec/N/pos (int)
    pub fn outputs_rec_pos(n1: u8) -> String {
        format!("/outputs/rec/{n1:02}/pos")
    }

    /// /outputs/rec/N/src (int)
    pub fn outputs_rec_src(n1: u8) -> String {
        format!("/outputs/rec/{n1:02}/src")
    }
}
/// Parameters for Channel strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelParam {
    AutomixGroup,
    AutomixWeight,
    ConfigColor,
    ConfigIcon,
    ConfigName,
    ConfigSource,
    DelayOn,
    DelayTime,
    DynAttack,
    DynAuto,
    DynDet,
    DynEnv,
    DynFilterF,
    DynFilterOn,
    DynFilterType,
    DynHold,
    DynKeysrc,
    DynKnee,
    DynMgain,
    DynMix,
    DynMode,
    DynOn,
    DynPos,
    DynRatio,
    DynRelease,
    DynThr,
    EqF { index: u8 },
    EqG { index: u8 },
    EqOn { index: u8 },
    EqQ { index: u8 },
    EqType { index: u8 },
    GateAttack,
    GateFilterF,
    GateFilterOn,
    GateFilterType,
    GateHold,
    GateKeysrc,
    GateMode,
    GateOn,
    GateRange,
    GateRelease,
    GateThr,
    GrpDca,
    GrpMute,
    InsertOn,
    InsertPos,
    InsertSel,
    MixLevel { index: u8 },
    MixPan { index: u8 },
    MixPanfollow { index: u8 },
    MixType { index: u8 },
    MixFader,
    MixMlevel,
    MixMono,
    MixOn,
    MixSt,
    PreamHpon,
    PreampHpf,
    PreampHpon,
    PreampHpslope,
    PreampInvert,
    PreampTrim,
}

impl ChannelParam {
    /// Return the OSC path for this parameter on a given strip.
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::AutomixGroup => format!("/ch/{strip:02}/automix/group"),
            Self::AutomixWeight => format!("/ch/{strip:02}/automix/weight"),
            Self::ConfigColor => format!("/ch/{strip:02}/config/color"),
            Self::ConfigIcon => format!("/ch/{strip:02}/config/icon"),
            Self::ConfigName => format!("/ch/{strip:02}/config/name"),
            Self::ConfigSource => format!("/ch/{strip:02}/config/source"),
            Self::DelayOn => format!("/ch/{strip:02}/delay/on"),
            Self::DelayTime => format!("/ch/{strip:02}/delay/time"),
            Self::DynAttack => format!("/ch/{strip:02}/dyn/attack"),
            Self::DynAuto => format!("/ch/{strip:02}/dyn/auto"),
            Self::DynDet => format!("/ch/{strip:02}/dyn/det"),
            Self::DynEnv => format!("/ch/{strip:02}/dyn/env"),
            Self::DynFilterF => format!("/ch/{strip:02}/dyn/filter/f"),
            Self::DynFilterOn => format!("/ch/{strip:02}/dyn/filter/on"),
            Self::DynFilterType => format!("/ch/{strip:02}/dyn/filter/type"),
            Self::DynHold => format!("/ch/{strip:02}/dyn/hold"),
            Self::DynKeysrc => format!("/ch/{strip:02}/dyn/keysrc"),
            Self::DynKnee => format!("/ch/{strip:02}/dyn/knee"),
            Self::DynMgain => format!("/ch/{strip:02}/dyn/mgain"),
            Self::DynMix => format!("/ch/{strip:02}/dyn/mix"),
            Self::DynMode => format!("/ch/{strip:02}/dyn/mode"),
            Self::DynOn => format!("/ch/{strip:02}/dyn/on"),
            Self::DynPos => format!("/ch/{strip:02}/dyn/pos"),
            Self::DynRatio => format!("/ch/{strip:02}/dyn/ratio"),
            Self::DynRelease => format!("/ch/{strip:02}/dyn/release"),
            Self::DynThr => format!("/ch/{strip:02}/dyn/thr"),
            Self::EqF { index } => format!("/ch/{strip:02}/eq/{index:02}/f"),
            Self::EqG { index } => format!("/ch/{strip:02}/eq/{index:02}/g"),
            Self::EqOn { index } => format!("/ch/{strip:02}/eq/{index:02}/on"),
            Self::EqQ { index } => format!("/ch/{strip:02}/eq/{index:02}/q"),
            Self::EqType { index } => format!("/ch/{strip:02}/eq/{index:02}/type"),
            Self::GateAttack => format!("/ch/{strip:02}/gate/attack"),
            Self::GateFilterF => format!("/ch/{strip:02}/gate/filter/f"),
            Self::GateFilterOn => format!("/ch/{strip:02}/gate/filter/on"),
            Self::GateFilterType => format!("/ch/{strip:02}/gate/filter/type"),
            Self::GateHold => format!("/ch/{strip:02}/gate/hold"),
            Self::GateKeysrc => format!("/ch/{strip:02}/gate/keysrc"),
            Self::GateMode => format!("/ch/{strip:02}/gate/mode"),
            Self::GateOn => format!("/ch/{strip:02}/gate/on"),
            Self::GateRange => format!("/ch/{strip:02}/gate/range"),
            Self::GateRelease => format!("/ch/{strip:02}/gate/release"),
            Self::GateThr => format!("/ch/{strip:02}/gate/thr"),
            Self::GrpDca => format!("/ch/{strip:02}/grp/dca"),
            Self::GrpMute => format!("/ch/{strip:02}/grp/mute"),
            Self::InsertOn => format!("/ch/{strip:02}/insert/on"),
            Self::InsertPos => format!("/ch/{strip:02}/insert/pos"),
            Self::InsertSel => format!("/ch/{strip:02}/insert/sel"),
            Self::MixLevel { index } => format!("/ch/{strip:02}/mix/{index:02}/level"),
            Self::MixPan { index } => format!("/ch/{strip:02}/mix/{index:02}/pan"),
            Self::MixPanfollow { index } => format!("/ch/{strip:02}/mix/{index:02}/panFollow"),
            Self::MixType { index } => format!("/ch/{strip:02}/mix/{index:02}/type"),
            Self::MixFader => format!("/ch/{strip:02}/mix/fader"),
            Self::MixMlevel => format!("/ch/{strip:02}/mix/mlevel"),
            Self::MixMono => format!("/ch/{strip:02}/mix/mono"),
            Self::MixOn => format!("/ch/{strip:02}/mix/on"),
            Self::MixSt => format!("/ch/{strip:02}/mix/st"),
            Self::PreamHpon => format!("/ch/{strip:02}/pream/hpon"),
            Self::PreampHpf => format!("/ch/{strip:02}/preamp/hpf"),
            Self::PreampHpon => format!("/ch/{strip:02}/preamp/hpon"),
            Self::PreampHpslope => format!("/ch/{strip:02}/preamp/hpslope"),
            Self::PreampInvert => format!("/ch/{strip:02}/preamp/invert"),
            Self::PreampTrim => format!("/ch/{strip:02}/preamp/trim"),
        }
    }
}

/// Parameters for Bus strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusParam {
    ConfigColor,
    ConfigIcon,
    ConfigName,
    DynAttack,
    DynAuto,
    DynDet,
    DynEnv,
    DynFilterF,
    DynFilterOn,
    DynFilterType,
    DynHold,
    DynKeysrc,
    DynKnee,
    DynMgain,
    DynMix,
    DynMode,
    DynOn,
    DynPos,
    DynRatio,
    DynRelease,
    DynThr,
    EqF { index: u8 },
    EqG { index: u8 },
    EqOn { index: u8 },
    EqQ { index: u8 },
    EqType { index: u8 },
    GrpDca,
    GrpMute,
    InsertOn,
    InsertPos,
    InsertSel,
    MixLevel { index: u8 },
    MixOn { index: u8 },
    MixPan { index: u8 },
    MixPanfollow { index: u8 },
    MixType { index: u8 },
    MixFader,
    MixMlevel,
    MixMono,
    MixSt,
}

impl BusParam {
    /// Return the OSC path for this parameter on a given strip.
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::ConfigColor => format!("/bus/{strip:02}/config/color"),
            Self::ConfigIcon => format!("/bus/{strip:02}/config/icon"),
            Self::ConfigName => format!("/bus/{strip:02}/config/name"),
            Self::DynAttack => format!("/bus/{strip:02}/dyn/attack"),
            Self::DynAuto => format!("/bus/{strip:02}/dyn/auto"),
            Self::DynDet => format!("/bus/{strip:02}/dyn/det"),
            Self::DynEnv => format!("/bus/{strip:02}/dyn/env"),
            Self::DynFilterF => format!("/bus/{strip:02}/dyn/filter/f"),
            Self::DynFilterOn => format!("/bus/{strip:02}/dyn/filter/on"),
            Self::DynFilterType => format!("/bus/{strip:02}/dyn/filter/type"),
            Self::DynHold => format!("/bus/{strip:02}/dyn/hold"),
            Self::DynKeysrc => format!("/bus/{strip:02}/dyn/keysrc"),
            Self::DynKnee => format!("/bus/{strip:02}/dyn/knee"),
            Self::DynMgain => format!("/bus/{strip:02}/dyn/mgain"),
            Self::DynMix => format!("/bus/{strip:02}/dyn/mix"),
            Self::DynMode => format!("/bus/{strip:02}/dyn/mode"),
            Self::DynOn => format!("/bus/{strip:02}/dyn/on"),
            Self::DynPos => format!("/bus/{strip:02}/dyn/pos"),
            Self::DynRatio => format!("/bus/{strip:02}/dyn/ratio"),
            Self::DynRelease => format!("/bus/{strip:02}/dyn/release"),
            Self::DynThr => format!("/bus/{strip:02}/dyn/thr"),
            Self::EqF { index } => format!("/bus/{strip:02}/eq/{index:02}/f"),
            Self::EqG { index } => format!("/bus/{strip:02}/eq/{index:02}/g"),
            Self::EqOn { index } => format!("/bus/{strip:02}/eq/{index:02}/on"),
            Self::EqQ { index } => format!("/bus/{strip:02}/eq/{index:02}/q"),
            Self::EqType { index } => format!("/bus/{strip:02}/eq/{index:02}/type"),
            Self::GrpDca => format!("/bus/{strip:02}/grp/dca"),
            Self::GrpMute => format!("/bus/{strip:02}/grp/mute"),
            Self::InsertOn => format!("/bus/{strip:02}/insert/on"),
            Self::InsertPos => format!("/bus/{strip:02}/insert/pos"),
            Self::InsertSel => format!("/bus/{strip:02}/insert/sel"),
            Self::MixLevel { index } => format!("/bus/{strip:02}/mix/{index:02}/level"),
            Self::MixOn { index } => format!("/bus/{strip:02}/mix/{index:02}/on"),
            Self::MixPan { index } => format!("/bus/{strip:02}/mix/{index:02}/pan"),
            Self::MixPanfollow { index } => format!("/bus/{strip:02}/mix/{index:02}/panFollow"),
            Self::MixType { index } => format!("/bus/{strip:02}/mix/{index:02}/type"),
            Self::MixFader => format!("/bus/{strip:02}/mix/fader"),
            Self::MixMlevel => format!("/bus/{strip:02}/mix/mlevel"),
            Self::MixMono => format!("/bus/{strip:02}/mix/mono"),
            Self::MixSt => format!("/bus/{strip:02}/mix/st"),
        }
    }
}

/// Parameters for AuxIn strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxInParam {
    ConfigColor,
    ConfigIcon,
    ConfigName,
    ConfigSource,
    EqF { index: u8 },
    EqG { index: u8 },
    EqOn { index: u8 },
    EqQ { index: u8 },
    EqType { index: u8 },
    GrpDca,
    GrpMute,
    MixMevel { index: u8 },
    MixOn { index: u8 },
    MixPan { index: u8 },
    MixPanfollow { index: u8 },
    MixType { index: u8 },
    MixFader,
    MixMlevel,
    MixMono,
    MixSt,
    PreampInvert,
    PreampTrim,
}

impl AuxInParam {
    /// Return the OSC path for this parameter on a given strip.
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::ConfigColor => format!("/auxin/{strip:02}/config/color"),
            Self::ConfigIcon => format!("/auxin/{strip:02}/config/icon"),
            Self::ConfigName => format!("/auxin/{strip:02}/config/name"),
            Self::ConfigSource => format!("/auxin/{strip:02}/config/source"),
            Self::EqF { index } => format!("/auxin/{strip:02}/eq/{index:02}/f"),
            Self::EqG { index } => format!("/auxin/{strip:02}/eq/{index:02}/g"),
            Self::EqOn { index } => format!("/auxin/{strip:02}/eq/{index:02}/on"),
            Self::EqQ { index } => format!("/auxin/{strip:02}/eq/{index:02}/q"),
            Self::EqType { index } => format!("/auxin/{strip:02}/eq/{index:02}/type"),
            Self::GrpDca => format!("/auxin/{strip:02}/grp/dca"),
            Self::GrpMute => format!("/auxin/{strip:02}/grp/mute"),
            Self::MixMevel { index } => format!("/auxin/{strip:02}/mix/{index:02}/mevel"),
            Self::MixOn { index } => format!("/auxin/{strip:02}/mix/{index:02}/on"),
            Self::MixPan { index } => format!("/auxin/{strip:02}/mix/{index:02}/pan"),
            Self::MixPanfollow { index } => format!("/auxin/{strip:02}/mix/{index:02}/panFollow"),
            Self::MixType { index } => format!("/auxin/{strip:02}/mix/{index:02}/type"),
            Self::MixFader => format!("/auxin/{strip:02}/mix/fader"),
            Self::MixMlevel => format!("/auxin/{strip:02}/mix/mlevel"),
            Self::MixMono => format!("/auxin/{strip:02}/mix/mono"),
            Self::MixSt => format!("/auxin/{strip:02}/mix/st"),
            Self::PreampInvert => format!("/auxin/{strip:02}/preamp/invert"),
            Self::PreampTrim => format!("/auxin/{strip:02}/preamp/trim"),
        }
    }
}

/// Parameters for FxRtn strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxRtnParam {
    ConfigColor,
    ConfigIcon,
    ConfigName,
    EqF { index: u8 },
    EqG { index: u8 },
    EqOn { index: u8 },
    EqQ { index: u8 },
    EqType { index: u8 },
    GrpDca,
    GrpMute,
    MixLevel { index: u8 },
    MixOn { index: u8 },
    MixFader,
    MixMlevel,
    MixMono,
    MixPan,
    MixSt,
}

impl FxRtnParam {
    /// Return the OSC path for this parameter on a given strip.
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::ConfigColor => format!("/fxrtn/{strip:02}/config/color"),
            Self::ConfigIcon => format!("/fxrtn/{strip:02}/config/icon"),
            Self::ConfigName => format!("/fxrtn/{strip:02}/config/name"),
            Self::EqF { index } => format!("/fxrtn/{strip:02}/eq/{index:02}/f"),
            Self::EqG { index } => format!("/fxrtn/{strip:02}/eq/{index:02}/g"),
            Self::EqOn { index } => format!("/fxrtn/{strip:02}/eq/{index:02}/on"),
            Self::EqQ { index } => format!("/fxrtn/{strip:02}/eq/{index:02}/q"),
            Self::EqType { index } => format!("/fxrtn/{strip:02}/eq/{index:02}/type"),
            Self::GrpDca => format!("/fxrtn/{strip:02}/grp/dca"),
            Self::GrpMute => format!("/fxrtn/{strip:02}/grp/mute"),
            Self::MixLevel { index } => format!("/fxrtn/{strip:02}/mix/{index:02}/level"),
            Self::MixOn { index } => format!("/fxrtn/{strip:02}/mix/{index:02}/on"),
            Self::MixFader => format!("/fxrtn/{strip:02}/mix/fader"),
            Self::MixMlevel => format!("/fxrtn/{strip:02}/mix/mlevel"),
            Self::MixMono => format!("/fxrtn/{strip:02}/mix/mono"),
            Self::MixPan => format!("/fxrtn/{strip:02}/mix/pan"),
            Self::MixSt => format!("/fxrtn/{strip:02}/mix/st"),
        }
    }
}

/// Parameters for Mtx strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MtxParam {
    ConfigColor,
    ConfigIcon,
    ConfigName,
    DynAttack,
    DynAuto,
    DynDet,
    DynEnv,
    DynFilterF,
    DynFilterOn,
    DynFilterType,
    DynHold,
    DynKnee,
    DynMgain,
    DynMix,
    DynMode,
    DynOn,
    DynPos,
    DynRatio,
    DynRelease,
    DynThr,
    EqF { index: u8 },
    EqG { index: u8 },
    EqOn { index: u8 },
    EqQ { index: u8 },
    EqType { index: u8 },
    InsertOn,
    InsertPos,
    InsertSel,
    MixFader,
    MixOn,
    Preamp,
}

impl MtxParam {
    /// Return the OSC path for this parameter on a given strip.
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::ConfigColor => format!("/mtx/{strip:02}/config/color"),
            Self::ConfigIcon => format!("/mtx/{strip:02}/config/icon"),
            Self::ConfigName => format!("/mtx/{strip:02}/config/name"),
            Self::DynAttack => format!("/mtx/{strip:02}/dyn/attack"),
            Self::DynAuto => format!("/mtx/{strip:02}/dyn/auto"),
            Self::DynDet => format!("/mtx/{strip:02}/dyn/det"),
            Self::DynEnv => format!("/mtx/{strip:02}/dyn/env"),
            Self::DynFilterF => format!("/mtx/{strip:02}/dyn/filter/f"),
            Self::DynFilterOn => format!("/mtx/{strip:02}/dyn/filter/on"),
            Self::DynFilterType => format!("/mtx/{strip:02}/dyn/filter/type"),
            Self::DynHold => format!("/mtx/{strip:02}/dyn/hold"),
            Self::DynKnee => format!("/mtx/{strip:02}/dyn/knee"),
            Self::DynMgain => format!("/mtx/{strip:02}/dyn/mgain"),
            Self::DynMix => format!("/mtx/{strip:02}/dyn/mix"),
            Self::DynMode => format!("/mtx/{strip:02}/dyn/mode"),
            Self::DynOn => format!("/mtx/{strip:02}/dyn/on"),
            Self::DynPos => format!("/mtx/{strip:02}/dyn/pos"),
            Self::DynRatio => format!("/mtx/{strip:02}/dyn/ratio"),
            Self::DynRelease => format!("/mtx/{strip:02}/dyn/release"),
            Self::DynThr => format!("/mtx/{strip:02}/dyn/thr"),
            Self::EqF { index } => format!("/mtx/{strip:02}/eq/{index:02}/f"),
            Self::EqG { index } => format!("/mtx/{strip:02}/eq/{index:02}/g"),
            Self::EqOn { index } => format!("/mtx/{strip:02}/eq/{index:02}/on"),
            Self::EqQ { index } => format!("/mtx/{strip:02}/eq/{index:02}/q"),
            Self::EqType { index } => format!("/mtx/{strip:02}/eq/{index:02}/type"),
            Self::InsertOn => format!("/mtx/{strip:02}/insert/on"),
            Self::InsertPos => format!("/mtx/{strip:02}/insert/pos"),
            Self::InsertSel => format!("/mtx/{strip:02}/insert/sel"),
            Self::MixFader => format!("/mtx/{strip:02}/mix/fader"),
            Self::MixOn => format!("/mtx/{strip:02}/mix/on"),
            Self::Preamp => format!("/mtx/{strip:02}/preamp"),
        }
    }
}

/// Parameters for Dca strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DcaParam {
    ConfigColor,
    ConfigIcon,
    ConfigName,
    Fader,
    MixFader,
    MixOn,
    On,
}

impl DcaParam {
    /// Return the OSC path for this parameter on a given strip.
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::ConfigColor => format!("/dca/{strip:02}/config/color"),
            Self::ConfigIcon => format!("/dca/{strip:02}/config/icon"),
            Self::ConfigName => format!("/dca/{strip:02}/config/name"),
            Self::Fader => format!("/dca/{strip:02}/fader"),
            Self::MixFader => format!("/dca/{strip:02}/mix/fader"),
            Self::MixOn => format!("/dca/{strip:02}/mix/on"),
            Self::On => format!("/dca/{strip:02}/on"),
        }
    }
}

/// Parameters for Fx strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxParam {
    Par { index: u8 },
    SourceL,
    SourceR,
    Type,
}

impl FxParam {
    /// Return the OSC path for this parameter on a given strip.
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::Par { index } => format!("/fx/{strip:02}/par/{index:02}"),
            Self::SourceL => format!("/fx/{strip:02}/source/l"),
            Self::SourceR => format!("/fx/{strip:02}/source/r"),
            Self::Type => format!("/fx/{strip:02}/type"),
        }
    }
}

/// Parameters for Headamp strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadampParam {
    Gain,
    Phatom,
}

impl HeadampParam {
    /// Return the OSC path for this parameter on a given strip.
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::Gain => format!("/headamp/{strip:02}/gain"),
            Self::Phatom => format!("/headamp/{strip:02}/phatom"),
        }
    }
}

/// Parameters for MainStereo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainStereoParam {
    Color,
    Icon,
    Name,
    Attack,
    Auto,
    Det,
    Env,
    FilterF,
    FilterOn,
    FilterType,
    Hold,
    Knee,
    Mgain,
    Mix,
    Mode,
    On,
    Pos,
    Ratio,
    Release,
    Thr,
    F { index: u8 },
    G { index: u8 },
    Q { index: u8 },
    Type { index: u8 },
    Sel,
    Level { index: u8 },
    Pan { index: u8 },
    Panfollow { index: u8 },
    Fader,
}

impl MainStereoParam {
    pub fn path(&self) -> String {
        match self {
            Self::Color => String::from("/main/st/config/color"),
            Self::Icon => String::from("/main/st/config/icon"),
            Self::Name => String::from("/main/st/config/name"),
            Self::Attack => String::from("/main/st/dyn/attack"),
            Self::Auto => String::from("/main/st/dyn/auto"),
            Self::Det => String::from("/main/st/dyn/det"),
            Self::Env => String::from("/main/st/dyn/env"),
            Self::FilterF => String::from("/main/st/dyn/filter/f"),
            Self::FilterOn => String::from("/main/st/dyn/filter/on"),
            Self::FilterType => String::from("/main/st/dyn/filter/type"),
            Self::Hold => String::from("/main/st/dyn/hold"),
            Self::Knee => String::from("/main/st/dyn/knee"),
            Self::Mgain => String::from("/main/st/dyn/mgain"),
            Self::Mix => String::from("/main/st/dyn/mix"),
            Self::Mode => String::from("/main/st/dyn/mode"),
            Self::On => String::from("/main/st/dyn/on"),
            Self::Pos => String::from("/main/st/dyn/pos"),
            Self::Ratio => String::from("/main/st/dyn/ratio"),
            Self::Release => String::from("/main/st/dyn/release"),
            Self::Thr => String::from("/main/st/dyn/thr"),
            Self::F { index } => format!("/main/st/eq/{index:02}/f"),
            Self::G { index } => format!("/main/st/eq/{index:02}/g"),
            Self::Q { index } => format!("/main/st/eq/{index:02}/q"),
            Self::Type { index } => format!("/main/st/eq/{index:02}/type"),
            Self::Sel => String::from("/main/st/insert/sel"),
            Self::Level { index } => format!("/main/st/mix/{index:02}/level"),
            Self::Pan { index } => format!("/main/st/mix/{index:02}/pan"),
            Self::Panfollow { index } => format!("/main/st/mix/{index:02}/panFollow"),
            Self::Fader => String::from("/main/st/mix/fader"),
        }
    }
}

/// Parameters for MainMono.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMonoParam {
    Color,
    Icon,
    Name,
    Attack,
    Auto,
    Det,
    Env,
    FilterF,
    FilterOn,
    FilterType,
    Hold,
    Knee,
    Mgain,
    Mix,
    Mode,
    On,
    Pos,
    Ratio,
    Release,
    Thr,
    F { index: u8 },
    G { index: u8 },
    Q { index: u8 },
    Type { index: u8 },
    Sel,
    Level { index: u8 },
    Pan { index: u8 },
    Panfollow { index: u8 },
    Fader,
}

impl MainMonoParam {
    pub fn path(&self) -> String {
        match self {
            Self::Color => String::from("/main/m/config/color"),
            Self::Icon => String::from("/main/m/config/icon"),
            Self::Name => String::from("/main/m/config/name"),
            Self::Attack => String::from("/main/m/dyn/attack"),
            Self::Auto => String::from("/main/m/dyn/auto"),
            Self::Det => String::from("/main/m/dyn/det"),
            Self::Env => String::from("/main/m/dyn/env"),
            Self::FilterF => String::from("/main/m/dyn/filter/f"),
            Self::FilterOn => String::from("/main/m/dyn/filter/on"),
            Self::FilterType => String::from("/main/m/dyn/filter/type"),
            Self::Hold => String::from("/main/m/dyn/hold"),
            Self::Knee => String::from("/main/m/dyn/knee"),
            Self::Mgain => String::from("/main/m/dyn/mgain"),
            Self::Mix => String::from("/main/m/dyn/mix"),
            Self::Mode => String::from("/main/m/dyn/mode"),
            Self::On => String::from("/main/m/dyn/on"),
            Self::Pos => String::from("/main/m/dyn/pos"),
            Self::Ratio => String::from("/main/m/dyn/ratio"),
            Self::Release => String::from("/main/m/dyn/release"),
            Self::Thr => String::from("/main/m/dyn/thr"),
            Self::F { index } => format!("/main/m/eq/{index:02}/f"),
            Self::G { index } => format!("/main/m/eq/{index:02}/g"),
            Self::Q { index } => format!("/main/m/eq/{index:02}/q"),
            Self::Type { index } => format!("/main/m/eq/{index:02}/type"),
            Self::Sel => String::from("/main/m/insert/sel"),
            Self::Level { index } => format!("/main/m/mix/{index:02}/level"),
            Self::Pan { index } => format!("/main/m/mix/{index:02}/pan"),
            Self::Panfollow { index } => format!("/main/m/mix/{index:02}/panFollow"),
            Self::Fader => String::from("/main/m/mix/fader"),
        }
    }
}

/// Parameters for aes outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputAesParam {
    Invert,
    Pos,
    Src,
}

impl OutputAesParam {
    pub fn path(&self, output: u8) -> String {
        match self {
            Self::Invert => format!("/outputs/aes/{output:02}/invert"),
            Self::Pos => format!("/outputs/aes/{output:02}/pos"),
            Self::Src => format!("/outputs/aes/{output:02}/src"),
        }
    }
}

/// Parameters for aux outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputAuxParam {
    Invert,
    Pos,
    Src,
}

impl OutputAuxParam {
    pub fn path(&self, output: u8) -> String {
        match self {
            Self::Invert => format!("/outputs/aux/{output:02}/invert"),
            Self::Pos => format!("/outputs/aux/{output:02}/pos"),
            Self::Src => format!("/outputs/aux/{output:02}/src"),
        }
    }
}

/// Parameters for main outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMainParam {
    DelayOn,
    Invert,
    Pos,
    Src,
}

impl OutputMainParam {
    pub fn path(&self, output: u8) -> String {
        match self {
            Self::DelayOn => format!("/outputs/main/{output:02}/delay/on"),
            Self::Invert => format!("/outputs/main/{output:02}/invert"),
            Self::Pos => format!("/outputs/main/{output:02}/pos"),
            Self::Src => format!("/outputs/main/{output:02}/src"),
        }
    }
}

/// Parameters for p16 outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputP16Param {
    IqEq,
    IqGroup,
    IqModel,
    IqSpeaker,
    Invert,
    Pos,
    Src,
}

impl OutputP16Param {
    pub fn path(&self, output: u8) -> String {
        match self {
            Self::IqEq => format!("/outputs/p16/{output:02}/iQ/eq"),
            Self::IqGroup => format!("/outputs/p16/{output:02}/iQ/group"),
            Self::IqModel => format!("/outputs/p16/{output:02}/iQ/model"),
            Self::IqSpeaker => format!("/outputs/p16/{output:02}/iQ/speaker"),
            Self::Invert => format!("/outputs/p16/{output:02}/invert"),
            Self::Pos => format!("/outputs/p16/{output:02}/pos"),
            Self::Src => format!("/outputs/p16/{output:02}/src"),
        }
    }
}

/// Parameters for rec outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputRecParam {
    Pos,
    Src,
}

impl OutputRecParam {
    pub fn path(&self, output: u8) -> String {
        match self {
            Self::Pos => format!("/outputs/rec/{output:02}/pos"),
            Self::Src => format!("/outputs/rec/{output:02}/src"),
        }
    }
}
