use crate::message::Message;
use crate::model::MixerModel;
use crate::parameters::OscValue;
use crate::state::StatusApp;
use crate::ui::panel_routing::top_panel_shell;
use crate::ui::widgets::*;
use maolan_widgets::iced::widget::{Space, button, column, row, scrollable, text, text_input};
use maolan_widgets::iced::{Background, Border, Color, Element, Length, Theme};

pub(crate) fn scenes_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    match app.mixer_model {
        MixerModel::XR18 => {
            let current_index = match app.parameter_values.get("/-snap/index") {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let current_name = match app.parameter_values.get("/-snap/name") {
                Some(OscValue::String(s)) => s.clone(),
                _ => String::new(),
            };
            let header = row![
                text(format!("Current: {current_name} (#{current_index})"))
                    .size(12)
                    .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            ]
            .spacing(8);
            let snaps_grid: Element<'_, Message> = (1..=64)
                .fold(column!().spacing(4), |mut col, snap| {
                    let name_path = format!("/-snap/{snap:02}/name");
                    let has_data_path = format!("/-snap/{snap:02}/hasdata");
                    let name = match app.parameter_values.get(&name_path) {
                        Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                        _ => format!("Snapshot {snap}"),
                    };
                    let has_data = param_bool(app, &has_data_path);
                    let name_color = if has_data {
                        Color::from_rgb8(0xC7, 0xC9, 0xD3)
                    } else {
                        Color::from_rgb8(0x60, 0x60, 0x60)
                    };
                    let is_current = snap == current_index as usize;
                    let num_color = if is_current {
                        Color::from_rgb8(0x29, 0xE6, 0xF2)
                    } else {
                        Color::from_rgb8(0x8E, 0x94, 0x9D)
                    };
                    let row = row![
                        text(format!("{snap:02}"))
                            .size(11)
                            .width(Length::Fixed(28.0))
                            .color(num_color),
                        text(name)
                            .size(11)
                            .width(Length::Fixed(140.0))
                            .color(name_color),
                        button(text("Recall").size(10))
                            .on_press(Message::SceneRecall(snap as i32))
                            .padding([2, 6])
                            .style(|_theme: &Theme, _status: button::Status| button::Style {
                                background: Some(Background::Color(Color::from_rgb8(
                                    0x2A, 0x5A, 0x3A
                                ))),
                                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                border: Border {
                                    color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }),
                        button(text("Save").size(10))
                            .on_press(Message::SceneSave(snap as i32))
                            .padding([2, 6])
                            .style(|_theme: &Theme, _status: button::Status| button::Style {
                                background: Some(Background::Color(Color::from_rgb8(
                                    0x3A, 0x3A, 0x5A
                                ))),
                                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                border: Border {
                                    color: Color::from_rgb8(0x5A, 0x5A, 0x8A),
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }),
                    ]
                    .spacing(4)
                    .align_y(maolan_widgets::iced::Alignment::Center);
                    col = col.push(row);
                    col
                })
                .into();
            top_panel_shell(
                scrollable(column![header, snaps_grid].spacing(8))
                    .direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new())),
            )
        }
        MixerModel::X32 => {
            let scenes_grid: Element<'_, Message> = (1..=100)
                .fold(column!().spacing(4), |mut col, scene| {
                    let name_path = format!("/-show/showfile/scene/{scene:03}/name");
                    let has_data_path = format!("/-show/showfile/scene/{scene:03}/hasData");
                    let name = match app.parameter_values.get(&name_path) {
                        Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                        _ => format!("Scene {scene}"),
                    };
                    let has_data = param_bool(app, &has_data_path);
                    let name_color = if has_data {
                        Color::from_rgb8(0xC7, 0xC9, 0xD3)
                    } else {
                        Color::from_rgb8(0x60, 0x60, 0x60)
                    };

                    let safes_path = format!("/-show/showfile/scene/{scene:03}/safes");
                    let safes_val = param_int(app, &safes_path);
                    let has_safes = safes_val != 0;
                    let notes = match app
                        .parameter_values
                        .get(&format!("/-show/showfile/scene/{scene:03}/notes"))
                    {
                        Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                        _ => String::new(),
                    };
                    let notes_short = if notes.len() > 20 {
                        format!("{}…", &notes[..20])
                    } else {
                        notes.clone()
                    };

                    let row = row![
                        text(format!("{scene:03}"))
                            .size(11)
                            .width(Length::Fixed(28.0))
                            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                        text(name)
                            .size(11)
                            .width(Length::Fixed(100.0))
                            .color(name_color),
                        text(notes_short)
                            .size(9)
                            .width(Length::Fixed(80.0))
                            .color(Color::from_rgb8(0x6E, 0x74, 0x7D)),
                        button(text("Recall").size(10))
                            .on_press(Message::SceneRecall(scene))
                            .padding([2, 6])
                            .style(|_theme: &Theme, _status: button::Status| button::Style {
                                background: Some(Background::Color(Color::from_rgb8(
                                    0x2A, 0x5A, 0x3A
                                ))),
                                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                border: Border {
                                    color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }),
                        button(text("Save").size(10))
                            .on_press(Message::SceneSave(scene))
                            .padding([2, 6])
                            .style(|_theme: &Theme, _status: button::Status| button::Style {
                                background: Some(Background::Color(Color::from_rgb8(
                                    0x3A, 0x3A, 0x5A
                                ))),
                                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                border: Border {
                                    color: Color::from_rgb8(0x5A, 0x5A, 0x8A),
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }),
                        button(text(if has_safes { "S!" } else { "S" }).size(10))
                            .on_press(Message::EditSceneSafes(scene))
                            .padding([2, 4])
                            .style(
                                move |_theme: &Theme, _status: button::Status| button::Style {
                                    background: if has_safes {
                                        Some(Background::Color(Color::from_rgb8(0x8A, 0x6A, 0x2A)))
                                    } else {
                                        Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                                    },
                                    text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                    border: Border {
                                        color: if has_safes {
                                            Color::from_rgb8(0xC0, 0xA0, 0x5A)
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
                    .align_y(maolan_widgets::iced::Alignment::Center);
                    col = col.push(row);

                    if app.editing_scene_safes == Some(scene) {
                        let safe_labels = [
                            (1, "TB"),
                            (2, "FX"),
                            (3, "Bus"),
                            (4, "Ch"),
                            (5, "Cfg"),
                            (6, "Pre"),
                            (7, "Out"),
                            (8, "Rte"),
                        ];
                        let safe_row =
                            safe_labels
                                .iter()
                                .fold(row!().spacing(2), |r, (bit, label)| {
                                    let active = (safes_val & (1 << bit)) != 0;
                                    let new_val = if active {
                                        safes_val & !(1 << bit)
                                    } else {
                                        safes_val | (1 << bit)
                                    };
                                    r.push(
                                        button(text(*label).size(9))
                                            .on_press(Message::ParameterChanged(
                                                safes_path.clone(),
                                                OscValue::Int(new_val),
                                            ))
                                            .padding([1, 4])
                                            .style(
                                                move |_theme: &Theme, _status: button::Status| {
                                                    button::Style {
                                                        background: if active {
                                                            Some(Background::Color(
                                                                Color::from_rgb8(0x8A, 0x6A, 0x2A),
                                                            ))
                                                        } else {
                                                            Some(Background::Color(
                                                                Color::from_rgb8(0x2A, 0x2A, 0x2C),
                                                            ))
                                                        },
                                                        text_color: if active {
                                                            Color::WHITE
                                                        } else {
                                                            Color::from_rgb8(0x8E, 0x94, 0x9D)
                                                        },
                                                        border: Border {
                                                            color: if active {
                                                                Color::from_rgb8(0xC0, 0xA0, 0x5A)
                                                            } else {
                                                                Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                                            },
                                                            width: 1.0,
                                                            radius: 2.0.into(),
                                                        },
                                                        ..Default::default()
                                                    }
                                                },
                                            ),
                                    )
                                });
                        col = col.push(safe_row);
                    }
                    col
                })
                .into();

            let cues_grid: Element<'_, Message> = (0..100)
                .fold(column!().spacing(4), |col, cue| {
                    let name_path = format!("/-show/showfile/cue/{cue:03}/name");
                    let scene_path = format!("/-show/showfile/cue/{cue:03}/scene");
                    let skip_path = format!("/-show/showfile/cue/{cue:03}/skip");
                    let name = match app.parameter_values.get(&name_path) {
                        Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                        _ => return col,
                    };
                    let scene_idx = match app.parameter_values.get(&scene_path) {
                        Some(OscValue::Int(v)) => *v,
                        _ => -1,
                    };
                    let skipped = param_bool(app, &skip_path);
                    let scene_label = if scene_idx >= 0 {
                        format!("→ Sc {scene_idx:03}")
                    } else {
                        "—".to_owned()
                    };
                    let name_color = if skipped {
                        Color::from_rgb8(0x60, 0x60, 0x60)
                    } else {
                        Color::from_rgb8(0xC7, 0xC9, 0xD3)
                    };

                    let midi_type = match app
                        .parameter_values
                        .get(&format!("/-show/showfile/cue/{cue:03}/miditype"))
                    {
                        Some(OscValue::Int(v)) => *v,
                        _ => 0,
                    };
                    let midi_label = if midi_type > 0 {
                        let midi_type_name = match midi_type {
                            1 => "PC",
                            2 => "CC",
                            3 => "Note",
                            _ => "?",
                        };
                        let midi_chan = match app
                            .parameter_values
                            .get(&format!("/-show/showfile/cue/{cue:03}/midichan"))
                        {
                            Some(OscValue::Int(v)) => *v + 1,
                            _ => 0,
                        };
                        let midi_para1 = match app
                            .parameter_values
                            .get(&format!("/-show/showfile/cue/{cue:03}/midipara1"))
                        {
                            Some(OscValue::Int(v)) => *v,
                            _ => 0,
                        };
                        format!("{midi_type_name} Ch{midi_chan} P{midi_para1}")
                    } else {
                        String::new()
                    };

                    let row = row![
                        text(format!("{cue:03}"))
                            .size(11)
                            .width(Length::Fixed(28.0))
                            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                        text(name)
                            .size(11)
                            .width(Length::Fixed(110.0))
                            .color(name_color),
                        text(scene_label)
                            .size(10)
                            .width(Length::Fixed(44.0))
                            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
                        text(midi_label)
                            .size(9)
                            .width(Length::Fixed(70.0))
                            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                        button(text("Go").size(10))
                            .on_press(Message::SceneRecall(cue))
                            .padding([2, 6])
                            .style(|_theme: &Theme, _status: button::Status| button::Style {
                                background: Some(Background::Color(Color::from_rgb8(
                                    0x2A, 0x5A, 0x3A
                                ))),
                                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                border: Border {
                                    color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }),
                    ]
                    .spacing(4)
                    .align_y(maolan_widgets::iced::Alignment::Center);
                    col.push(row)
                })
                .into();

            let snippets_grid: Element<'_, Message> = (0..100)
                .fold(column!().spacing(4), |col, snip| {
                    let name_path = format!("/-show/showfile/snippet/{snip:03}/name");
                    let has_data_path = format!("/-show/showfile/snippet/{snip:03}/hasData");
                    let name = match app.parameter_values.get(&name_path) {
                        Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                        _ => return col,
                    };
                    let has_data = param_bool(app, &has_data_path);
                    let name_color = if has_data {
                        Color::from_rgb8(0xC7, 0xC9, 0xD3)
                    } else {
                        Color::from_rgb8(0x60, 0x60, 0x60)
                    };

                    let editing = app.editing_snippet_filters == Some(snip);
                    let row = row![
                        text(format!("{snip:03}"))
                            .size(11)
                            .width(Length::Fixed(28.0))
                            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                        text(name)
                            .size(11)
                            .width(Length::Fixed(110.0))
                            .color(name_color),
                        button(text("Recall").size(10))
                            .on_press(Message::SnippetRecall(snip))
                            .padding([2, 5])
                            .style(|_theme: &Theme, _status: button::Status| button::Style {
                                background: Some(Background::Color(Color::from_rgb8(
                                    0x2A, 0x5A, 0x3A
                                ))),
                                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                border: Border {
                                    color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }),
                        button(text("Save").size(10))
                            .on_press(Message::SnippetSave(snip))
                            .padding([2, 5])
                            .style(|_theme: &Theme, _status: button::Status| button::Style {
                                background: Some(Background::Color(Color::from_rgb8(
                                    0x3A, 0x3A, 0x5A
                                ))),
                                text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                border: Border {
                                    color: Color::from_rgb8(0x5A, 0x5A, 0x8A),
                                    width: 1.0,
                                    radius: 2.0.into()
                                },
                                ..Default::default()
                            }),
                        button(text("Filt").size(10))
                            .on_press(Message::EditSnippetFilters(snip))
                            .padding([2, 5])
                            .style(
                                move |_theme: &Theme, _status: button::Status| button::Style {
                                    background: if editing {
                                        Some(Background::Color(Color::from_rgb8(0x8A, 0x6A, 0x2A)))
                                    } else {
                                        Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C)))
                                    },
                                    text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                                    border: Border {
                                        color: if editing {
                                            Color::from_rgb8(0xC0, 0xA0, 0x5A)
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
                    .align_y(maolan_widgets::iced::Alignment::Center);
                    let mut col = col.push(row);

                    if editing {
                        let eventtyp_path = format!("/-show/showfile/snippet/{snip:03}/eventtyp");
                        let eventtyp_val = match app.parameter_values.get(&eventtyp_path) {
                            Some(OscValue::Int(v)) => *v as u32,
                            _ => 0,
                        };
                        let channels_path = format!("/-show/showfile/snippet/{snip:03}/channels");
                        let channels_val = match app.parameter_values.get(&channels_path) {
                            Some(OscValue::Int(v)) => *v as u32,
                            _ => 0,
                        };
                        let auxbuses_path = format!("/-show/showfile/snippet/{snip:03}/auxbuses");
                        let auxbuses_val = match app.parameter_values.get(&auxbuses_path) {
                            Some(OscValue::Int(v)) => *v as u32,
                            _ => 0,
                        };
                        let maingrps_path = format!("/-show/showfile/snippet/{snip:03}/maingrps");
                        let maingrps_val = match app.parameter_values.get(&maingrps_path) {
                            Some(OscValue::Int(v)) => *v as u32,
                            _ => 0,
                        };

                        let make_bit_toggles =
                            |label: &'static str,
                             path: String,
                             value: u32,
                             groups: &'static [(u32, &'static str)]|
                             -> Element<'_, Message> {
                                let toggles =
                                    groups.iter().fold(row!().spacing(2), |r, (mask, name)| {
                                        let active = (value & mask) != 0;
                                        let new_val = if (value & mask) == *mask {
                                            (value & !mask) as i32
                                        } else {
                                            (value | mask) as i32
                                        };
                                        r.push(
                            button(text(*name).size(9))
                                .on_press(Message::ParameterChanged(
                                    path.clone(),
                                    OscValue::Int(new_val),
                                ))
                                .padding([1, 3])
                                .style(move |_theme: &Theme, _status: button::Status| {
                                    button::Style {
                                        background: if active {
                                            Some(Background::Color(Color::from_rgb8(
                                                0x8A, 0x6A, 0x2A,
                                            )))
                                        } else {
                                            Some(Background::Color(Color::from_rgb8(
                                                0x2A, 0x2A, 0x2C,
                                            )))
                                        },
                                        text_color: if active {
                                            Color::WHITE
                                        } else {
                                            Color::from_rgb8(0x8E, 0x94, 0x9D)
                                        },
                                        border: Border {
                                            color: if active {
                                                Color::from_rgb8(0xC0, 0xA0, 0x5A)
                                            } else {
                                                Color::from_rgb8(0x4A, 0x4A, 0x4C)
                                            },
                                            width: 1.0,
                                            radius: 2.0.into(),
                                        },
                                        ..Default::default()
                                    }
                                }),
                        )
                                    });
                                row![
                                    text(label)
                                        .size(9)
                                        .width(Length::Fixed(36.0))
                                        .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                                    toggles,
                                ]
                                .spacing(4)
                                .align_y(maolan_widgets::iced::Alignment::Center)
                                .into()
                            };

                        let eventtyp_groups: &[(u32, &str)] = &[
                            (1 << 0, "Pre"),
                            (1 << 1, "Cfg"),
                            (1 << 2, "EQ"),
                            (1 << 3, "Dyn"),
                            (1 << 4, "Ins"),
                            (1 << 5, "Grp"),
                            (1 << 6, "Fdr"),
                            (1 << 7, "Mute"),
                            (0x1F << 8, "Snd"),
                            (0xFF << 13, "FX"),
                            (1 << 22, "Solo"),
                            (1 << 23, "Rte"),
                            (1 << 24, "Out"),
                        ];
                        let channels_groups: &[(u32, &str)] = &[
                            (0xFF, "1-8"),
                            (0xFF << 8, "9-16"),
                            (0xFF << 16, "17-24"),
                            (0xFF << 24, "25-32"),
                        ];
                        let auxbuses_groups: &[(u32, &str)] = &[
                            (0xFF, "Aux"),
                            (0xFF << 8, "FxR"),
                            (0xFF << 16, "Bus1"),
                            (0xFF << 24, "Bus9"),
                        ];
                        let maingrps_groups: &[(u32, &str)] =
                            &[(0x3F, "Mtx"), (0x3 << 8, "Main"), (0xFF << 16, "DCA")];

                        col = col.push(make_bit_toggles(
                            "Evt",
                            eventtyp_path,
                            eventtyp_val,
                            eventtyp_groups,
                        ));
                        col = col.push(make_bit_toggles(
                            "Ch",
                            channels_path,
                            channels_val,
                            channels_groups,
                        ));
                        col = col.push(make_bit_toggles(
                            "Aux",
                            auxbuses_path,
                            auxbuses_val,
                            auxbuses_groups,
                        ));
                        col = col.push(make_bit_toggles(
                            "Main",
                            maingrps_path,
                            maingrps_val,
                            maingrps_groups,
                        ));
                    }
                    col
                })
                .into();

            let show_file_row = row![
                text_input("Show filename", &app.show_file_name)
                    .size(11)
                    .width(Length::Fixed(180.0))
                    .on_input(Message::ShowFileNameChanged),
                button(text("Load").size(10))
                    .on_press(Message::ShowFileLoad)
                    .padding([3, 8])
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x2A, 0x5A, 0x3A))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x4A, 0x8A, 0x5A),
                            width: 1.0,
                            radius: 2.0.into()
                        },
                        ..Default::default()
                    }),
                button(text("Save").size(10))
                    .on_press(Message::ShowFileSave)
                    .padding([3, 8])
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x3A, 0x3A, 0x5A))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x5A, 0x5A, 0x8A),
                            width: 1.0,
                            radius: 2.0.into()
                        },
                        ..Default::default()
                    }),
                Space::new().width(Length::Fixed(12.0)),
                button(text("Undo").size(10))
                    .on_press(Message::Undo)
                    .padding([3, 8])
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color::from_rgb8(0x5A, 0x5A, 0x3A))),
                        text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                        border: Border {
                            color: Color::from_rgb8(0x8A, 0x8A, 0x5A),
                            width: 1.0,
                            radius: 2.0.into()
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(6)
            .align_y(maolan_widgets::iced::Alignment::Center);

            top_panel_shell(
                column![
                    text("Show File")
                        .size(12)
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    show_file_row,
                    text("Scenes")
                        .size(12)
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    scrollable(scenes_grid).height(Length::Fixed(100.0)),
                    text("Cues")
                        .size(12)
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    scrollable(cues_grid).height(Length::Fixed(60.0)),
                    text("Snippets")
                        .size(12)
                        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
                    scrollable(snippets_grid).height(Length::Fixed(60.0)),
                ]
                .spacing(4),
            )
        }
    }
}
