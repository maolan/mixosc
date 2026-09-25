use crate::model::{FaderTarget, GainSource, MixerModel};
use crate::state::StatusApp;
use maolan_widgets::iced::Color;

pub(crate) fn x32_color_to_rgb(value: u8) -> Color {
    match value {
        1 => Color::from_rgb8(0xFF, 0x45, 0x45),
        2 => Color::from_rgb8(0x32, 0xCD, 0x32),
        3 => Color::from_rgb8(0xFF, 0xD7, 0x00),
        4 => Color::from_rgb8(0x41, 0x69, 0xE1),
        5 => Color::from_rgb8(0xFF, 0x00, 0xFF),
        6 => Color::from_rgb8(0x00, 0xFF, 0xFF),
        7 => Color::from_rgb8(0xFF, 0xFF, 0xFF),
        9 => Color::from_rgb8(0xCC, 0x33, 0x33),
        10 => Color::from_rgb8(0x28, 0xA4, 0x28),
        11 => Color::from_rgb8(0xCC, 0xAC, 0x00),
        12 => Color::from_rgb8(0x33, 0x55, 0xB4),
        13 => Color::from_rgb8(0xCC, 0x00, 0xCC),
        14 => Color::from_rgb8(0x00, 0xCC, 0xCC),
        15 => Color::from_rgb8(0xDD, 0xDD, 0xDD),
        _ => Color::from_rgb8(0x3B, 0x42, 0x52),
    }
}

pub(crate) fn strip_label(target: FaderTarget) -> String {
    match target {
        FaderTarget::Channel(channel) => format!("CH {channel:02}"),
        FaderTarget::Aux(aux) => format!("AUX {aux:02}"),
        FaderTarget::Bus(bus) => format!("BUS {bus:02}"),
        FaderTarget::FxRtn(fx) => format!("FX {fx:02}"),
        FaderTarget::Mtx(mtx) => format!("MTX {mtx:02}"),
        FaderTarget::Dca(dca) => format!("DCA {dca}"),
        FaderTarget::Main => "LR".to_owned(),
    }
}

pub(crate) fn strip_name(app: &StatusApp, index: usize, target: FaderTarget) -> String {
    app.names[index]
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| strip_label(target))
}

pub(crate) fn format_fader_label(value: f32) -> String {
    if value <= 0.0 {
        return "-oo".to_owned();
    }

    format!("{:.1} dB", x32_fader_db(value))
}

pub(crate) fn format_pan_label(value: f32) -> String {
    let offset = ((value.clamp(0.0, 1.0) - 0.5) * 200.0).round() as i32;

    if offset == 0 {
        "C".to_owned()
    } else if offset < 0 {
        format!("L{}", -offset)
    } else {
        format!("R{offset}")
    }
}

pub(crate) fn gain_range(source: GainSource) -> std::ops::RangeInclusive<f32> {
    match source {
        GainSource::Headamp(_) => -12.0..=60.0,
        GainSource::Trim => -18.0..=18.0,
    }
}

pub(crate) fn gain_step(source: GainSource) -> f32 {
    match source {
        GainSource::Headamp(_) => 0.1,
        GainSource::Trim => 0.25,
    }
}

pub(crate) fn quantize_gain_value(value: f32, source: GainSource) -> f32 {
    let range = gain_range(source);
    let min = *range.start();
    let max = *range.end();
    let step = gain_step(source);
    let steps = ((value.clamp(min, max) - min) / step).round();
    (min + steps * step).clamp(min, max)
}

pub(crate) fn format_gain_label(value: f32, source: GainSource) -> String {
    match source {
        GainSource::Headamp(_) => format!("{value:+.1} dB"),
        GainSource::Trim => format!("T {value:+.1} dB"),
    }
}

pub(crate) fn linf_value(raw: f32, min: f32, max: f32) -> f32 {
    raw.clamp(0.0, 1.0) * (max - min) + min
}

pub(crate) fn logf_value(raw: f32, min: f32, max: f32) -> f32 {
    let raw = raw.clamp(0.0, 1.0);
    if min <= 0.0 || max <= 0.0 || min == max {
        return raw * (max - min) + min;
    }
    min * (max / min).powf(raw)
}

pub(crate) fn format_hz(value: f32) -> String {
    if value >= 1000.0 {
        format!("{:.2} kHz", value / 1000.0)
    } else {
        format!("{:.1} Hz", value)
    }
}

pub(crate) fn format_db(value: f32) -> String {
    format!("{value:+.2} dB")
}

pub(crate) fn format_db1(value: f32) -> String {
    format!("{value:+.1} dB")
}

pub(crate) fn format_ms(value: f32) -> String {
    if value < 1.0 {
        format!("{value:.2} ms")
    } else if value < 100.0 {
        format!("{value:.1} ms")
    } else {
        format!("{value:.0} ms")
    }
}

pub(crate) fn format_q(value: f32) -> String {
    format!("{value:.2}")
}

pub(crate) fn format_pct(value: f32) -> String {
    format!("{:.0}%", value * 100.0)
}

pub(crate) fn auxin_source_name(value: i32) -> String {
    match value {
        0 => "OFF".to_owned(),
        1..=32 => format!("In{value:02}"),
        33..=38 => format!("Aux {:02}", value - 32),
        39 => "USB L".to_owned(),
        40 => "USB R".to_owned(),
        41..=48 => {
            let names = [
                "Fx1L", "Fx1R", "Fx2L", "Fx2R", "Fx3L", "Fx3R", "Fx4L", "Fx4R",
            ];
            names
                .get((value - 41) as usize)
                .copied()
                .unwrap_or("?")
                .to_owned()
        }
        49..=64 => format!("Bus {:02}", value - 48),
        _ => format!("{value}"),
    }
}

pub(crate) fn key_source_name(value: i32) -> String {
    match value {
        0 => "Self".to_owned(),
        1..=32 => format!("Ch {value:02}"),
        33..=40 => format!("Aux {:02}", value - 32),
        41 => "USB L".to_owned(),
        42 => "USB R".to_owned(),
        43..=50 => {
            let names = [
                "Fx1L", "Fx1R", "Fx2L", "Fx2R", "Fx3L", "Fx3R", "Fx4L", "Fx4R",
            ];
            names
                .get((value - 43) as usize)
                .copied()
                .unwrap_or("?")
                .to_owned()
        }
        51..=66 => format!("Bus {:02}", value - 50),
        _ => format!("Src {value}"),
    }
}

pub(crate) fn x32_fader_db(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);

    if value >= 0.5 {
        value * 40.0 - 30.0
    } else if value >= 0.25 {
        value * 80.0 - 50.0
    } else if value >= 0.0625 {
        value * 160.0 - 70.0
    } else {
        value * 480.0 - 90.0
    }
}

pub(crate) fn x32_fader_normalized_for_db(db: f32) -> f32 {
    let db = db.clamp(-90.0, 10.0);

    if db >= -10.0 {
        (db + 30.0) / 40.0
    } else if db >= -30.0 {
        (db + 50.0) / 80.0
    } else if db >= -60.0 {
        (db + 70.0) / 160.0
    } else {
        (db + 90.0) / 480.0
    }
}

pub(crate) fn linear_meter_to_db(value: f32) -> f32 {
    let value = value.max(0.000_031_622_78);
    (20.0 * value.log10()).clamp(-90.0, 20.0)
}

pub(crate) fn strip_base_path(target: FaderTarget, model: MixerModel) -> String {
    match model {
        MixerModel::X32 => match target {
            FaderTarget::Channel(n) => format!("/ch/{n:02}"),
            FaderTarget::Aux(n) => format!("/auxin/{n:02}"),
            FaderTarget::Bus(n) => format!("/bus/{n:02}"),
            FaderTarget::FxRtn(n) => format!("/fxrtn/{n:02}"),
            FaderTarget::Mtx(n) => format!("/mtx/{n:02}"),
            FaderTarget::Dca(n) => format!("/dca/{n}"),
            FaderTarget::Main => "/main/st".to_owned(),
        },
        MixerModel::XR18 => match target {
            FaderTarget::Channel(n) => format!("/ch/{n:02}"),
            FaderTarget::Bus(n) => format!("/bus/{n}"),
            FaderTarget::FxRtn(5) => "/rtn/aux".to_owned(),
            FaderTarget::FxRtn(n) => format!("/rtn/{n}"),
            FaderTarget::Dca(n) => format!("/dca/{n}"),
            FaderTarget::Main => "/lr".to_owned(),
            _ => String::new(),
        },
    }
}

pub(crate) fn main_base_path(model: MixerModel) -> String {
    match model {
        MixerModel::X32 => "/main/st".to_owned(),
        MixerModel::XR18 => "/lr".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x32_fader_scale_inverse_matches_fader_curve() {
        for db in [-50.0, -30.0, -10.0, -5.0, 0.0, 5.0, 10.0] {
            let normalized = x32_fader_normalized_for_db(db);
            assert!((x32_fader_db(normalized) - db).abs() < 0.001);
        }
        assert!((x32_fader_normalized_for_db(0.0) - 0.75).abs() < 0.001);
    }
}
