use crate::message::{Message, SelectedStrip};
use crate::model::{FaderTarget, MixerModel};
use crate::parameters::OscValue;
use crate::state::StatusApp;
use crate::ui::format::*;
use crate::ui::panel_routing::{detail_panel, top_panel_shell};
use crate::ui::widgets::*;
use crate::ui::x32_fx::{self, FxParameter, FxParameterKind};
use maolan_widgets::horizontal_slider::horizontal_slider;
use maolan_widgets::iced::widget::{button, column, row, scrollable, text};
use maolan_widgets::iced::{Color, Element, Length};

pub(crate) fn main_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            let base = main_base_path(app.mixer_model);
            let st_on = param_bool(app, &format!("{base}/mix/on"));
            let st_fader = param_float(app, &format!("{base}/mix/fader"));
            let st_pan = param_float(app, &format!("{base}/mix/pan"));
            let mut row = row![detail_panel(
                "Main",
                column![
                    param_toggle("On", format!("{base}/mix/on"), st_on),
                    param_slider_labeled(
                        "Fader",
                        format!("{base}/mix/fader"),
                        st_fader,
                        format_fader_label
                    ),
                    param_slider_labeled(
                        "Pan",
                        format!("{base}/mix/pan"),
                        st_pan,
                        format_pan_label
                    ),
                ]
                .spacing(8)
            ),];
            if app.mixer_model == MixerModel::X32 {
                let m_on = param_bool(app, "/main/m/mix/on");
                let m_fader = param_float(app, "/main/m/mix/fader");
                row = row.push(detail_panel(
                    "Main Mono",
                    column![
                        param_toggle("On", "/main/m/mix/on".to_owned(), m_on),
                        param_slider_labeled(
                            "Fader",
                            "/main/m/mix/fader".to_owned(),
                            m_fader,
                            format_fader_label
                        ),
                    ]
                    .spacing(8),
                ));
            }
            return top_panel_shell(row);
        }
    };
    let target = app.visible_strips()[index];
    let base = strip_base_path(target, app.mixer_model);

    let mut panels = row!().spacing(8);

    match target {
        FaderTarget::Channel(_)
        | FaderTarget::Aux(_)
        | FaderTarget::FxRtn(_)
        | FaderTarget::Bus(_) => {
            let lr_on = param_bool(app, &format!("{base}/mix/st"));
            let lr_fader = param_float(app, &format!("{base}/mix/fader"));
            let lr_pan = param_float(app, &format!("{base}/mix/pan"));
            let mono_on = param_bool(app, &format!("{base}/mix/mono"));
            let mono_level = param_float(app, &format!("{base}/mix/mlevel"));

            panels = panels.push(detail_panel(
                "Main LR",
                column![
                    param_toggle("On", format!("{base}/mix/st"), lr_on),
                    param_slider_labeled(
                        "Fader",
                        format!("{base}/mix/fader"),
                        lr_fader,
                        format_fader_label
                    ),
                    param_slider_labeled(
                        "Pan",
                        format!("{base}/mix/pan"),
                        lr_pan,
                        format_pan_label
                    ),
                ]
                .spacing(8),
            ));
            panels = panels.push(detail_panel(
                "Main Mono",
                column![
                    param_toggle("On", format!("{base}/mix/mono"), mono_on),
                    param_slider_labeled(
                        "Level",
                        format!("{base}/mix/mlevel"),
                        mono_level,
                        format_fader_label
                    ),
                ]
                .spacing(8),
            ));
        }
        FaderTarget::Mtx(_) => {
            let on = param_bool(app, &format!("{base}/mix/on"));
            let fader = param_float(app, &format!("{base}/mix/fader"));
            let pan = param_float(app, &format!("{base}/mix/pan"));
            panels = panels.push(detail_panel(
                "Matrix Output",
                column![
                    param_toggle("On", format!("{base}/mix/on"), on),
                    param_slider_labeled(
                        "Fader",
                        format!("{base}/mix/fader"),
                        fader,
                        format_fader_label
                    ),
                    param_slider_labeled("Pan", format!("{base}/mix/pan"), pan, format_pan_label),
                ]
                .spacing(8),
            ));
        }
        FaderTarget::Dca(_) => {
            let on = param_bool(app, &format!("{base}/on"));
            let fader = param_float(app, &format!("{base}/fader"));
            panels = panels.push(detail_panel(
                "DCA",
                column![
                    param_toggle("On", format!("{base}/on"), on),
                    param_slider_labeled(
                        "Fader",
                        format!("{base}/fader"),
                        fader,
                        format_fader_label
                    ),
                ]
                .spacing(8),
            ));
        }
        FaderTarget::Main => unreachable!(),
    }

    top_panel_shell(panels)
}

pub(crate) fn output_routing_message(
    path: String,
    byte_offset: u8,
    new_source: i32,
    app: &StatusApp,
) -> Message {
    let current = match app.parameter_values.get(&path) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let shift = byte_offset * 8;
    let new_val = (current & !(0xFF << shift)) | ((new_source & 0xFF) << shift);
    Message::ParameterChanged(path, OscValue::Int(new_val))
}

pub(crate) fn fx_type_name(slot: u8, fx_type: i32) -> &'static str {
    if slot <= 4 {
        const FX1_4_NAMES: [&str; 61] = [
            "HALL", "AMBI", "RPLT", "ROOM", "CHAM", "PLAT", "VREV", "VRM", "GATE", "RVRS", "DLY",
            "3TAP", "4TAP", "CRS", "FLNG", "PHAS", "DIMC", "FILT", "ROTA", "PAN", "SUB", "D/RV",
            "CR/R", "FL/R", "D/CR", "D/FL", "MODD", "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES",
            "P1A", "P1A2", "PQ5", "PQ5S", "WAVD", "LIM", "CMB", "CMB2", "FAC", "FAC1M", "FAC2",
            "LEC", "LEC2", "ULC", "ULC2", "ENH2", "ENH", "EXC2", "EXC", "IMG", "EDI", "SON",
            "AMP2", "AMP", "DRV2", "DRV", "PIT2", "PIT",
        ];
        FX1_4_NAMES
            .get(fx_type as usize)
            .copied()
            .unwrap_or("UNKNOWN")
    } else {
        const FX5_8_NAMES: [&str; 34] = [
            "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES", "P1A", "P1A2", "PQ5", "PQ5S", "WAVD",
            "LIM", "FAC", "FAC1M", "FAC2", "LEC", "LEC2", "ULC", "ULC2", "ENH2", "ENH", "EXC2",
            "EXC", "IMG", "EDI", "SON", "AMP2", "AMP", "DRV2", "DRV", "PHAS", "FILT", "PAN", "SUB",
        ];
        FX5_8_NAMES
            .get(fx_type as usize)
            .copied()
            .unwrap_or("UNKNOWN")
    }
}

pub(crate) fn fx_source_name(source: i32) -> &'static str {
    const SOURCES: [&str; 18] = [
        "INS", "MIX1", "MIX2", "MIX3", "MIX4", "MIX5", "MIX6", "MIX7", "MIX8", "MIX9", "MIX10",
        "MIX11", "MIX12", "MIX13", "MIX14", "MIX15", "MIX16", "M/C",
    ];
    SOURCES.get(source as usize).copied().unwrap_or("-")
}

fn x32_fx_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let slot = app.selected_fx_slot.clamp(1, 8);
    let slots = (1..=8).fold(row!().spacing(4), |row, slot_index| {
        let fx_type = x32_fx::fx_type_name(
            slot_index,
            param_int(app, &format!("/fx/{slot_index:02}/type")),
        );
        let label = if slot_index == slot {
            format!("> FX {slot_index}\n{fx_type}")
        } else {
            format!("FX {slot_index}\n{fx_type}")
        };
        row.push(
            button(text(label).size(11))
                .on_press(Message::FxSlotSelected(slot_index))
                .padding([5, 10]),
        )
    });

    let base = format!("/fx/{slot:02}");
    let fx_type = param_int(app, &format!("{base}/type"));
    let fx_name = x32_fx::fx_type_name(slot, fx_type);
    let type_names = if slot <= 4 {
        x32_fx::FX1_4_NAMES
    } else {
        x32_fx::FX5_8_NAMES
    };
    let mut settings = row![
        column![
            text("Effect").size(11),
            enum_selector(
                format!("{base}/type"),
                fx_type,
                type_names.iter().map(|name| (*name).to_owned()).collect(),
            ),
        ]
        .width(Length::Fixed(180.0)),
    ]
    .spacing(8);

    if slot <= 4 {
        let source_names = (0..18).map(|index| fx_source_name(index).to_owned());
        settings = settings.push(
            column![
                text("Source L").size(11),
                enum_selector(
                    format!("{base}/source/l"),
                    param_int(app, &format!("{base}/source/l")),
                    source_names.clone().collect(),
                ),
            ]
            .width(Length::Fixed(130.0)),
        );
        settings = settings.push(
            column![
                text("Source R").size(11),
                enum_selector(
                    format!("{base}/source/r"),
                    param_int(app, &format!("{base}/source/r")),
                    source_names.collect(),
                ),
            ]
            .width(Length::Fixed(130.0)),
        );
    }

    let parameters: Element<'_, Message> = if let Some(definition) = x32_fx::fx_definition(fx_name)
    {
        definition
            .parameters
            .chunks(3)
            .enumerate()
            .fold(column!().spacing(8), |column, (chunk_index, chunk)| {
                let controls = chunk.iter().enumerate().fold(
                    row!().spacing(12),
                    |row, (offset, parameter)| {
                        let index = chunk_index * 3 + offset + 1;
                        row.push(x32_fx_parameter_control(app, &base, index, parameter))
                    },
                );
                column.push(controls)
            })
            .into()
    } else {
        text("No parameters available for this effect").into()
    };

    top_panel_shell(
        column![
            slots,
            settings,
            scrollable(parameters).height(Length::Fixed(280.0)),
        ]
        .spacing(10),
    )
}

fn x32_fx_parameter_control<'a>(
    app: &'a StatusApp,
    base: &str,
    index: usize,
    parameter: &'a FxParameter,
) -> Element<'a, Message> {
    let path = format!("{base}/par/{index:02}");
    let label = row![
        text(parameter.name)
            .size(11)
            .width(Length::Fixed(100.0))
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
    ]
    .spacing(4);

    match parameter.kind {
        FxParameterKind::Enum(values) => row![
            text(parameter.name)
                .size(11)
                .width(Length::Fixed(100.0))
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            enum_selector(
                path,
                param_int(app, &format!("{base}/par/{index:02}")),
                values.iter().map(|value| (*value).to_owned()).collect(),
            ),
        ]
        .spacing(4)
        .width(Length::FillPortion(1))
        .into(),
        kind => {
            let value = param_float(app, &path);
            let name = parameter.name;
            column![
                row![
                    label,
                    text(kind.format(name, value))
                        .size(11)
                        .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                ]
                .spacing(4),
                horizontal_slider(0.0..=1.0, value, move |next| {
                    Message::ParameterChanged(path.clone(), OscValue::Float(next))
                })
                .fill_from_start()
                .step(0.001)
                .width(Length::Fill)
                .height(Length::Fixed(14.0)),
            ]
            .spacing(2)
            .width(Length::FillPortion(1))
            .into()
        }
    }
}

pub(crate) fn fx_param_names(fx_name: &str) -> [&'static str; 8] {
    match fx_name {
        "HALL" | "AMBI" | "RPLT" | "ROOM" | "CHAM" | "PLAT" => [
            "Pre Delay",
            "Decay",
            "Size",
            "Damping",
            "Diffuse",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "VREV" => [
            "Pre Delay",
            "Decay",
            "Modulate",
            "Vintage",
            "Position",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "VRM" => [
            "Rvb Delay",
            "Decay",
            "Size",
            "Density",
            "ER Level",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "GATE" => [
            "Pre Delay",
            "Decay",
            "Attack",
            "Density",
            "Spread",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "RVRS" => [
            "Pre Delay",
            "Decay",
            "Rise",
            "Diffuse",
            "Spread",
            "Level",
            "Lo Cut",
            "Hi Cut",
        ],
        "DLY" => [
            "Mix", "Time", "Mode", "Factor L", "Factor R", "Feedback", "Hi Cut", "X Feed",
        ],
        "3TAP" => [
            "Time", "Gain", "Pan", "Feedback", "Lo Cut", "Hi Cut", "Tap 2", "Tap 3",
        ],
        "4TAP" => [
            "Time", "Gain", "Feedback", "Lo Cut", "Hi Cut", "Tap 2", "Tap 3", "Tap 4",
        ],
        "CRS" => [
            "Speed", "Depth L", "Depth R", "Delay L", "Delay R", "Phase", "Mod", "Mix",
        ],
        "FLNG" => [
            "Speed", "Depth L", "Depth R", "Delay L", "Delay R", "Phase", "Feed", "Mix",
        ],
        "PHAS" => [
            "Speed",
            "Depth",
            "Resonance",
            "Base",
            "Stages",
            "Mix",
            "Spacing",
            "Pole",
        ],
        "DIMC" => ["Active", "Mode", "Dry", "M1", "M2", "M3", "M4", "M5"],
        "FILT" => [
            "Speed",
            "Depth",
            "Resonance",
            "Base",
            "Mode",
            "Polarity",
            "Mix",
            "Level",
        ],
        "ROTA" => [
            "Lo Speed", "Hi Speed", "Accel", "Distance", "Balance", "Mic Dist", "Mix", "Level",
        ],
        "PAN" => [
            "Speed", "Phase", "Wave", "Depth", "Env Spd", "Env Dpth", "Pan Ctr", "Mix",
        ],
        "SUB" => [
            "Active L", "Dry L", "Oct -1 L", "Oct -2 L", "Active R", "Dry R", "Oct -1 R",
            "Oct -2 R",
        ],
        "D/RV" => [
            "Time", "Pattern", "Feedback", "X Feed", "Hi Cut", "Mix", "Reverb", "D/R Lvl",
        ],
        "CR/R" => [
            "Speed", "Depth", "Delay", "Phase", "Wave", "Balance", "Reverb", "Level",
        ],
        "FL/R" => [
            "Speed", "Depth", "Delay", "Phase", "Feed", "Balance", "Reverb", "Level",
        ],
        "D/CR" => [
            "Time", "Pattern", "Hi Cut", "Feedback", "X Feed", "Mix", "Chorus", "D/C Lvl",
        ],
        "D/FL" => [
            "Time", "Pattern", "Hi Cut", "Feedback", "X Feed", "Mix", "Flanger", "D/F Lvl",
        ],
        "MODD" => [
            "Time", "Delay", "Feed", "Lo Cut", "Hi Cut", "Mod Spd", "Mod Dpth", "Mix",
        ],
        "GEQ2" | "GEQ" | "TEQ2" | "TEQ" => [
            "Band 1", "Band 2", "Band 3", "Band 4", "Band 5", "Band 6", "Band 7", "Band 8",
        ],
        "DES2" => [
            "Lo A", "Hi A", "Lo B", "Hi B", "Voice A", "Voice B", "In Gain", "Out Gain",
        ],
        "DES" => [
            "Lo L", "Hi L", "Lo R", "Hi R", "Voice L", "Voice R", "In Gain", "Out Gain",
        ],
        "P1A" => [
            "Active",
            "Gain",
            "Lo Boost",
            "Lo Freq",
            "Mid W",
            "Mid Boost",
            "Mid Freq",
            "Hi Boost",
        ],
        "P1A2" => [
            "Act A",
            "Gain A",
            "Lo A",
            "Lo F A",
            "Mid A",
            "Mid Bst A",
            "Mid F A",
            "Hi A",
        ],
        "PQ5" => [
            "Active",
            "Gain",
            "Lo Freq",
            "Mid Boost",
            "Hi Freq",
            "Hi Boost",
            "In Gain",
            "Out Gain",
        ],
        "PQ5S" => [
            "Act A", "Gain A", "Lo F A", "Mid A", "Hi F A", "Hi Bst A", "In Gain", "Out Gain",
        ],
        "WAVD" => ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8"],
        "LIM" => [
            "In Gain",
            "Out Gain",
            "Squeeze",
            "Knee",
            "Attack",
            "Release",
            "Stereo Lk",
            "Auto Gain",
        ],
        "CMB" => ["Active", "Solo", "P1", "P2", "P3", "P4", "P5", "P6"],
        "CMB2" => ["Act A", "Solo A", "P1", "P2", "P3", "P4", "P5", "P6"],
        "FAC" | "FAC1M" => [
            "Active",
            "In Gain",
            "Threshold",
            "Time",
            "Bias",
            "Gain",
            "P7",
            "P8",
        ],
        "FAC2" => [
            "Act A",
            "In Gain A",
            "Thr A",
            "Time A",
            "Bias A",
            "Gain A",
            "P7",
            "P8",
        ],
        "LEC" => ["Active", "Gain", "Peak", "Mode", "Gain", "P6", "P7", "P8"],
        "LEC2" => [
            "Act A", "Gain A", "Peak A", "Mode A", "Gain A", "P6", "P7", "P8",
        ],
        "ULC" => [
            "Active", "In Gain", "Out Gain", "Attack", "Release", "Ratio", "P7", "P8",
        ],
        "ULC2" => [
            "Act A",
            "In Gain A",
            "Out A",
            "Att A",
            "Rel A",
            "Ratio A",
            "P7",
            "P8",
        ],
        "ENH2" => [
            "Out A",
            "Speed A",
            "Bass A",
            "Bass F A",
            "Mid A",
            "Mid F A",
            "Treble A",
            "Treble F A",
        ],
        "ENH" => [
            "Out Gain", "Speed", "Bass", "Bass F", "Mid", "Mid F", "Treble", "Treble F",
        ],
        "EXC2" => [
            "Tune A", "Peak A", "Zero A", "Timbre A", "Harm A", "Mix A", "P7", "P8",
        ],
        "EXC" => ["Tune", "Peak", "Zero", "Timbre", "Harm", "Mix", "P7", "P8"],
        "IMG" => [
            "Balance",
            "Mono Pan",
            "Stereo Pan",
            "Shv Gain",
            "Shv Freq",
            "Width",
            "Mix",
            "Level",
        ],
        "EDI" => [
            "Active", "In Gain", "Out Gain", "Attack", "Release", "Ratio", "P7", "P8",
        ],
        "SON" => ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8"],
        "AMP2" => [
            "Pre A", "Buzz A", "Punch A", "Crunch A", "Drive A", "Low A", "Mid A", "High A",
        ],
        "AMP" => [
            "Preamp", "Buzz", "Punch", "Crunch", "Drive", "Low", "Mid", "High",
        ],
        "DRV2" => [
            "Drive A", "Even A", "Odd A", "Gain A", "Lo Cut A", "Hi Cut A", "Mix A", "Level A",
        ],
        "DRV" => [
            "Drive", "Even", "Odd", "Gain", "Lo Cut", "Hi Cut", "Mix", "Level",
        ],
        "PIT2" => [
            "Semi A", "Cent A", "Delay A", "Lo Cut A", "Hi Cut A", "Mix A", "P7", "P8",
        ],
        "PIT" => [
            "Semitone", "Cent", "Delay", "Lo Cut", "Hi Cut", "Mix", "P7", "P8",
        ],
        _ => ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8"],
    }
}

pub(crate) fn fx_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    if app.mixer_model == MixerModel::X32 {
        return x32_fx_detail_panel(app);
    }

    let mut slots = row!().spacing(6);
    let fx_slots = if app.mixer_model == MixerModel::X32 {
        8
    } else {
        4
    };
    for slot in 1..=fx_slots {
        let base = format!("/fx/{slot:02}");
        let fx_type = match app.parameter_values.get(&format!("{base}/type")) {
            Some(OscValue::Int(t)) => fx_type_name(slot, *t),
            _ => "-",
        };
        let source_l = match app.parameter_values.get(&format!("{base}/source/l")) {
            Some(OscValue::Int(v)) => fx_source_name(*v),
            _ => "-",
        };
        let source_r = match app.parameter_values.get(&format!("{base}/source/r")) {
            Some(OscValue::Int(v)) => fx_source_name(*v),
            _ => "-",
        };

        let mut col = column![
            text(format!("FX {slot}"))
                .size(12)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            text(fx_type)
                .size(11)
                .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        ]
        .spacing(2)
        .width(Length::Fixed(90.0));

        if slot <= 4 {
            col = col.push(text(format!("L: {source_l}")).size(10));
            col = col.push(text(format!("R: {source_r}")).size(10));
        }

        let param_names = fx_param_names(fx_type);
        for par in 1..=8 {
            let par_path = format!("{base}/par/{par:02}");
            let par_val = param_float(app, &par_path);
            let par_name = param_names.get(par - 1).copied().unwrap_or("P?");
            col = col.push(
                row![
                    text(par_name).size(9).width(Length::Fixed(36.0)),
                    horizontal_slider(0.0..=1.0, par_val, move |v| {
                        Message::ParameterChanged(par_path.clone(), OscValue::Float(v))
                    })
                    .fill_from_start()
                    .step(0.01)
                    .width(Length::Fixed(50.0))
                    .height(Length::Fixed(12.0)),
                ]
                .spacing(2)
                .align_y(maolan_widgets::iced::Alignment::Center),
            );
        }

        slots = slots.push(col);
    }

    top_panel_shell(
        scrollable(slots).direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new(),
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_controls_for_every_x32_effect_type() {
        let mut app = StatusApp::default();
        for (slot, names) in [(1, x32_fx::FX1_4_NAMES), (5, x32_fx::FX5_8_NAMES)] {
            app.selected_fx_slot = slot;
            for effect_type in 0..names.len() {
                app.parameter_values.insert(
                    format!("/fx/{slot:02}/type"),
                    OscValue::Int(effect_type as i32),
                );
                let _ = x32_fx_detail_panel(&app);
            }
        }
    }
}
