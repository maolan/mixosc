use crate::common::{
    osc_address, osc_float_message, osc_int_message, osc_padded_len, osc_string, osc_string_message,
};

#[derive(Debug, Clone, PartialEq)]
pub enum OscValue {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
}

impl OscValue {
    pub fn float(v: f32) -> Self {
        Self::Float(v)
    }

    pub fn int(v: i32) -> Self {
        Self::Int(v)
    }

    pub fn string(v: impl Into<String>) -> Self {
        Self::String(v.into())
    }

    pub fn bool(v: bool) -> Self {
        Self::Bool(v)
    }
}

pub fn build_get(path: &str) -> Vec<u8> {
    osc_string(path)
}

pub fn build_set(path: &str, value: OscValue) -> Vec<u8> {
    match value {
        OscValue::Float(v) => osc_float_message(path, v),
        OscValue::Int(v) => osc_int_message(path, v),
        OscValue::String(v) => osc_string_message(path, &v),
        OscValue::Bool(v) => osc_int_message(path, i32::from(v)),
    }
}

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

pub mod path {

    pub fn _stat_solosw(n1: u8) -> String {
        format!("/-stat/solosw/{n1:02}")
    }

    pub fn _stat_talk_a() -> String {
        String::from("/-stat/talk/A")
    }

    pub fn _stat_talk_b() -> String {
        String::from("/-stat/talk/B")
    }

    pub fn auxin_config_color(n1: u8) -> String {
        format!("/auxin/{n1:02}/config/color")
    }

    pub fn auxin_config_icon(n1: u8) -> String {
        format!("/auxin/{n1:02}/config/icon")
    }

    pub fn auxin_config_name(n1: u8) -> String {
        format!("/auxin/{n1:02}/config/name")
    }

    pub fn auxin_config_source(n1: u8) -> String {
        format!("/auxin/{n1:02}/config/source")
    }

    pub fn auxin_eq_f(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/f")
    }

    pub fn auxin_eq_g(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/g")
    }

    pub fn auxin_eq_on(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/on")
    }

    pub fn auxin_eq_q(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/q")
    }

    pub fn auxin_eq_type(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/eq/{n2:02}/type")
    }

    pub fn auxin_eq_on_2(n1: u8) -> String {
        format!("/auxin/{n1:02}/eq/on")
    }

    pub fn auxin_grp_dca(n1: u8) -> String {
        format!("/auxin/{n1:02}/grp/dca")
    }

    pub fn auxin_grp_mute(n1: u8) -> String {
        format!("/auxin/{n1:02}/grp/mute")
    }

    pub fn auxin_mix_mevel(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/mevel")
    }

    pub fn auxin_mix_on(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/on")
    }

    pub fn auxin_mix_pan(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/pan")
    }

    pub fn auxin_mix_pan_follow(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/panFollow")
    }

    pub fn auxin_mix_type(n1: u8, n2: u8) -> String {
        format!("/auxin/{n1:02}/mix/{n2:02}/type")
    }

    pub fn auxin_mix_fader(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/fader")
    }

    pub fn auxin_mix_mlevel(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/mlevel")
    }

    pub fn auxin_mix_mono(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/mono")
    }

    pub fn auxin_mix_on_2(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/on")
    }

    pub fn auxin_mix_pan_2(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/pan")
    }

    pub fn auxin_mix_st(n1: u8) -> String {
        format!("/auxin/{n1:02}/mix/st")
    }

    pub fn auxin_preamp_invert(n1: u8) -> String {
        format!("/auxin/{n1:02}/preamp/invert")
    }

    pub fn auxin_preamp_trim(n1: u8) -> String {
        format!("/auxin/{n1:02}/preamp/trim")
    }

    pub fn bus_config_color(n1: u8) -> String {
        format!("/bus/{n1:02}/config/color")
    }

    pub fn bus_config_icon(n1: u8) -> String {
        format!("/bus/{n1:02}/config/icon")
    }

    pub fn bus_config_name(n1: u8) -> String {
        format!("/bus/{n1:02}/config/name")
    }

    pub fn bus_dyn_attack(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/attack")
    }

    pub fn bus_dyn_auto(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/auto")
    }

    pub fn bus_dyn_det(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/det")
    }

    pub fn bus_dyn_env(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/env")
    }

    pub fn bus_dyn_filter_f(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/filter/f")
    }

    pub fn bus_dyn_filter_on(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/filter/on")
    }

    pub fn bus_dyn_filter_type(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/filter/type")
    }

    pub fn bus_dyn_hold(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/hold")
    }

    pub fn bus_dyn_keysrc(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/keysrc")
    }

    pub fn bus_dyn_knee(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/knee")
    }

    pub fn bus_dyn_mgain(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/mgain")
    }

    pub fn bus_dyn_mix(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/mix")
    }

    pub fn bus_dyn_mode(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/mode")
    }

    pub fn bus_dyn_on(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/on")
    }

    pub fn bus_dyn_pos(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/pos")
    }

    pub fn bus_dyn_ratio(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/ratio")
    }

    pub fn bus_dyn_release(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/release")
    }

    pub fn bus_dyn_thr(n1: u8) -> String {
        format!("/bus/{n1:02}/dyn/thr")
    }

    pub fn bus_eq_f(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/f")
    }

    pub fn bus_eq_g(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/g")
    }

    pub fn bus_eq_on(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/on")
    }

    pub fn bus_eq_q(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/q")
    }

    pub fn bus_eq_type(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/eq/{n2:02}/type")
    }

    pub fn bus_eq_on_2(n1: u8) -> String {
        format!("/bus/{n1:02}/eq/on")
    }

    pub fn bus_grp_dca(n1: u8) -> String {
        format!("/bus/{n1:02}/grp/dca")
    }

    pub fn bus_grp_mute(n1: u8) -> String {
        format!("/bus/{n1:02}/grp/mute")
    }

    pub fn bus_insert_on(n1: u8) -> String {
        format!("/bus/{n1:02}/insert/on")
    }

    pub fn bus_insert_pos(n1: u8) -> String {
        format!("/bus/{n1:02}/insert/pos")
    }

    pub fn bus_insert_sel(n1: u8) -> String {
        format!("/bus/{n1:02}/insert/sel")
    }

    pub fn bus_mix_level(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/level")
    }

    pub fn bus_mix_on(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/on")
    }

    pub fn bus_mix_pan(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/pan")
    }

    pub fn bus_mix_pan_follow(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/panFollow")
    }

    pub fn bus_mix_type(n1: u8, n2: u8) -> String {
        format!("/bus/{n1:02}/mix/{n2:02}/type")
    }

    pub fn bus_mix_fader(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/fader")
    }

    pub fn bus_mix_mlevel(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/mlevel")
    }

    pub fn bus_mix_mono(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/mono")
    }

    pub fn bus_mix_on_2(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/on")
    }

    pub fn bus_mix_pan_2(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/pan")
    }

    pub fn bus_mix_st(n1: u8) -> String {
        format!("/bus/{n1:02}/mix/st")
    }

    pub fn ch_automix_group(n1: u8) -> String {
        format!("/ch/{n1:02}/automix/group")
    }

    pub fn ch_automix_weight(n1: u8) -> String {
        format!("/ch/{n1:02}/automix/weight")
    }

    pub fn ch_config_color(n1: u8) -> String {
        format!("/ch/{n1:02}/config/color")
    }

    pub fn ch_config_icon(n1: u8) -> String {
        format!("/ch/{n1:02}/config/icon")
    }

    pub fn ch_config_name(n1: u8) -> String {
        format!("/ch/{n1:02}/config/name")
    }

    pub fn ch_config_source(n1: u8) -> String {
        format!("/ch/{n1:02}/config/source")
    }

    pub fn ch_delay_on(n1: u8) -> String {
        format!("/ch/{n1:02}/delay/on")
    }

    pub fn ch_delay_time(n1: u8) -> String {
        format!("/ch/{n1:02}/delay/time")
    }

    pub fn ch_dyn_attack(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/attack")
    }

    pub fn ch_dyn_auto(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/auto")
    }

    pub fn ch_dyn_det(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/det")
    }

    pub fn ch_dyn_env(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/env")
    }

    pub fn ch_dyn_filter_f(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/filter/f")
    }

    pub fn ch_dyn_filter_on(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/filter/on")
    }

    pub fn ch_dyn_filter_type(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/filter/type")
    }

    pub fn ch_dyn_hold(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/hold")
    }

    pub fn ch_dyn_keysrc(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/keysrc")
    }

    pub fn ch_dyn_knee(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/knee")
    }

    pub fn ch_dyn_mgain(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/mgain")
    }

    pub fn ch_dyn_mix(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/mix")
    }

    pub fn ch_dyn_mode(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/mode")
    }

    pub fn ch_dyn_on(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/on")
    }

    pub fn ch_dyn_pos(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/pos")
    }

    pub fn ch_dyn_ratio(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/ratio")
    }

    pub fn ch_dyn_release(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/release")
    }

    pub fn ch_dyn_thr(n1: u8) -> String {
        format!("/ch/{n1:02}/dyn/thr")
    }

    pub fn ch_eq_f(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/f")
    }

    pub fn ch_eq_g(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/g")
    }

    pub fn ch_eq_on(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/on")
    }

    pub fn ch_eq_q(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/q")
    }

    pub fn ch_eq_type(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/eq/{n2:02}/type")
    }

    pub fn ch_eq_on_2(n1: u8) -> String {
        format!("/ch/{n1:02}/eq/on")
    }

    pub fn ch_gate_attack(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/attack")
    }

    pub fn ch_gate_filter_f(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/filter/f")
    }

    pub fn ch_gate_filter_on(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/filter/on")
    }

    pub fn ch_gate_filter_type(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/filter/type")
    }

    pub fn ch_gate_hold(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/hold")
    }

    pub fn ch_gate_keysrc(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/keysrc")
    }

    pub fn ch_gate_mode(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/mode")
    }

    pub fn ch_gate_on(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/on")
    }

    pub fn ch_gate_range(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/range")
    }

    pub fn ch_gate_release(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/release")
    }

    pub fn ch_gate_thr(n1: u8) -> String {
        format!("/ch/{n1:02}/gate/thr")
    }

    pub fn ch_grp_dca(n1: u8) -> String {
        format!("/ch/{n1:02}/grp/dca")
    }

    pub fn ch_grp_mute(n1: u8) -> String {
        format!("/ch/{n1:02}/grp/mute")
    }

    pub fn ch_insert_on(n1: u8) -> String {
        format!("/ch/{n1:02}/insert/on")
    }

    pub fn ch_insert_pos(n1: u8) -> String {
        format!("/ch/{n1:02}/insert/pos")
    }

    pub fn ch_insert_sel(n1: u8) -> String {
        format!("/ch/{n1:02}/insert/sel")
    }

    pub fn ch_mix_level(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/mix/{n2:02}/level")
    }

    pub fn ch_mix_pan(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/mix/{n2:02}/pan")
    }

    pub fn ch_mix_pan_follow(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/mix/{n2:02}/panFollow")
    }

    pub fn ch_mix_type(n1: u8, n2: u8) -> String {
        format!("/ch/{n1:02}/mix/{n2:02}/type")
    }

    pub fn ch_mix_fader(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/fader")
    }

    pub fn ch_mix_mlevel(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/mlevel")
    }

    pub fn ch_mix_mono(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/mono")
    }

    pub fn ch_mix_on(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/on")
    }

    pub fn ch_mix_pan_2(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/pan")
    }

    pub fn ch_mix_st(n1: u8) -> String {
        format!("/ch/{n1:02}/mix/st")
    }

    pub fn ch_pream_hpon(n1: u8) -> String {
        format!("/ch/{n1:02}/pream/hpon")
    }

    pub fn ch_preamp_hpf(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/hpf")
    }

    pub fn ch_preamp_hpon(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/hpon")
    }

    pub fn ch_preamp_hpslope(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/hpslope")
    }

    pub fn ch_preamp_invert(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/invert")
    }

    pub fn ch_preamp_trim(n1: u8) -> String {
        format!("/ch/{n1:02}/preamp/trim")
    }

    pub fn config_auxlink(n1: u8) -> String {
        format!("/config/auxlink/{n1:02}")
    }

    pub fn config_buslink(n1: u8) -> String {
        format!("/config/buslink/{n1:02}")
    }

    pub fn config_chlink(n1: u8) -> String {
        format!("/config/chlink/{n1:02}")
    }

    pub fn config_dp48_broadcast() -> String {
        String::from("/config/dp48/broadcast")
    }

    pub fn config_dp48_scope() -> String {
        String::from("/config/dp48/scope")
    }

    pub fn config_fxlink(n1: u8) -> String {
        format!("/config/fxlink/{n1:02}")
    }

    pub fn config_linkcfg_dyn() -> String {
        String::from("/config/linkcfg/dyn")
    }

    pub fn config_linkcfg_eq() -> String {
        String::from("/config/linkcfg/eq")
    }

    pub fn config_linkcfg_fdrmute() -> String {
        String::from("/config/linkcfg/fdrmute")
    }

    pub fn config_linkcfg_hadly() -> String {
        String::from("/config/linkcfg/hadly")
    }

    pub fn config_mono_link() -> String {
        String::from("/config/mono/link")
    }

    pub fn config_mono_mode() -> String {
        String::from("/config/mono/mode")
    }

    pub fn config_mtxlink(n1: u8) -> String {
        format!("/config/mtxlink/{n1:02}")
    }

    pub fn config_mute(n1: u8) -> String {
        format!("/config/mute/{n1:02}")
    }

    pub fn config_osc_dest() -> String {
        String::from("/config/osc/dest")
    }

    pub fn config_osc_f() -> String {
        String::from("/config/osc/f")
    }

    pub fn config_osc_fsel() -> String {
        String::from("/config/osc/fsel")
    }

    pub fn config_osc_level() -> String {
        String::from("/config/osc/level")
    }

    pub fn config_osc_type() -> String {
        String::from("/config/osc/type")
    }

    pub fn config_routing_aes50_a(n1: u8) -> String {
        format!("/config/routing/AES50A/{n1:02}")
    }

    pub fn config_routing_aes50_b(n1: u8) -> String {
        format!("/config/routing/AES50B/{n1:02}")
    }

    pub fn config_routing_card(n1: u8) -> String {
        format!("/config/routing/CARD/{n1:02}")
    }

    pub fn config_routing_in_aux() -> String {
        String::from("/config/routing/IN/AUX")
    }

    pub fn config_routing_in(n1: u8) -> String {
        format!("/config/routing/IN/{n1:02}")
    }

    pub fn config_routing_out(n1: u8) -> String {
        format!("/config/routing/OUT/{n1:02}")
    }

    pub fn config_routing_play_aux() -> String {
        String::from("/config/routing/PLAY/AUX")
    }

    pub fn config_routing_play(n1: u8) -> String {
        format!("/config/routing/PLAY/{n1:02}")
    }

    pub fn config_routing_routswitch() -> String {
        String::from("/config/routing/routswitch")
    }

    pub fn config_solo_busmode() -> String {
        String::from("/config/solo/busmode")
    }

    pub fn config_solo_chmode() -> String {
        String::from("/config/solo/chmode")
    }

    pub fn config_solo_dcamode() -> String {
        String::from("/config/solo/dcamode")
    }

    pub fn config_solo_delay() -> String {
        String::from("/config/solo/delay")
    }

    pub fn config_solo_delaytime() -> String {
        String::from("/config/solo/delaytime")
    }

    pub fn config_solo_dim() -> String {
        String::from("/config/solo/dim")
    }

    pub fn config_solo_dimatt() -> String {
        String::from("/config/solo/dimatt")
    }

    pub fn config_solo_dimpfl() -> String {
        String::from("/config/solo/dimpfl")
    }

    pub fn config_solo_exclusive() -> String {
        String::from("/config/solo/exclusive")
    }

    pub fn config_solo_followsel() -> String {
        String::from("/config/solo/followsel")
    }

    pub fn config_solo_followsolo() -> String {
        String::from("/config/solo/followsolo")
    }

    pub fn config_solo_level() -> String {
        String::from("/config/solo/level")
    }

    pub fn config_solo_masterctrl() -> String {
        String::from("/config/solo/masterctrl")
    }

    pub fn config_solo_mono() -> String {
        String::from("/config/solo/mono")
    }

    pub fn config_solo_mute() -> String {
        String::from("/config/solo/mute")
    }

    pub fn config_solo_source() -> String {
        String::from("/config/solo/source")
    }

    pub fn config_solo_sourcetrim() -> String {
        String::from("/config/solo/sourcetrim")
    }

    pub fn config_talk_a_destmap() -> String {
        String::from("/config/talk/A/destmap")
    }

    pub fn config_talk_a_dim() -> String {
        String::from("/config/talk/A/dim")
    }

    pub fn config_talk_a_latch() -> String {
        String::from("/config/talk/A/latch")
    }

    pub fn config_talk_a_level() -> String {
        String::from("/config/talk/A/level")
    }

    pub fn config_talk_b_destmap() -> String {
        String::from("/config/talk/B/destmap")
    }

    pub fn config_talk_b_dim() -> String {
        String::from("/config/talk/B/dim")
    }

    pub fn config_talk_b_latch() -> String {
        String::from("/config/talk/B/latch")
    }

    pub fn config_talk_b_level() -> String {
        String::from("/config/talk/B/level")
    }

    pub fn config_talk_enable() -> String {
        String::from("/config/talk/enable")
    }

    pub fn config_talk_source() -> String {
        String::from("/config/talk/source")
    }

    pub fn config_tape_autoplay() -> String {
        String::from("/config/tape/autoplay")
    }

    pub fn config_tape_gain_l() -> String {
        String::from("/config/tape/gainL")
    }

    pub fn config_tape_gain_r() -> String {
        String::from("/config/tape/gainR")
    }

    pub fn config_userctrl_a_btn(n1: u8) -> String {
        format!("/config/userctrl/A/btn/{n1:02}")
    }

    pub fn config_userctrl_a_color() -> String {
        String::from("/config/userctrl/A/color")
    }

    pub fn config_userctrl_a_enc(n1: u8) -> String {
        format!("/config/userctrl/A/enc/{n1:02}")
    }

    pub fn config_userctrl_b_btn(n1: u8) -> String {
        format!("/config/userctrl/B/btn/{n1:02}")
    }

    pub fn config_userctrl_b_color() -> String {
        String::from("/config/userctrl/B/color")
    }

    pub fn config_userctrl_b_enc(n1: u8) -> String {
        format!("/config/userctrl/B/enc/{n1:02}")
    }

    pub fn config_userctrl_c_btn(n1: u8) -> String {
        format!("/config/userctrl/C/btn/{n1:02}")
    }

    pub fn config_userctrl_c_color() -> String {
        String::from("/config/userctrl/C/color")
    }

    pub fn config_userctrl_c_enc(n1: u8) -> String {
        format!("/config/userctrl/C/enc/{n1:02}")
    }

    pub fn dca_config_color(n1: u8) -> String {
        format!("/dca/{n1:02}/config/color")
    }

    pub fn dca_config_icon(n1: u8) -> String {
        format!("/dca/{n1:02}/config/icon")
    }

    pub fn dca_config_name(n1: u8) -> String {
        format!("/dca/{n1:02}/config/name")
    }

    pub fn dca_fader(n1: u8) -> String {
        format!("/dca/{n1:02}/fader")
    }

    pub fn dca_mix_fader(n1: u8) -> String {
        format!("/dca/{n1:02}/mix/fader")
    }

    pub fn dca_mix_on(n1: u8) -> String {
        format!("/dca/{n1:02}/mix/on")
    }

    pub fn dca_on(n1: u8) -> String {
        format!("/dca/{n1:02}/on")
    }

    pub fn fx_par(n1: u8, n2: u8) -> String {
        format!("/fx/{n1:02}/par/{n2:02}")
    }

    pub fn fx_source_l(n1: u8) -> String {
        format!("/fx/{n1:02}/source/l")
    }

    pub fn fx_source_r(n1: u8) -> String {
        format!("/fx/{n1:02}/source/r")
    }

    pub fn fx_type(n1: u8) -> String {
        format!("/fx/{n1:02}/type")
    }

    pub fn fxrtn_config_color(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/config/color")
    }

    pub fn fxrtn_config_icon(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/config/icon")
    }

    pub fn fxrtn_config_name(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/config/name")
    }

    pub fn fxrtn_eq_f(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/f")
    }

    pub fn fxrtn_eq_g(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/g")
    }

    pub fn fxrtn_eq_on(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/on")
    }

    pub fn fxrtn_eq_q(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/q")
    }

    pub fn fxrtn_eq_type(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/{n2:02}/type")
    }

    pub fn fxrtn_eq_on_2(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/eq/on")
    }

    pub fn fxrtn_grp_dca(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/grp/dca")
    }

    pub fn fxrtn_grp_mute(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/grp/mute")
    }

    pub fn fxrtn_mix_level(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/{n2:02}/level")
    }

    pub fn fxrtn_mix_on(n1: u8, n2: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/{n2:02}/on")
    }

    pub fn fxrtn_mix_fader(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/fader")
    }

    pub fn fxrtn_mix_mlevel(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/mlevel")
    }

    pub fn fxrtn_mix_mono(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/mono")
    }

    pub fn fxrtn_mix_on_2(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/on")
    }

    pub fn fxrtn_mix_pan(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/pan")
    }

    pub fn fxrtn_mix_st(n1: u8) -> String {
        format!("/fxrtn/{n1:02}/mix/st")
    }

    pub fn headamp_gain(n1: u8) -> String {
        format!("/headamp/{n1:02}/gain")
    }

    pub fn headamp_phatom(n1: u8) -> String {
        format!("/headamp/{n1:02}/phatom")
    }

    pub fn main_m_config_color() -> String {
        String::from("/main/m/config/color")
    }

    pub fn main_m_config_icon() -> String {
        String::from("/main/m/config/icon")
    }

    pub fn main_m_config_name() -> String {
        String::from("/main/m/config/name")
    }

    pub fn main_m_dyn_attack() -> String {
        String::from("/main/m/dyn/attack")
    }

    pub fn main_m_dyn_auto() -> String {
        String::from("/main/m/dyn/auto")
    }

    pub fn main_m_dyn_det() -> String {
        String::from("/main/m/dyn/det")
    }

    pub fn main_m_dyn_env() -> String {
        String::from("/main/m/dyn/env")
    }

    pub fn main_m_dyn_filter_f() -> String {
        String::from("/main/m/dyn/filter/f")
    }

    pub fn main_m_dyn_filter_on() -> String {
        String::from("/main/m/dyn/filter/on")
    }

    pub fn main_m_dyn_filter_type() -> String {
        String::from("/main/m/dyn/filter/type")
    }

    pub fn main_m_dyn_hold() -> String {
        String::from("/main/m/dyn/hold")
    }

    pub fn main_m_dyn_knee() -> String {
        String::from("/main/m/dyn/knee")
    }

    pub fn main_m_dyn_mgain() -> String {
        String::from("/main/m/dyn/mgain")
    }

    pub fn main_m_dyn_mix() -> String {
        String::from("/main/m/dyn/mix")
    }

    pub fn main_m_dyn_mode() -> String {
        String::from("/main/m/dyn/mode")
    }

    pub fn main_m_dyn_on() -> String {
        String::from("/main/m/dyn/on")
    }

    pub fn main_m_dyn_pos() -> String {
        String::from("/main/m/dyn/pos")
    }

    pub fn main_m_dyn_ratio() -> String {
        String::from("/main/m/dyn/ratio")
    }

    pub fn main_m_dyn_release() -> String {
        String::from("/main/m/dyn/release")
    }

    pub fn main_m_dyn_thr() -> String {
        String::from("/main/m/dyn/thr")
    }

    pub fn main_m_eq_f(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/f")
    }

    pub fn main_m_eq_g(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/g")
    }

    pub fn main_m_eq_on(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/on")
    }

    pub fn main_m_eq_q(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/q")
    }

    pub fn main_m_eq_type(n1: u8) -> String {
        format!("/main/m/eq/{n1:02}/type")
    }

    pub fn main_m_eq_on_2() -> String {
        String::from("/main/m/eq/on")
    }

    pub fn main_m_insert_on() -> String {
        String::from("/main/m/insert/on")
    }

    pub fn main_m_insert_pos() -> String {
        String::from("/main/m/insert/pos")
    }

    pub fn main_m_insert_sel() -> String {
        String::from("/main/m/insert/sel")
    }

    pub fn main_m_mix_level(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/level")
    }

    pub fn main_m_mix_on(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/on")
    }

    pub fn main_m_mix_pan(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/pan")
    }

    pub fn main_m_mix_pan_follow(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/panFollow")
    }

    pub fn main_m_mix_type(n1: u8) -> String {
        format!("/main/m/mix/{n1:02}/type")
    }

    pub fn main_m_mix_fader() -> String {
        String::from("/main/m/mix/fader")
    }

    pub fn main_m_mix_on_2() -> String {
        String::from("/main/m/mix/on")
    }

    pub fn main_st_config_color() -> String {
        String::from("/main/st/config/color")
    }

    pub fn main_st_config_icon() -> String {
        String::from("/main/st/config/icon")
    }

    pub fn main_st_config_name() -> String {
        String::from("/main/st/config/name")
    }

    pub fn main_st_dyn_attack() -> String {
        String::from("/main/st/dyn/attack")
    }

    pub fn main_st_dyn_auto() -> String {
        String::from("/main/st/dyn/auto")
    }

    pub fn main_st_dyn_det() -> String {
        String::from("/main/st/dyn/det")
    }

    pub fn main_st_dyn_env() -> String {
        String::from("/main/st/dyn/env")
    }

    pub fn main_st_dyn_filter_f() -> String {
        String::from("/main/st/dyn/filter/f")
    }

    pub fn main_st_dyn_filter_on() -> String {
        String::from("/main/st/dyn/filter/on")
    }

    pub fn main_st_dyn_filter_type() -> String {
        String::from("/main/st/dyn/filter/type")
    }

    pub fn main_st_dyn_hold() -> String {
        String::from("/main/st/dyn/hold")
    }

    pub fn main_st_dyn_knee() -> String {
        String::from("/main/st/dyn/knee")
    }

    pub fn main_st_dyn_mgain() -> String {
        String::from("/main/st/dyn/mgain")
    }

    pub fn main_st_dyn_mix() -> String {
        String::from("/main/st/dyn/mix")
    }

    pub fn main_st_dyn_mode() -> String {
        String::from("/main/st/dyn/mode")
    }

    pub fn main_st_dyn_on() -> String {
        String::from("/main/st/dyn/on")
    }

    pub fn main_st_dyn_pos() -> String {
        String::from("/main/st/dyn/pos")
    }

    pub fn main_st_dyn_ratio() -> String {
        String::from("/main/st/dyn/ratio")
    }

    pub fn main_st_dyn_release() -> String {
        String::from("/main/st/dyn/release")
    }

    pub fn main_st_dyn_thr() -> String {
        String::from("/main/st/dyn/thr")
    }

    pub fn main_st_eq_f(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/f")
    }

    pub fn main_st_eq_g(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/g")
    }

    pub fn main_st_eq_on(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/on")
    }

    pub fn main_st_eq_q(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/q")
    }

    pub fn main_st_eq_type(n1: u8) -> String {
        format!("/main/st/eq/{n1:02}/type")
    }

    pub fn main_st_eq_on_2() -> String {
        String::from("/main/st/eq/on")
    }

    pub fn main_st_insert_on() -> String {
        String::from("/main/st/insert/on")
    }

    pub fn main_st_insert_pos() -> String {
        String::from("/main/st/insert/pos")
    }

    pub fn main_st_insert_sel() -> String {
        String::from("/main/st/insert/sel")
    }

    pub fn main_st_mix_level(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/level")
    }

    pub fn main_st_mix_on(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/on")
    }

    pub fn main_st_mix_pan(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/pan")
    }

    pub fn main_st_mix_pan_follow(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/panFollow")
    }

    pub fn main_st_mix_type(n1: u8) -> String {
        format!("/main/st/mix/{n1:02}/type")
    }

    pub fn main_st_mix_fader() -> String {
        String::from("/main/st/mix/fader")
    }

    pub fn main_st_mix_on_2() -> String {
        String::from("/main/st/mix/on")
    }

    pub fn main_st_mix_pan_2() -> String {
        String::from("/main/st/mix/pan")
    }

    pub fn mtx_config_color(n1: u8) -> String {
        format!("/mtx/{n1:02}/config/color")
    }

    pub fn mtx_config_icon(n1: u8) -> String {
        format!("/mtx/{n1:02}/config/icon")
    }

    pub fn mtx_config_name(n1: u8) -> String {
        format!("/mtx/{n1:02}/config/name")
    }

    pub fn mtx_dyn_attack(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/attack")
    }

    pub fn mtx_dyn_auto(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/auto")
    }

    pub fn mtx_dyn_det(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/det")
    }

    pub fn mtx_dyn_env(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/env")
    }

    pub fn mtx_dyn_filter_f(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/filter/f")
    }

    pub fn mtx_dyn_filter_on(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/filter/on")
    }

    pub fn mtx_dyn_filter_type(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/filter/type")
    }

    pub fn mtx_dyn_hold(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/hold")
    }

    pub fn mtx_dyn_knee(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/knee")
    }

    pub fn mtx_dyn_mgain(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/mgain")
    }

    pub fn mtx_dyn_mix(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/mix")
    }

    pub fn mtx_dyn_mode(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/mode")
    }

    pub fn mtx_dyn_on(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/on")
    }

    pub fn mtx_dyn_pos(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/pos")
    }

    pub fn mtx_dyn_ratio(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/ratio")
    }

    pub fn mtx_dyn_release(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/release")
    }

    pub fn mtx_dyn_thr(n1: u8) -> String {
        format!("/mtx/{n1:02}/dyn/thr")
    }

    pub fn mtx_eq_f(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/f")
    }

    pub fn mtx_eq_g(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/g")
    }

    pub fn mtx_eq_on(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/on")
    }

    pub fn mtx_eq_q(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/q")
    }

    pub fn mtx_eq_type(n1: u8, n2: u8) -> String {
        format!("/mtx/{n1:02}/eq/{n2:02}/type")
    }

    pub fn mtx_eq_on_2(n1: u8) -> String {
        format!("/mtx/{n1:02}/eq/on")
    }

    pub fn mtx_insert_on(n1: u8) -> String {
        format!("/mtx/{n1:02}/insert/on")
    }

    pub fn mtx_insert_pos(n1: u8) -> String {
        format!("/mtx/{n1:02}/insert/pos")
    }

    pub fn mtx_insert_sel(n1: u8) -> String {
        format!("/mtx/{n1:02}/insert/sel")
    }

    pub fn mtx_mix_fader(n1: u8) -> String {
        format!("/mtx/{n1:02}/mix/fader")
    }

    pub fn mtx_mix_on(n1: u8) -> String {
        format!("/mtx/{n1:02}/mix/on")
    }

    pub fn mtx_preamp(n1: u8) -> String {
        format!("/mtx/{n1:02}/preamp")
    }

    pub fn outputs_aes_invert(n1: u8) -> String {
        format!("/outputs/aes/{n1:02}/invert")
    }

    pub fn outputs_aes_pos(n1: u8) -> String {
        format!("/outputs/aes/{n1:02}/pos")
    }

    pub fn outputs_aes_src(n1: u8) -> String {
        format!("/outputs/aes/{n1:02}/src")
    }

    pub fn outputs_aux_invert(n1: u8) -> String {
        format!("/outputs/aux/{n1:02}/invert")
    }

    pub fn outputs_aux_pos(n1: u8) -> String {
        format!("/outputs/aux/{n1:02}/pos")
    }

    pub fn outputs_aux_src(n1: u8) -> String {
        format!("/outputs/aux/{n1:02}/src")
    }

    pub fn outputs_main_delay_on(n1: u8) -> String {
        format!("/outputs/main/{n1:02}/delay/on")
    }

    pub fn outputs_main_invert(n1: u8) -> String {
        format!("/outputs/main/{n1:02}/invert")
    }

    pub fn outputs_main_pos(n1: u8) -> String {
        format!("/outputs/main/{n1:02}/pos")
    }

    pub fn outputs_main_src(n1: u8) -> String {
        format!("/outputs/main/{n1:02}/src")
    }

    pub fn outputs_p16_i_q_eq(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/iQ/eq")
    }

    pub fn outputs_p16_i_q_group(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/iQ/group")
    }

    pub fn outputs_p16_i_q_model(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/iQ/model")
    }

    pub fn outputs_p16_i_q_speaker(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/iQ/speaker")
    }

    pub fn outputs_p16_invert(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/invert")
    }

    pub fn outputs_p16_pos(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/pos")
    }

    pub fn outputs_p16_src(n1: u8) -> String {
        format!("/outputs/p16/{n1:02}/src")
    }

    pub fn outputs_rec_pos(n1: u8) -> String {
        format!("/outputs/rec/{n1:02}/pos")
    }

    pub fn outputs_rec_src(n1: u8) -> String {
        format!("/outputs/rec/{n1:02}/src")
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxParam {
    Par { index: u8 },
    SourceL,
    SourceR,
    Type,
}

impl FxParam {
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::Par { index } => format!("/fx/{strip:02}/par/{index:02}"),
            Self::SourceL => format!("/fx/{strip:02}/source/l"),
            Self::SourceR => format!("/fx/{strip:02}/source/r"),
            Self::Type => format!("/fx/{strip:02}/type"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadampParam {
    Gain,
    Phatom,
}

impl HeadampParam {
    pub fn path(&self, strip: u8) -> String {
        match self {
            Self::Gain => format!("/headamp/{strip:02}/gain"),
            Self::Phatom => format!("/headamp/{strip:02}/phatom"),
        }
    }
}

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
