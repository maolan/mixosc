use crate::message::Message;
use crate::model::MixerModel;
use crate::parameters::OscValue;
use crate::state::StatusApp;
use crate::ui::format::*;
use crate::ui::panel_fx::output_routing_message;
use crate::ui::widgets::*;
use maolan_widgets::horizontal_slider::horizontal_slider;
use maolan_widgets::iced::widget::{Space, button, column, container, row, scrollable, text};
use maolan_widgets::iced::{Background, Border, Color, Element, Length, Theme};

pub(crate) fn routing_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let mut panels = row!().spacing(8);

    let chlink_col: Element<'_, Message> = (1..=16)
        .fold(column!().spacing(3), |col, n| {
            let path = format!("/config/chlink/{n:02}");
            let val = match app.parameter_values.get(&path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let active = val != 0;
            let label = format!("Ch {:02}/{:02}", n * 2 - 1, n * 2);
            col.push(
                row![
                    text(label)
                        .size(10)
                        .width(Length::Fixed(60.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text(if active { "ON" } else { "OFF" }).size(10))
                        .on_press(Message::ParameterChanged(
                            path.clone(),
                            OscValue::Int(if active { 0 } else { 1 })
                        ))
                        .padding([2, 6])
                        .style(
                            move |_theme: &Theme, _status: button::Status| button::Style {
                                background: if active {
                                    Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                                } else {
                                    Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                                },
                                text_color: if active {
                                    Color::WHITE
                                } else {
                                    Color::from_rgb8(0x8E, 0x94, 0x9D)
                                },
                                border: Border {
                                    color: if active {
                                        Color::from_rgb8(0x5A, 0x8A, 0x5A)
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
                .spacing(4)
                .align_y(maolan_widgets::iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel("Ch Link", chlink_col));

    let mut link_col = column!().spacing(3);
    for n in 1..=4 {
        let path = format!("/config/auxlink/{n:02}");
        let val = match app.parameter_values.get(&path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let active = val != 0;
        let label = format!("Aux {:02}/{:02}", n * 2 - 1, n * 2);
        link_col = link_col.push(
            row![
                text(label)
                    .size(10)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                button(text(if active { "ON" } else { "OFF" }).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(if active { 0 } else { 1 })
                    ))
                    .padding([2, 6])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if active {
                                Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: if active {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x8E, 0x94, 0x9D)
                            },
                            border: Border {
                                color: if active {
                                    Color::from_rgb8(0x5A, 0x8A, 0x5A)
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
            .spacing(4)
            .align_y(maolan_widgets::iced::Alignment::Center),
        );
    }
    for n in 1..=8 {
        let path = format!("/config/buslink/{n:02}");
        let val = match app.parameter_values.get(&path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let active = val != 0;
        let label = format!("Bus {:02}/{:02}", n * 2 - 1, n * 2);
        link_col = link_col.push(
            row![
                text(label)
                    .size(10)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                button(text(if active { "ON" } else { "OFF" }).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(if active { 0 } else { 1 })
                    ))
                    .padding([2, 6])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if active {
                                Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: if active {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x8E, 0x94, 0x9D)
                            },
                            border: Border {
                                color: if active {
                                    Color::from_rgb8(0x5A, 0x8A, 0x5A)
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
            .spacing(4)
            .align_y(maolan_widgets::iced::Alignment::Center),
        );
    }
    for n in 1..=4 {
        let path = format!("/config/fxlink/{n:02}");
        let val = match app.parameter_values.get(&path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let active = val != 0;
        let label = format!("FX {:02}/{:02}", n * 2 - 1, n * 2);
        link_col = link_col.push(
            row![
                text(label)
                    .size(10)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                button(text(if active { "ON" } else { "OFF" }).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(if active { 0 } else { 1 })
                    ))
                    .padding([2, 6])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if active {
                                Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: if active {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x8E, 0x94, 0x9D)
                            },
                            border: Border {
                                color: if active {
                                    Color::from_rgb8(0x5A, 0x8A, 0x5A)
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
            .spacing(4)
            .align_y(maolan_widgets::iced::Alignment::Center),
        );
    }
    for n in 1..=3 {
        let path = format!("/config/mtxlink/{n:02}");
        let val = match app.parameter_values.get(&path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let active = val != 0;
        let label = format!("Mtx {:02}/{:02}", n * 2 - 1, n * 2);
        link_col = link_col.push(
            row![
                text(label)
                    .size(10)
                    .width(Length::Fixed(70.0))
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                button(text(if active { "ON" } else { "OFF" }).size(10))
                    .on_press(Message::ParameterChanged(
                        path.clone(),
                        OscValue::Int(if active { 0 } else { 1 })
                    ))
                    .padding([2, 6])
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: if active {
                                Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                            } else {
                                Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                            },
                            text_color: if active {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x8E, 0x94, 0x9D)
                            },
                            border: Border {
                                color: if active {
                                    Color::from_rgb8(0x5A, 0x8A, 0x5A)
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
            .spacing(4)
            .align_y(maolan_widgets::iced::Alignment::Center),
        );
    }
    panels = panels.push(detail_panel("Bus/FX/Mtx", link_col));

    let linkcfg_hadly = param_bool(app, "/config/linkcfg/hadly");
    let linkcfg_eq = param_bool(app, "/config/linkcfg/eq");
    let linkcfg_dyn = param_bool(app, "/config/linkcfg/dyn");
    let linkcfg_fdrmute = param_bool(app, "/config/linkcfg/fdrmute");

    let linkcfg_col = column![
        param_toggle("HA+Dly", "/config/linkcfg/hadly".to_owned(), linkcfg_hadly),
        param_toggle("EQ", "/config/linkcfg/eq".to_owned(), linkcfg_eq),
        param_toggle("Dyn", "/config/linkcfg/dyn".to_owned(), linkcfg_dyn),
        param_toggle(
            "Fdr+Mute",
            "/config/linkcfg/fdrmute".to_owned(),
            linkcfg_fdrmute
        ),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Link Cfg", linkcfg_col));

    let src_col: Element<'_, Message> = (1..=32)
        .fold(column!().spacing(3), |col, ch| {
            let path = format!("/ch/{ch:02}/config/source");
            let val = match app.parameter_values.get(&path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            col.push(
                row![
                    text(format!("Ch {ch:02}"))
                        .size(10)
                        .width(Length::Fixed(40.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    param_slider_labeled("Src", path, val as f32 / 64.0, |v| format!(
                        "{:.0}",
                        v * 64.0
                    )),
                ]
                .spacing(4)
                .align_y(maolan_widgets::iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel(
        "Ch Source",
        scrollable(src_col).height(Length::Fixed(200.0)),
    ));

    if app.mixer_model == MixerModel::X32 {
        let aux_in_src_col: Element<'_, Message> = (1..=8)
            .fold(column!().spacing(3), |col, n| {
                let path = format!("/auxin/{n:02}/config/source");
                let val = match app.parameter_values.get(&path) {
                    Some(OscValue::Int(v)) => *v,
                    _ => 0,
                };
                col.push(
                    row![
                        text(format!("Aux {n:02}"))
                            .size(10)
                            .width(Length::Fixed(40.0))
                            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                        param_slider_labeled("Src", path, val as f32 / 64.0, |v| {
                            auxin_source_name((v * 64.0).round() as i32)
                        }),
                    ]
                    .spacing(4)
                    .align_y(maolan_widgets::iced::Alignment::Center),
                )
            })
            .into();
        panels = panels.push(detail_panel(
            "Aux In Src",
            scrollable(aux_in_src_col).height(Length::Fixed(120.0)),
        ));
    }

    let out_routing_col: Element<'_, Message> = [
        ("/config/routing/OUT/1-4", 1u8),
        ("/config/routing/OUT/5-8", 5u8),
        ("/config/routing/OUT/9-12", 9u8),
        ("/config/routing/OUT/13-16", 13u8),
    ]
    .into_iter()
    .fold(column!().spacing(2), |col, (path, offset)| {
        let val = match app.parameter_values.get(path) {
            Some(OscValue::Int(v)) => *v,
            _ => 0,
        };
        let mut group_col = column!().spacing(1);
        for i in 0..4 {
            let out_num = offset + i;
            let src = (val >> (i * 8)) & 0xFF;
            let path = path.to_owned();
            let prev = (src - 1).max(0);
            let next = (src + 1).min(255);
            group_col = group_col.push(
                row![
                    text(format!("Out {out_num:02}"))
                        .size(9)
                        .width(Length::Fixed(40.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text("-").size(9))
                        .on_press(output_routing_message(path.clone(), i, prev, app))
                        .padding([1, 4]),
                    text(format!("{:02}", src))
                        .size(9)
                        .width(Length::Fixed(20.0))
                        .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                    button(text("+").size(9))
                        .on_press(output_routing_message(path.clone(), i, next, app))
                        .padding([1, 4]),
                ]
                .spacing(2)
                .align_y(maolan_widgets::iced::Alignment::Center),
            );
        }
        col.push(group_col)
    })
    .into();
    panels = panels.push(detail_panel(
        "Out Route",
        scrollable(out_routing_col).height(Length::Fixed(200.0)),
    ));

    let out_delay_col: Element<'_, Message> = (1..=16)
        .fold(column!().spacing(2), |col, out| {
            let on_path = format!("/outputs/main/{out:02}/delay/on");
            let time_path = format!("/outputs/main/{out:02}/delay/time");
            let on = param_bool(app, &on_path);
            let time = param_float(app, &time_path);
            col.push(
                row![
                    text(format!("Out {out:02}"))
                        .size(9)
                        .width(Length::Fixed(36.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text(if on { "ON" } else { "OFF" }).size(9))
                        .on_press(Message::ParameterChanged(on_path, OscValue::Bool(!on)))
                        .padding([1, 4])
                        .style(
                            move |_theme: &Theme, _status: button::Status| button::Style {
                                background: if on {
                                    Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A)))
                                } else {
                                    Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                                },
                                text_color: if on {
                                    Color::WHITE
                                } else {
                                    Color::from_rgb8(0x8E, 0x94, 0x9D)
                                },
                                border: Border {
                                    color: if on {
                                        Color::from_rgb8(0x5A, 0x8A, 0x5A)
                                    } else {
                                        Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                    },
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }
                        ),
                    horizontal_slider(0.0..=1.0, time, move |v| {
                        Message::ParameterChanged(time_path.clone(), OscValue::Float(v))
                    })
                    .fill_from_start()
                    .step(0.01)
                    .width(Length::Fixed(50.0))
                    .height(Length::Fixed(10.0)),
                ]
                .spacing(4)
                .align_y(maolan_widgets::iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel(
        "Out Delay",
        scrollable(out_delay_col).height(Length::Fixed(200.0)),
    ));

    let output_src_name = |src: i32| -> &'static str {
        match src {
            0 => "OFF",
            1 => "Main L",
            2 => "Main R",
            3 => "M/C",
            4..=19 => "MixBus",
            20..=25 => "Matrix",
            26..=57 => "DirOut",
            58..=65 => "DirAux",
            66..=73 => "DirFX",
            74 => "Mon L",
            75 => "Mon R",
            76 => "TB",
            _ => "?",
        }
    };

    let out_src_col: Element<'_, Message> = (1..=16)
        .fold(column!().spacing(2), |col, out| {
            let path = format!("/outputs/main/{out:02}/src");
            let val = match app.parameter_values.get(&path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let prev = (val - 1).max(0);
            let next = (val + 1).min(76);
            col.push(
                row![
                    text(format!("Out {out:02}"))
                        .size(9)
                        .width(Length::Fixed(40.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text("-").size(9))
                        .on_press(Message::ParameterChanged(path.clone(), OscValue::Int(prev)))
                        .padding([1, 4]),
                    text(output_src_name(val))
                        .size(9)
                        .width(Length::Fixed(44.0))
                        .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                    button(text("+").size(9))
                        .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
                        .padding([1, 4]),
                ]
                .spacing(2)
                .align_y(maolan_widgets::iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel(
        "Out Src",
        scrollable(out_src_col).height(Length::Fixed(200.0)),
    ));

    let aux_src_col: Element<'_, Message> = (1..=6)
        .fold(column!().spacing(2), |col, out| {
            let path = format!("/outputs/aux/{out:02}/src");
            let val = match app.parameter_values.get(&path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let prev = (val - 1).max(0);
            let next = (val + 1).min(76);
            col.push(
                row![
                    text(format!("Aux {out:02}"))
                        .size(9)
                        .width(Length::Fixed(40.0))
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    button(text("-").size(9))
                        .on_press(Message::ParameterChanged(path.clone(), OscValue::Int(prev)))
                        .padding([1, 4]),
                    text(output_src_name(val))
                        .size(9)
                        .width(Length::Fixed(44.0))
                        .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                    button(text("+").size(9))
                        .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
                        .padding([1, 4]),
                ]
                .spacing(2)
                .align_y(maolan_widgets::iced::Alignment::Center),
            )
        })
        .into();
    panels = panels.push(detail_panel(
        "Aux Src",
        scrollable(aux_src_col).height(Length::Fixed(120.0)),
    ));

    top_panel_shell(panels)
}

pub(crate) fn rta_source_name(source: i32) -> String {
    match source {
        0 => "None".to_owned(),
        1 => "Monitor".to_owned(),
        2..=33 => format!("Ch {:02}", source - 1),
        34..=41 => format!("Aux {:02}", source - 33),
        42..=49 => format!("FX {}", source - 41),
        50..=65 => format!("Bus {:02}", source - 49),
        66..=71 => format!("Mtx {}", source - 65),
        72 => "Main".to_owned(),
        73 => "Mono".to_owned(),
        _ => format!("Src {source}"),
    }
}

pub(crate) fn rta_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let max_db = 0.0f32;
    let min_db = -90.0f32;
    let range = max_db - min_db;

    let source = param_int(app, "/-prefs/rta/source");
    let source_label = rta_source_name(source);
    let prev_source = (source - 1).max(0);
    let next_source = (source + 1).min(73);

    let bars: Element<'_, Message> = app
        .rta_meters_db
        .iter()
        .enumerate()
        .fold(row!().spacing(1), |row, (_i, &db)| {
            let norm = ((db - min_db) / range).clamp(0.0, 1.0);
            let height = 180.0 * norm;
            let color = if norm > 0.75 {
                Color::from_rgb8(0xD0, 0x40, 0x40)
            } else if norm > 0.5 {
                Color::from_rgb8(0xD0, 0xA0, 0x30)
            } else {
                Color::from_rgb8(0x30, 0xA0, 0x50)
            };
            row.push(
                container(
                    Space::new()
                        .width(Length::Fixed(4.0))
                        .height(Length::Fixed(height)),
                )
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(color)),
                    ..Default::default()
                }),
            )
        })
        .into();

    let rta_gain = param_int(app, "/-prefs/rta/gain");
    let rta_autogain = param_bool(app, "/-prefs/rta/autogain");
    let rta_decay = param_float(app, "/-prefs/rta/decay");
    let rta_mode = param_int(app, "/-prefs/rta/mode");

    let controls = row![
        row![
            text("Source:")
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            button(text("-").size(11))
                .on_press(Message::ParameterChanged(
                    "/-prefs/rta/source".to_owned(),
                    OscValue::Int(prev_source),
                ))
                .padding([2, 6]),
            text(source_label)
                .size(11)
                .width(Length::Fixed(80.0))
                .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
            button(text("+").size(11))
                .on_press(Message::ParameterChanged(
                    "/-prefs/rta/source".to_owned(),
                    OscValue::Int(next_source),
                ))
                .padding([2, 6]),
        ]
        .spacing(6)
        .align_y(maolan_widgets::iced::Alignment::Center),
        Space::new().width(Length::Fixed(16.0)),
        row![
            text("Gain:")
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            button(text("-").size(11))
                .on_press(Message::ParameterChanged(
                    "/-prefs/rta/gain".to_owned(),
                    OscValue::Int((rta_gain - 6).max(0)),
                ))
                .padding([2, 6]),
            text(format!("{} dB", rta_gain))
                .size(11)
                .width(Length::Fixed(40.0))
                .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
            button(text("+").size(11))
                .on_press(Message::ParameterChanged(
                    "/-prefs/rta/gain".to_owned(),
                    OscValue::Int((rta_gain + 6).min(60)),
                ))
                .padding([2, 6]),
        ]
        .spacing(4)
        .align_y(maolan_widgets::iced::Alignment::Center),
        Space::new().width(Length::Fixed(12.0)),
        param_toggle("Auto", "/-prefs/rta/autogain".to_owned(), rta_autogain),
        Space::new().width(Length::Fixed(12.0)),
        row![
            text("Decay:")
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            horizontal_slider(0.0..=1.0, rta_decay, move |v| {
                Message::ParameterChanged("/-prefs/rta/decay".to_owned(), OscValue::Float(v))
            })
            .fill_from_start()
            .step(0.01)
            .width(Length::Fixed(60.0))
            .height(Length::Fixed(12.0)),
        ]
        .spacing(4)
        .align_y(maolan_widgets::iced::Alignment::Center),
        Space::new().width(Length::Fixed(12.0)),
        cycle_button(
            "Mode",
            "/-prefs/rta/mode".to_owned(),
            rta_mode,
            &["Bar", "Spec"]
        ),
    ]
    .spacing(4)
    .align_y(maolan_widgets::iced::Alignment::Center);

    let content = column![
        controls,
        row![
            text("20 Hz")
                .size(9)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
            Space::new().width(Length::Fill).height(Length::Fixed(1.0)),
            text("18.7 kHz")
                .size(9)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
        ]
        .spacing(4)
        .align_y(maolan_widgets::iced::Alignment::Center),
        container(bars)
            .height(Length::Fixed(200.0))
            .align_y(maolan_widgets::iced::Alignment::End)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x1A, 0x1A, 0x1C))),
                ..Default::default()
            }),
        row![
            text("-90 dB")
                .size(9)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
            Space::new().width(Length::Fill).height(Length::Fixed(1.0)),
            text("0 dB")
                .size(9)
                .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
        ]
        .spacing(4)
        .align_y(maolan_widgets::iced::Alignment::Center),
    ]
    .spacing(6)
    .align_x(maolan_widgets::iced::Alignment::Center);

    top_panel_shell(content)
}

pub(crate) fn detail_panel<'a>(
    title: &'static str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![
            text(title)
                .size(14)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            content.into(),
        ]
        .spacing(10)
        .align_x(maolan_widgets::iced::Alignment::Center),
    )
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x1A, 0x1A, 0x1C))),
        border: Border {
            color: Color::from_rgb8(0x4B, 0x4B, 0x4B),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .padding([10, 10])
    .height(Length::Fixed(220.0))
    .width(Length::Fixed(160.0))
    .into()
}

pub(crate) fn top_panel_shell<'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(content.into())
        .padding([0, 0])
        .height(Length::Shrink)
        .width(Length::Fill)
        .into()
}

pub(crate) fn channel_send_row<'a>(
    strip_index: usize,
    bus_index: usize,
    bus: u8,
    send_value: f32,
) -> Element<'a, Message> {
    row![
        text(format!("{bus:02}"))
            .size(13)
            .width(Length::Fixed(22.0))
            .color(Color::from_rgb8(0x29, 0xE6, 0xF2)),
        horizontal_slider(0.0..=1.0, send_value, move |next| {
            Message::SendChanged(strip_index, bus_index, next)
        })
        .fill_from_start()
        .step(0.01)
        .double_click_reset(0.0)
        .width(Length::Fixed(110.0))
        .height(Length::Fixed(10.0)),
    ]
    .spacing(8)
    .align_y(maolan_widgets::iced::Alignment::Center)
    .into()
}
