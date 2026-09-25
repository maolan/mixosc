use crate::message::{Message, SelectedStrip};
use crate::model::{FaderTarget, GainSource, MixerModel};
use crate::parameters::OscValue;
use crate::state::StatusApp;
use crate::ui::format::*;
use crate::ui::panel_routing::{channel_send_row, detail_panel, top_panel_shell};
use crate::ui::strips::*;
use crate::ui::widgets::*;
use maolan_widgets::horizontal_slider::horizontal_slider;
use maolan_widgets::iced::widget::{button, column, container, row, scrollable, text};
use maolan_widgets::iced::{Background, Border, Color, Element, Length, Theme};

pub(crate) fn channel_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => 0,
    };
    let target = app.visible_strips()[index];
    let pan_value = app.pans[index].unwrap_or(0.5);
    let base = strip_base_path(target, app.mixer_model);

    let sends: Element<'_, Message> = match target {
        FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => app
            .send_buses()
            .iter()
            .enumerate()
            .fold(column!().spacing(4), |column, (bus_index, bus)| {
                let send_value = app.sends[index][bus_index].unwrap_or(0.0);
                column.push(channel_send_row(index, bus_index, *bus, send_value))
            })
            .into(),
        FaderTarget::Bus(_) | FaderTarget::Main => app
            .matrix_sends()
            .iter()
            .enumerate()
            .fold(column!().spacing(4), |column, (bus_index, bus)| {
                let send_value = app.sends[index][bus_index].unwrap_or(0.0);
                column.push(channel_send_row(index, bus_index, *bus, send_value))
            })
            .into(),
        FaderTarget::Mtx(_) | FaderTarget::Dca(_) => text("No sends")
            .size(14)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D))
            .into(),
    };

    let gate_content: Element<'_, Message> = if matches!(target, FaderTarget::Channel(_)) {
        gate_summary(app, &base)
    } else {
        text("N/A")
            .size(12)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D))
            .into()
    };

    let eq_content: Element<'_, Message> = {
        let bands = eq_band_count(target);
        if bands > 0 {
            eq_summary(app, &base, bands)
        } else {
            text("N/A")
                .size(12)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D))
                .into()
        }
    };

    let dyn_content: Element<'_, Message> = if matches!(
        target,
        FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Dca(_)
    ) {
        text("N/A")
            .size(12)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D))
            .into()
    } else {
        dyn_summary(app, &base)
    };

    let gate_panel = module_summary_panel("Noise Gate", gate_content);
    let eq_panel = module_summary_panel("Equalizer", eq_content);
    let dyn_panel = module_summary_panel("Dynamics", dyn_content);
    let sends_panel = detail_panel("Bus Sends", sends);

    let groups_panel: Option<Element<'_, Message>> = if matches!(
        target,
        FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Bus(_)
    ) {
        Some(detail_panel(
            "Groups",
            column![dca_group_chips(app, &base), mute_group_chips(app, &base),].spacing(8),
        ))
    } else {
        None
    };

    let balance_panel = detail_panel(
        "Balance",
        column![
            text(format_pan_label(pan_value))
                .size(18)
                .color(Color::from_rgb8(0xE6, 0xE8, 0xEE)),
            horizontal_slider(0.0..=1.0, pan_value, move |next| Message::PanChanged(
                index, next
            ))
            .step(0.01)
            .double_click_reset(0.5)
            .width(Length::Fixed(150.0))
            .height(Length::Fixed(14.0)),
        ]
        .spacing(10)
        .align_x(maolan_widgets::iced::Alignment::Center),
    );

    let mut panels_row = row![gate_panel, eq_panel, dyn_panel, sends_panel,].spacing(2);
    if let Some(groups) = groups_panel {
        panels_row = panels_row.push(groups);
    }
    panels_row = panels_row.push(balance_panel);
    container(panels_row)
        .height(Length::Shrink)
        .width(Length::Fill)
        .into()
}

pub(crate) fn dca_group_chips<'a>(app: &'a StatusApp, base: &str) -> Element<'a, Message> {
    let path = format!("{base}/grp/dca");
    let dca_val = match app.parameter_values.get(&path) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let dca_count = app.config().dca_count;
    let chips: Element<'_, Message> = (1..=dca_count)
        .fold(row!().spacing(3), |row, dca| {
            let active = (dca_val & (1 << (dca - 1))) != 0;
            let bit = 1 << (dca - 1);
            let new_val = if active {
                dca_val & !bit
            } else {
                dca_val | bit
            };
            let (bg, border) = if active {
                (
                    Color::from_rgb8(0x3A, 0x5A, 0x3A),
                    Color::from_rgb8(0x5A, 0x8A, 0x5A),
                )
            } else {
                (
                    Color::from_rgb8(0x2A, 0x2A, 0x2C),
                    Color::from_rgb8(0x4A, 0x4A, 0x4C),
                )
            };
            row.push(
                button(text(format!("D{dca}")).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(new_val),
                    ))
                    .padding([2, 4])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(bg)),
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: border,
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        },
                    ),
            )
        })
        .into();
    column![
        text("DCA Groups")
            .size(11)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        chips,
    ]
    .spacing(4)
    .into()
}

pub(crate) fn mute_group_chips<'a>(app: &'a StatusApp, base: &str) -> Element<'a, Message> {
    let path = format!("{base}/grp/mute");
    let mute_val = match app.parameter_values.get(&path) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let mute_count = app.config().mute_group_count;
    let chips: Element<'_, Message> = (1..=mute_count)
        .fold(row!().spacing(3), |row, grp| {
            let active = (mute_val & (1 << (grp - 1))) != 0;
            let bit = 1 << (grp - 1);
            let new_val = if active {
                mute_val & !bit
            } else {
                mute_val | bit
            };
            let (bg, border) = if active {
                (
                    Color::from_rgb8(0x5A, 0x3A, 0x3A),
                    Color::from_rgb8(0x8A, 0x5A, 0x5A),
                )
            } else {
                (
                    Color::from_rgb8(0x2A, 0x2A, 0x2C),
                    Color::from_rgb8(0x4A, 0x4A, 0x4C),
                )
            };
            row.push(
                button(text(format!("M{grp}")).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(new_val),
                    ))
                    .padding([2, 4])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(bg)),
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: border,
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        },
                    ),
            )
        })
        .into();
    column![
        text("Mute Groups")
            .size(11)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        chips,
    ]
    .spacing(4)
    .into()
}

pub(crate) fn config_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return config_detail_panel_for_base(
                app,
                main_base_path(app.mixer_model),
                FaderTarget::Main,
                0,
            );
        }
    };
    let target = app.visible_strips()[index];
    let base = strip_base_path(target, app.mixer_model);
    config_detail_panel_for_base(app, base, target, index)
}

pub(crate) fn config_detail_panel_for_base<'a>(
    app: &'a StatusApp,
    base: String,
    target: FaderTarget,
    strip_index: usize,
) -> Element<'a, Message> {
    let mut panels = row!().spacing(8);

    match target {
        FaderTarget::Channel(_) | FaderTarget::Aux(_) => {
            let trim = param_float(app, &format!("{base}/preamp/trim"));
            let invert = param_bool(app, &format!("{base}/preamp/invert"));
            let mut preamp_col = column!().spacing(8);
            preamp_col = preamp_col.push(param_slider_labeled(
                "Trim",
                format!("{base}/preamp/trim"),
                trim,
                |v| format_db1(linf_value(v, -18.0, 18.0)),
            ));
            preamp_col = preamp_col.push(param_toggle(
                "Invert",
                format!("{base}/preamp/invert"),
                invert,
            ));
            if matches!(target, FaderTarget::Channel(_)) {
                if let GainSource::Headamp(idx) = app.gain_sources[strip_index] {
                    let gain = if app.mixer_model == MixerModel::X32 {
                        let gain_path = format!("/headamp/{idx:03}/gain");
                        let gain = param_float(app, &gain_path);
                        preamp_col =
                            preamp_col.push(param_slider_labeled("Gain", gain_path, gain, |v| {
                                format_db1(linf_value(v, -12.0, 60.0))
                            }));
                        gain
                    } else {
                        let gain_path = format!("/headamp/{idx:02}/gain");
                        let gain = param_float(app, &gain_path);
                        preamp_col =
                            preamp_col.push(param_slider_labeled("Gain", gain_path, gain, |v| {
                                format_db1(linf_value(v, -12.0, 20.0))
                            }));
                        gain
                    };
                    let _ = gain;
                }
                let hpon = param_bool(app, &format!("{base}/preamp/hpon"));
                let hpf = param_float(app, &format!("{base}/preamp/hpf"));
                let hpslope = match app.parameter_values.get(&format!("{base}/preamp/hpslope")) {
                    Some(OscValue::Int(v)) => *v,
                    _ => 0,
                };
                preamp_col =
                    preamp_col.push(param_toggle("HP On", format!("{base}/preamp/hpon"), hpon));
                preamp_col = preamp_col.push(param_slider_labeled(
                    "HP Freq",
                    format!("{base}/preamp/hpf"),
                    hpf,
                    |v| format_hz(logf_value(v, 20.0, 400.0)),
                ));
                preamp_col = preamp_col.push(cycle_button(
                    "Slope",
                    format!("{base}/preamp/hpslope"),
                    hpslope,
                    &["12", "18", "24"],
                ));
            }
            panels = panels.push(detail_panel("Preamp", preamp_col));
        }
        FaderTarget::Mtx(_) => {
            let invert = param_bool(app, &format!("{base}/preamp/invert"));
            let col = column![param_toggle(
                "Invert",
                format!("{base}/preamp/invert"),
                invert
            ),]
            .spacing(8);
            panels = panels.push(detail_panel("Preamp", col));
        }
        _ => {}
    }

    if matches!(target, FaderTarget::Channel(_) | FaderTarget::Aux(_)) {
        let delay_on = param_bool(app, &format!("{base}/delay/on"));
        let delay_time = param_float(app, &format!("{base}/delay/time"));
        panels = panels.push(detail_panel(
            "Delay",
            column![
                param_toggle("On", format!("{base}/delay/on"), delay_on),
                param_slider_labeled("Time", format!("{base}/delay/time"), delay_time, |v| {
                    format_ms(linf_value(v, 0.3, 500.0))
                }),
            ]
            .spacing(8),
        ));
    }

    if matches!(
        target,
        FaderTarget::Channel(_) | FaderTarget::Bus(_) | FaderTarget::Mtx(_) | FaderTarget::Main
    ) {
        let insert_on = param_bool(app, &format!("{base}/insert/on"));
        let insert_pos = param_bool(app, &format!("{base}/insert/pos"));
        let insert_sel = match app.parameter_values.get(&format!("{base}/insert/sel")) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let insert_col = column![
            param_toggle("On", format!("{base}/insert/on"), insert_on),
            param_toggle("Post", format!("{base}/insert/pos"), insert_pos),
            insert_selector(format!("{base}/insert/sel"), insert_sel),
        ]
        .spacing(8);
        let title = if target == FaderTarget::Main {
            "Insert ST"
        } else {
            "Insert"
        };
        panels = panels.push(detail_panel(title, insert_col));
    }

    if app.mixer_model == MixerModel::X32 && target == FaderTarget::Main {
        let insert_on = param_bool(app, "/main/m/insert/on");
        let insert_pos = param_bool(app, "/main/m/insert/pos");
        let insert_sel = match app.parameter_values.get("/main/m/insert/sel") {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let insert_col = column![
            param_toggle("On", "/main/m/insert/on".to_owned(), insert_on),
            param_toggle("Post", "/main/m/insert/pos".to_owned(), insert_pos),
            insert_selector("/main/m/insert/sel".to_owned(), insert_sel),
        ]
        .spacing(8);
        panels = panels.push(detail_panel("Insert M", insert_col));
    }

    if let FaderTarget::Channel(ch) = target {
        let headamp_index = match app.gain_sources[strip_index] {
            GainSource::Headamp(idx) => idx,
            _ => ch - 1,
        };
        let phantom_path = if app.mixer_model == MixerModel::X32 {
            format!("/headamp/{headamp_index:03}/phantom")
        } else {
            format!("/headamp/{headamp_index:02}/phantom")
        };
        let phantom_on = param_bool(app, &phantom_path);
        panels = panels.push(detail_panel(
            "Phantom",
            column![param_toggle("+48V", phantom_path, phantom_on)].spacing(8),
        ));
    }

    if !matches!(target, FaderTarget::Main) {
        let color_path = format!("{base}/config/color");
        let color_val = match app.parameter_values.get(&color_path) {
            Some(OscValue::Int(v)) => *v as u8,
            _ => 0,
        };
        let mut color_icon_col = column![
            text("Color")
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            color_selector(color_path, color_val),
        ]
        .spacing(4);
        if !matches!(target, FaderTarget::Dca(_)) {
            let icon_path = format!("{base}/config/icon");
            let icon_val = match app.parameter_values.get(&icon_path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            color_icon_col = color_icon_col.push(icon_selector(icon_path, icon_val));
        }
        panels = panels.push(detail_panel("Appearance", color_icon_col));
    }

    if matches!(
        target,
        FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Bus(_)
    ) {
        panels = panels.push(detail_panel(
            "Groups",
            column![dca_group_chips(app, &base), mute_group_chips(app, &base),].spacing(8),
        ));
    }

    if let FaderTarget::Channel(ch) = target
        && ch <= 8
    {
        let amix_on = param_bool(app, &format!("{base}/amix/on"));
        let amix_group = match app.parameter_values.get(&format!("{base}/amix/group")) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let amix_weight = param_float(app, &format!("{base}/amix/weight"));
        panels = panels.push(detail_panel(
            "Automix",
            column![
                param_toggle("On", format!("{base}/amix/on"), amix_on),
                cycle_button(
                    "Group",
                    format!("{base}/amix/group"),
                    amix_group,
                    &["A", "B"]
                ),
                param_slider_labeled(
                    "Weight",
                    format!("{base}/amix/weight"),
                    amix_weight,
                    |v| format!("{:.0}", v * 100.0)
                ),
            ]
            .spacing(6),
        ));
    }

    let copy_paste_col = column![
        button(text("Copy").size(10))
            .on_press(Message::CopyStrip(strip_index))
            .padding([3, 10])
            .style(
                move |_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x8A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x8A, 0xC0),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }
            ),
        button(text("Paste").size(10))
            .on_press(Message::PasteStrip(strip_index))
            .padding([3, 10])
            .style(
                move |_theme: &Theme, _status: button::Status| button::Style {
                    background: if app.copy_buffer.is_some() {
                        Some(Background::Color(Color::from_rgb8(0x3A, 0x8A, 0x5A)))
                    } else {
                        Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                    },
                    text_color: if app.copy_buffer.is_some() {
                        Color::WHITE
                    } else {
                        Color::from_rgb8(0x8E, 0x94, 0x9D)
                    },
                    border: Border {
                        color: if app.copy_buffer.is_some() {
                            Color::from_rgb8(0x5A, 0xC0, 0x5A)
                        } else {
                            Color::from_rgb8(0x4A, 0x4A, 0x4C)
                        },
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }
            ),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Utility", copy_paste_col));

    top_panel_shell(panels)
}

pub(crate) fn gate_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return top_panel_shell(row![text("Main stereo has no noise gate").size(14)]);
        }
    };
    let target = app.visible_strips()[index];

    if !matches!(target, FaderTarget::Channel(_)) {
        return top_panel_shell(row![text("No noise gate for this strip").size(14)]);
    }

    let base = strip_base_path(target, app.mixer_model);

    let on = param_bool(app, &format!("{base}/gate/on"));
    let thr = param_float(app, &format!("{base}/gate/thr"));
    let range = param_float(app, &format!("{base}/gate/range"));
    let attack = param_float(app, &format!("{base}/gate/attack"));
    let hold = param_float(app, &format!("{base}/gate/hold"));
    let release = param_float(app, &format!("{base}/gate/release"));

    let mode = match app.parameter_values.get(&format!("{base}/gate/mode")) {
        Some(OscValue::Int(v)) => *v,
        _ => 3,
    };
    let auto = param_bool(app, &format!("{base}/gate/auto"));
    let keysrc = match app.parameter_values.get(&format!("{base}/gate/keysrc")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let filter_on = param_bool(app, &format!("{base}/gate/filter/on"));
    let filter_type = match app
        .parameter_values
        .get(&format!("{base}/gate/filter/type"))
    {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let filter_f = param_float(app, &format!("{base}/gate/filter/f"));

    top_panel_shell(row![
        detail_panel(
            "Gate",
            column![
                param_toggle("On", format!("{base}/gate/on"), on),
                cycle_button(
                    "Mode",
                    format!("{base}/gate/mode"),
                    mode,
                    &["EXP2", "EXP3", "EXP4", "GATE", "DUCK"]
                ),
                param_toggle("Auto", format!("{base}/gate/auto"), auto),
                param_slider_labeled(
                    "Threshold",
                    format!("{base}/gate/thr"),
                    thr,
                    |v| format_db1(linf_value(v, -80.0, 0.0))
                ),
                param_slider_labeled(
                    "Range",
                    format!("{base}/gate/range"),
                    range,
                    |v| format_db1(linf_value(v, 3.0, 60.0))
                ),
                param_slider_labeled(
                    "Keysrc",
                    format!("{base}/gate/keysrc"),
                    keysrc as f32 / 66.0,
                    |v| key_source_name((v * 66.0) as i32)
                ),
            ]
            .spacing(6)
        ),
        detail_panel(
            "Envelope",
            column![
                param_slider_labeled("Attack", format!("{base}/gate/attack"), attack, |v| {
                    format_ms(linf_value(v, 0.0, 120.0))
                }),
                param_slider_labeled("Hold", format!("{base}/gate/hold"), hold, |v| format_ms(
                    logf_value(v, 0.02, 2000.0)
                )),
                param_slider_labeled("Release", format!("{base}/gate/release"), release, |v| {
                    format_ms(logf_value(v, 5.0, 4000.0))
                }),
            ]
            .spacing(8)
        ),
        detail_panel(
            "Filter",
            column![
                param_toggle("On", format!("{base}/gate/filter/on"), filter_on),
                cycle_button(
                    "Type",
                    format!("{base}/gate/filter/type"),
                    filter_type,
                    &[
                        "LC6", "LC12", "HC6", "HC12", "1.0", "2.0", "3.0", "5.0", "10.0"
                    ]
                ),
                param_slider_labeled("Freq", format!("{base}/gate/filter/f"), filter_f, |v| {
                    format_hz(logf_value(v, 20.0, 20000.0))
                }),
            ]
            .spacing(6)
        ),
    ])
}

pub(crate) fn dyn_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return dyn_detail_panel_for_base(app, main_base_path(app.mixer_model));
        }
    };
    let target = app.visible_strips()[index];

    if matches!(
        target,
        FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Dca(_)
    ) {
        return top_panel_shell(row![text("No dynamics for this strip").size(14)]);
    }

    let base = strip_base_path(target, app.mixer_model);
    dyn_detail_panel_for_base(app, base)
}

pub(crate) fn cycle_button(
    label: &'static str,
    path: String,
    current: i32,
    names: &[&'static str],
) -> Element<'static, Message> {
    let name = names.get(current as usize).copied().unwrap_or("?");
    let next = ((current + 1) % names.len() as i32).max(0);
    row![
        text(label)
            .size(11)
            .width(Length::Fixed(40.0))
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        button(text(name).size(11))
            .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
            .padding([2, 6])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Background::Color(Color::from_rgb8(0x2A, 0x2D, 0x33))),
                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                border: Border {
                    color: Color::from_rgb8(0x4A, 0x4D, 0x52),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
    ]
    .spacing(4)
    .align_y(maolan_widgets::iced::Alignment::Center)
    .into()
}

pub(crate) fn dyn_detail_panel_for_base<'a>(
    app: &'a StatusApp,
    base: String,
) -> Element<'a, Message> {
    let on = param_bool(app, &format!("{base}/dyn/on"));
    let thr = param_float(app, &format!("{base}/dyn/thr"));
    let ratio = param_float(app, &format!("{base}/dyn/ratio")) as i32;
    let knee = param_float(app, &format!("{base}/dyn/knee"));
    let mgain = param_float(app, &format!("{base}/dyn/mgain"));
    let attack = param_float(app, &format!("{base}/dyn/attack"));
    let hold = param_float(app, &format!("{base}/dyn/hold"));
    let release = param_float(app, &format!("{base}/dyn/release"));
    let mix = param_float(app, &format!("{base}/dyn/mix"));
    let mode = match app.parameter_values.get(&format!("{base}/dyn/mode")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let det = match app.parameter_values.get(&format!("{base}/dyn/det")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let env = match app.parameter_values.get(&format!("{base}/dyn/env")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let pos = param_bool(app, &format!("{base}/dyn/pos"));
    let keysrc = match app.parameter_values.get(&format!("{base}/dyn/keysrc")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let auto = param_bool(app, &format!("{base}/dyn/auto"));
    let filter_on = param_bool(app, &format!("{base}/dyn/filter/on"));
    let filter_type = match app.parameter_values.get(&format!("{base}/dyn/filter/type")) {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let filter_f = param_float(app, &format!("{base}/dyn/filter/f"));

    top_panel_shell(row![
        detail_panel(
            "Dynamics",
            column![
                param_toggle("On", format!("{base}/dyn/on"), on),
                cycle_button("Mode", format!("{base}/dyn/mode"), mode, &["COMP", "EXP"]),
                cycle_button("Det", format!("{base}/dyn/det"), det, &["PEAK", "RMS"]),
                cycle_button("Env", format!("{base}/dyn/env"), env, &["LIN", "LOG"]),
                param_toggle("Auto", format!("{base}/dyn/auto"), auto),
                param_slider_labeled("Threshold", format!("{base}/dyn/thr"), thr, |v| format_db1(
                    linf_value(v, -80.0, 0.0)
                )),
                cycle_button(
                    "Ratio",
                    format!("{base}/dyn/ratio"),
                    ratio,
                    &[
                        "1.1", "1.3", "1.5", "2.0", "2.5", "3.0", "4.0", "5.0", "7.0", "10", "20",
                        "100"
                    ]
                ),
                param_slider_labeled("Knee", format!("{base}/dyn/knee"), knee, |v| format!(
                    "{:.1}",
                    linf_value(v, 0.0, 5.0)
                )),
            ]
            .spacing(5)
        ),
        detail_panel(
            "Envelope",
            column![
                param_slider_labeled("Attack", format!("{base}/dyn/attack"), attack, |v| {
                    format_ms(linf_value(v, 0.0, 120.0))
                }),
                param_slider_labeled("Hold", format!("{base}/dyn/hold"), hold, |v| format_ms(
                    logf_value(v, 0.02, 2000.0)
                )),
                param_slider_labeled("Release", format!("{base}/dyn/release"), release, |v| {
                    format_ms(logf_value(v, 5.0, 4000.0))
                }),
                param_toggle("Post", format!("{base}/dyn/pos"), pos),
                param_slider_labeled(
                    "Keysrc",
                    format!("{base}/dyn/keysrc"),
                    keysrc as f32 / 66.0,
                    |v| key_source_name((v * 66.0) as i32)
                ),
            ]
            .spacing(6)
        ),
        detail_panel(
            "Output",
            column![
                param_slider_labeled("Gain", format!("{base}/dyn/mgain"), mgain, |v| format_db1(
                    linf_value(v, 0.0, 24.0)
                )),
                param_slider_labeled("Mix", format!("{base}/dyn/mix"), mix, format_pct),
            ]
            .spacing(8)
        ),
        detail_panel(
            "Filter",
            column![
                param_toggle("On", format!("{base}/dyn/filter/on"), filter_on),
                cycle_button(
                    "Type",
                    format!("{base}/dyn/filter/type"),
                    filter_type,
                    &[
                        "LC6", "LC12", "HC6", "HC12", "1.0", "2.0", "3.0", "5.0", "10.0"
                    ]
                ),
                param_slider_labeled("Freq", format!("{base}/dyn/filter/f"), filter_f, |v| {
                    format_hz(logf_value(v, 20.0, 20000.0))
                }),
            ]
            .spacing(6)
        ),
    ])
}

pub(crate) fn eq_band_count(target: FaderTarget) -> u8 {
    match target {
        FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => 4,
        FaderTarget::Bus(_) | FaderTarget::Mtx(_) | FaderTarget::Main => 6,
        FaderTarget::Dca(_) => 0,
    }
}

pub(crate) fn eq_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => {
            return eq_detail_panel_for_base(app, main_base_path(app.mixer_model), 6);
        }
    };
    let target = app.visible_strips()[index];
    let base = strip_base_path(target, app.mixer_model);
    let bands = eq_band_count(target);

    if bands == 0 {
        return top_panel_shell(row![text("No EQ for this strip").size(14)]);
    }

    eq_detail_panel_for_base(app, base, bands)
}

pub(crate) fn eq_detail_panel_for_base<'a>(
    app: &'a StatusApp,
    base: String,
    bands: u8,
) -> Element<'a, Message> {
    let eq_on = param_bool(app, &format!("{base}/eq/on"));
    let eq_toggle = param_toggle("EQ", format!("{base}/eq/on"), eq_on);

    let bands_row: Element<'_, Message> = {
        let mut row = row!().spacing(6);
        for band in 1..=bands {
            let band_base = format!("{base}/eq/{band:02}");
            let f = param_float(app, &format!("{band_base}/f"));
            let g = param_float(app, &format!("{band_base}/g"));
            let q = param_float(app, &format!("{band_base}/q"));
            let on = param_bool(app, &format!("{band_base}/on"));

            let col = column![
                text(format!("Band {band}"))
                    .size(11)
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                param_toggle("On", format!("{band_base}/on"), on),
                param_slider_labeled("Freq", format!("{band_base}/f"), f, |v| format_hz(
                    logf_value(v, 20.0, 20000.0)
                )),
                param_slider_labeled("Gain", format!("{band_base}/g"), g, |v| format_db(
                    linf_value(v, -15.0, 15.0)
                )),
                param_slider_labeled("Q", format!("{band_base}/q"), q, |v| format_q(logf_value(
                    v, 10.0, 0.3
                ))),
            ]
            .spacing(3)
            .width(Length::FillPortion(1));
            row = row.push(col);
        }
        row.into()
    };

    top_panel_shell(
        column![
            row![eq_toggle, text("Equalizer").size(14)]
                .spacing(12)
                .align_y(maolan_widgets::iced::Alignment::Center),
            bands_row,
        ]
        .spacing(12),
    )
}

pub(crate) fn sends_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let selected = app.selected_strip.unwrap_or(SelectedStrip::Strip(0));
    let index = match selected {
        SelectedStrip::Strip(index) => index,
        SelectedStrip::Master => return top_panel_shell(row![text("Main sends").size(14)]),
    };
    let target = app.visible_strips()[index];
    let base = strip_base_path(target, app.mixer_model);

    let (sends, has_tap_types, has_main_lr, has_main_mono): (Vec<u8>, bool, bool, bool) =
        match target {
            FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => {
                let count = if app.mixer_model == MixerModel::X32 {
                    16
                } else {
                    6
                };
                let has_mono = app.mixer_model == MixerModel::X32;
                ((1..=count).collect(), true, true, has_mono)
            }
            FaderTarget::Bus(_) => {
                let count = if app.mixer_model == MixerModel::X32 {
                    6
                } else {
                    0
                };
                let has_mono = app.mixer_model == MixerModel::X32;
                ((1..=count).collect(), false, true, has_mono)
            }
            FaderTarget::Mtx(_) => (Vec::new(), false, true, false),
            FaderTarget::Dca(_) => (Vec::new(), false, true, false),
            FaderTarget::Main => (Vec::new(), false, true, false),
        };

    if sends.is_empty() && !has_main_lr {
        return top_panel_shell(row![text("No sends for this strip").size(14)]);
    }

    let mut panels = row!().spacing(8);

    if !sends.is_empty() {
        let send_rows: Element<'_, Message> = sends
            .chunks(4)
            .fold(column!().spacing(6), |col, chunk| {
                let row = chunk.iter().fold(row!().spacing(6), |row, send| {
                    let send_base = format!("{base}/mix/{send:02}");
                    let level = param_float(app, &format!("{send_base}/level"));
                    let on = param_bool(app, &format!("{send_base}/on"));
                    let is_odd = send % 2 == 1;
                    let pan = if is_odd {
                        param_float(app, &format!("{send_base}/pan"))
                    } else {
                        0.5
                    };
                    let (label, stereo) = match target {
                        FaderTarget::Bus(_) => {
                            let st = if is_odd {
                                param_bool(app, &format!("/mtx/{send:02}/mix/st"))
                            } else {
                                param_bool(app, &format!("/mtx/{:02}/mix/st", send - 1))
                            };
                            (format!("Mtx {send}"), st)
                        }
                        _ => {
                            let st = if is_odd {
                                let st_path = if app.mixer_model == MixerModel::X32 {
                                    format!("/bus/{send:02}/mix/st")
                                } else {
                                    format!("/bus/{send}/mix/st")
                                };
                                param_bool(app, &st_path)
                            } else {
                                let st_path = if app.mixer_model == MixerModel::X32 {
                                    format!("/bus/{:02}/mix/st", send - 1)
                                } else {
                                    format!("/bus/{}/mix/st", send - 1)
                                };
                                param_bool(app, &st_path)
                            };
                            (format!("Bus {send}"), st)
                        }
                    };
                    let mut send_col = column![
                        text(label)
                            .size(11)
                            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                        param_toggle("On", format!("{send_base}/on"), on),
                        param_slider_labeled(
                            "Level",
                            format!("{send_base}/level"),
                            level,
                            format_fader_label
                        ),
                    ]
                    .spacing(2)
                    .width(Length::Fixed(90.0));
                    if is_odd && stereo {
                        send_col = send_col.push(param_slider_labeled(
                            "Pan",
                            format!("{send_base}/pan"),
                            pan,
                            format_pan_label,
                        ));
                    }
                    row.push(send_col)
                });
                col.push(row)
            })
            .into();

        panels = panels.push(detail_panel(
            "Sends",
            scrollable(send_rows).height(Length::Fixed(200.0)),
        ));
    }

    if has_tap_types {
        let tap_count = if app.mixer_model == MixerModel::X32 {
            16
        } else {
            6
        };
        let tap_rows: Element<'_, Message> = (1..=tap_count)
            .step_by(2)
            .fold(column!().spacing(4), |col, bus| {
                let type_path = format!("{base}/mix/{bus:02}/type");
                let current_type = match app.parameter_values.get(&type_path) {
                    Some(OscValue::Int(v)) => *v,
                    _ => 4,
                };
                col.push(tap_type_selector(bus, type_path, current_type))
            })
            .into();
        panels = panels.push(detail_panel(
            "Tap Points",
            scrollable(tap_rows).height(Length::Fixed(200.0)),
        ));
    }

    if has_main_lr {
        let mut main_col = column!().spacing(8);
        if target != FaderTarget::Main {
            let lr_on = param_bool(app, &format!("{base}/mix/st"));
            let lr_fader = param_float(app, &format!("{base}/mix/fader"));
            let lr_pan = param_float(app, &format!("{base}/mix/pan"));
            main_col = main_col.push(param_toggle("LR On", format!("{base}/mix/st"), lr_on));
            main_col = main_col.push(param_slider_labeled(
                "LR Fader",
                format!("{base}/mix/fader"),
                lr_fader,
                format_fader_label,
            ));
            main_col = main_col.push(param_slider_labeled(
                "LR Pan",
                format!("{base}/mix/pan"),
                lr_pan,
                format_pan_label,
            ));
        }
        if has_main_mono
            && !matches!(
                target,
                FaderTarget::Main | FaderTarget::Mtx(_) | FaderTarget::Dca(_)
            )
        {
            let mono_on = param_bool(app, &format!("{base}/mix/mono"));
            let mono_level = param_float(app, &format!("{base}/mix/mlevel"));
            main_col = main_col.push(param_toggle("Mono On", format!("{base}/mix/mono"), mono_on));
            main_col = main_col.push(param_slider_labeled(
                "Mono Level",
                format!("{base}/mix/mlevel"),
                mono_level,
                format_fader_label,
            ));
        }
        if target == FaderTarget::Main {
            let main_base = main_base_path(app.mixer_model);
            let st_on = param_bool(app, &format!("{main_base}/mix/on"));
            let st_fader = param_float(app, &format!("{main_base}/mix/fader"));
            let st_pan = param_float(app, &format!("{main_base}/mix/pan"));
            main_col = main_col.push(param_toggle("ST On", format!("{main_base}/mix/on"), st_on));
            main_col = main_col.push(param_slider_labeled(
                "ST Fader",
                format!("{main_base}/mix/fader"),
                st_fader,
                format_fader_label,
            ));
            main_col = main_col.push(param_slider_labeled(
                "ST Pan",
                format!("{main_base}/mix/pan"),
                st_pan,
                format_pan_label,
            ));
        }
        if app.mixer_model == MixerModel::X32
            && matches!(target, FaderTarget::Main | FaderTarget::Dca(_))
        {
            let m_on = param_bool(app, "/main/m/mix/on");
            let m_fader = param_float(app, "/main/m/mix/fader");
            main_col = main_col.push(param_toggle("M On", "/main/m/mix/on".to_owned(), m_on));
            main_col = main_col.push(param_slider_labeled(
                "M Fader",
                "/main/m/mix/fader".to_owned(),
                m_fader,
                format_fader_label,
            ));
        }
        panels = panels.push(detail_panel("Main", main_col));
    }

    top_panel_shell(panels)
}
