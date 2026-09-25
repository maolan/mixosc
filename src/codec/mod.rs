use crate::model::{ConsoleUpdate, FaderTarget, MainMeterLevels, MixerModel, StripMeter};
use crate::net::ProbeError;

pub(crate) mod x32;
pub(crate) mod xr18;

pub(crate) fn fader_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::fader_path(target),
        MixerModel::XR18 => crate::codec::xr18::fader_path(target),
    }
}

pub(crate) fn pan_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::pan_path(target),
        MixerModel::XR18 => crate::codec::xr18::pan_path(target),
    }
}

pub(crate) fn gain_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::gain_path(target),
        MixerModel::XR18 => crate::codec::xr18::gain_path(target),
    }
}

pub(crate) fn headamp_index_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::headamp_index_path(target),
        MixerModel::XR18 => crate::codec::xr18::headamp_index_path(target),
    }
}

pub(crate) fn headamp_gain_path(index: u8, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::headamp_gain_path(index),
        MixerModel::XR18 => crate::codec::xr18::headamp_gain_path(index),
    }
}

pub(crate) fn _headamp_index_from_gain_path(path: &str, model: MixerModel) -> Option<u8> {
    match model {
        MixerModel::X32 => crate::codec::x32::headamp_index_from_gain_path(path),
        MixerModel::XR18 => crate::codec::xr18::headamp_index_from_gain_path(path),
    }
}

pub(crate) fn send_level_path(target: FaderTarget, bus: u8, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::send_level_path(target, bus),
        MixerModel::XR18 => crate::codec::xr18::send_level_path(target, bus),
    }
}

pub(crate) fn mute_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::mute_path(target),
        MixerModel::XR18 => crate::codec::xr18::mute_path(target),
    }
}

pub(crate) fn solo_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::solo_path(target),
        MixerModel::XR18 => crate::codec::xr18::solo_path(target),
    }
}

pub(crate) fn name_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::name_path(target),
        MixerModel::XR18 => crate::codec::xr18::name_path(target),
    }
}

pub(crate) fn color_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => crate::codec::x32::color_path(target),
        MixerModel::XR18 => crate::codec::xr18::color_path(target),
    }
}

pub fn parse_console_update(packet: &[u8], model: MixerModel) -> Option<ConsoleUpdate> {
    match model {
        MixerModel::X32 => crate::codec::x32::parse_console_update(packet),
        MixerModel::XR18 => crate::codec::xr18::parse_console_update(packet),
    }
}

pub fn parse_input_meter_packet(
    packet: &[u8],
    model: MixerModel,
) -> Result<Vec<StripMeter>, ProbeError> {
    match model {
        MixerModel::X32 => crate::codec::x32::parse_input_meter_packet(packet),
        MixerModel::XR18 => crate::codec::xr18::parse_input_meter_packet(packet),
    }
}

pub fn parse_main_meter_packet(
    packet: &[u8],
    model: MixerModel,
) -> Result<MainMeterLevels, ProbeError> {
    match model {
        MixerModel::X32 => crate::codec::x32::parse_main_meter_packet(packet),
        MixerModel::XR18 => crate::codec::xr18::parse_main_meter_packet(packet),
    }
}

pub fn parse_rta_meter_packet(packet: &[u8], model: MixerModel) -> Result<[f32; 100], ProbeError> {
    match model {
        MixerModel::X32 => crate::codec::x32::parse_rta_meter_packet(packet),
        MixerModel::XR18 => Err(ProbeError::Protocol("RTA not supported on XR18".to_owned())),
    }
}
