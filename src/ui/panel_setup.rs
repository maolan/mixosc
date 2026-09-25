use crate::message::Message;
use crate::parameters::OscValue;
use crate::state::StatusApp;
use crate::ui::format::*;
use crate::ui::panel_routing::{detail_panel, top_panel_shell};
use crate::ui::widgets::*;
use maolan_widgets::iced::widget::{button, column, row, scrollable, text};
use maolan_widgets::iced::{Background, Border, Color, Element, Length, Theme};

pub(crate) fn setup_detail_panel(app: &StatusApp) -> Element<'_, Message> {
    let mut panels = row!().spacing(8);

    let talk_enable = param_bool(app, "/config/talk/enable");
    let talk_source = match app.parameter_values.get("/config/talk/source") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let talk_a_level = param_float(app, "/config/talk/A/level");
    let talk_a_latch = param_bool(app, "/config/talk/A/latch");
    let talk_a_dim = param_bool(app, "/config/talk/A/dim");
    let talk_a_destmap = match app.parameter_values.get("/config/talk/A/destmap") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let talk_b_level = param_float(app, "/config/talk/B/level");
    let talk_b_latch = param_bool(app, "/config/talk/B/latch");
    let talk_b_dim = param_bool(app, "/config/talk/B/dim");
    let talk_b_destmap = match app.parameter_values.get("/config/talk/B/destmap") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };

    let make_talk_dests = |path: String, value: i32| -> Element<'_, Message> {
        let dest_labels: [(u32, &str); 18] = [
            (1 << 0, "B1"),
            (1 << 1, "B2"),
            (1 << 2, "B3"),
            (1 << 3, "B4"),
            (1 << 4, "B5"),
            (1 << 5, "B6"),
            (1 << 6, "B7"),
            (1 << 7, "B8"),
            (1 << 8, "B9"),
            (1 << 9, "B10"),
            (1 << 10, "B11"),
            (1 << 11, "B12"),
            (1 << 12, "B13"),
            (1 << 13, "B14"),
            (1 << 14, "B15"),
            (1 << 15, "B16"),
            (1 << 16, "LR"),
            (1 << 17, "MC"),
        ];
        let rows = dest_labels
            .chunks(6)
            .fold(column!().spacing(2), |col, chunk| {
                let row = chunk.iter().fold(row!().spacing(2), |r, (mask, label)| {
                    let active = (value as u32 & mask) != 0;
                    let new_val = if active {
                        value & !(*mask as i32)
                    } else {
                        value | (*mask as i32)
                    };
                    r.push(
                        button(text(*label).size(8))
                            .on_press(Message::ParameterChanged(
                                path.clone(),
                                OscValue::Int(new_val),
                            ))
                            .padding([1, 3])
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
                                        radius: 2.0.into(),
                                    },
                                    ..Default::default()
                                },
                            ),
                    )
                });
                col.push(row)
            });
        rows.into()
    };

    let talk_col = column![
        param_toggle("Talk Enable", "/config/talk/enable".to_owned(), talk_enable),
        param_slider_labeled(
            "Talk Src",
            "/config/talk/source".to_owned(),
            talk_source as f32 / 37.0,
            |v| format!("{:.0}", v * 37.0)
        ),
        text("Talkback A")
            .size(12)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        param_slider_labeled(
            "Level",
            "/config/talk/A/level".to_owned(),
            talk_a_level,
            format_fader_label
        ),
        param_toggle("Latch", "/config/talk/A/latch".to_owned(), talk_a_latch),
        param_toggle("Dim", "/config/talk/A/dim".to_owned(), talk_a_dim),
        make_talk_dests("/config/talk/A/destmap".to_owned(), talk_a_destmap),
        text("Talkback B")
            .size(12)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        param_slider_labeled(
            "Level",
            "/config/talk/B/level".to_owned(),
            talk_b_level,
            format_fader_label
        ),
        param_toggle("Latch", "/config/talk/B/latch".to_owned(), talk_b_latch),
        param_toggle("Dim", "/config/talk/B/dim".to_owned(), talk_b_dim),
        make_talk_dests("/config/talk/B/destmap".to_owned(), talk_b_destmap),
    ]
    .spacing(5);
    panels = panels.push(detail_panel("Talkback", talk_col));

    let osc_type = match app.parameter_values.get("/config/osc/type") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let osc_f = param_float(app, "/config/osc/f");
    let osc_fsel = match app.parameter_values.get("/config/osc/fsel") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let osc_level = param_float(app, "/config/osc/level");
    let osc_dest = match app.parameter_values.get("/config/osc/dest") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };

    let osc_col = column![
        cycle_button(
            "Type",
            "/config/osc/type".to_owned(),
            osc_type,
            &["SINE", "PINK", "WHITE"]
        ),
        param_slider_labeled("Freq", "/config/osc/f".to_owned(), osc_f, |v| format_hz(
            logf_value(v, 20.0, 20000.0)
        )),
        cycle_button(
            "F Sel",
            "/config/osc/fsel".to_owned(),
            osc_fsel,
            &["F1", "F2"]
        ),
        param_slider_labeled(
            "Level",
            "/config/osc/level".to_owned(),
            osc_level,
            format_fader_label
        ),
        param_slider_labeled(
            "Dest",
            "/config/osc/dest".to_owned(),
            osc_dest as f32 / 25.0,
            |v| format!("{:.0}", v * 25.0)
        ),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Oscillator", osc_col));

    let solo_level = param_float(app, "/config/solo/level");
    let solo_source = match app.parameter_values.get("/config/solo/source") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_sourcetrim = match app.parameter_values.get("/config/solo/sourcetrim") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_chmode = match app.parameter_values.get("/config/solo/chmode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_busmode = match app.parameter_values.get("/config/solo/busmode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_dcamode = match app.parameter_values.get("/config/solo/dcamode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_exclusive = param_bool(app, "/config/solo/exclusive");
    let solo_followsel = param_bool(app, "/config/solo/followsel");
    let solo_followsolo = param_bool(app, "/config/solo/followsolo");
    let solo_dimatt = match app.parameter_values.get("/config/solo/dimatt") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_dim = param_bool(app, "/config/solo/dim");
    let solo_mono = param_bool(app, "/config/solo/mono");
    let solo_delay = param_bool(app, "/config/solo/delay");
    let solo_delaytime = match app.parameter_values.get("/config/solo/delaytime") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let solo_masterctrl = param_bool(app, "/config/solo/masterctrl");
    let solo_mute = param_bool(app, "/config/solo/mute");
    let solo_dimpfl = param_bool(app, "/config/solo/dimpfl");

    let solo_col = column![
        param_slider_labeled(
            "Level",
            "/config/solo/level".to_owned(),
            solo_level,
            format_fader_label
        ),
        param_slider_labeled(
            "Source",
            "/config/solo/source".to_owned(),
            solo_source as f32 / 37.0,
            |v| format!("{:.0}", v * 37.0)
        ),
        param_slider_labeled(
            "Trim",
            "/config/solo/sourcetrim".to_owned(),
            solo_sourcetrim as f32 / 18.0,
            |v| format!("{:.0}", v * 18.0)
        ),
        cycle_button(
            "Ch Mode",
            "/config/solo/chmode".to_owned(),
            solo_chmode,
            &["AFL", "PFL"]
        ),
        cycle_button(
            "Bus Mode",
            "/config/solo/busmode".to_owned(),
            solo_busmode,
            &["AFL", "PFL"]
        ),
        cycle_button(
            "DCA Mode",
            "/config/solo/dcamode".to_owned(),
            solo_dcamode,
            &["AFL", "PFL"]
        ),
        param_toggle(
            "Exclusive",
            "/config/solo/exclusive".to_owned(),
            solo_exclusive
        ),
        param_toggle(
            "Follow Sel",
            "/config/solo/followsel".to_owned(),
            solo_followsel
        ),
        param_toggle(
            "Follow Solo",
            "/config/solo/followsolo".to_owned(),
            solo_followsolo
        ),
        param_slider_labeled(
            "Dim Att",
            "/config/solo/dimatt".to_owned(),
            solo_dimatt as f32 / 40.0,
            |v| format!("{:.0} dB", v * 40.0)
        ),
        param_toggle("Dim", "/config/solo/dim".to_owned(), solo_dim),
        param_toggle("Mono", "/config/solo/mono".to_owned(), solo_mono),
        param_toggle("Delay", "/config/solo/delay".to_owned(), solo_delay),
        param_slider_labeled(
            "Dly Time",
            "/config/solo/delaytime".to_owned(),
            solo_delaytime as f32 / 500.0,
            |v| format_ms(v * 500.0)
        ),
        param_toggle(
            "Mst Ctrl",
            "/config/solo/masterctrl".to_owned(),
            solo_masterctrl
        ),
        param_toggle("Mute", "/config/solo/mute".to_owned(), solo_mute),
        param_toggle("Dim PFL", "/config/solo/dimpfl".to_owned(), solo_dimpfl),
    ]
    .spacing(4);
    panels = panels.push(detail_panel("Solo / Mon", solo_col));

    let sends_on_fader = param_bool(app, "/-stat/sends on fader");
    let sof_col = column![param_toggle(
        "Sends on Fdr",
        "/-stat/sends on fader".to_owned(),
        sends_on_fader
    ),]
    .spacing(6);
    panels = panels.push(detail_panel("Sends on Fdr", sof_col));

    let geq_on_fader = param_bool(app, "/-stat/geqonfdr");
    let geq_pos = match app.parameter_values.get("/-stat/geqpos") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let geq_fx_slot = (geq_pos >> 8) as u8;
    let geq_window = (geq_pos & 0xFF) as u8;
    let geq_pos_text = if geq_on_fader {
        format!("FX{} Win{}", geq_fx_slot, geq_window)
    } else {
        "—".to_owned()
    };
    let geq_col = column![
        param_toggle("GEQ on Fdr", "/-stat/geqonfdr".to_owned(), geq_on_fader),
        text(geq_pos_text)
            .size(10)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("GEQ", geq_col));

    let rec_state = match app.parameter_values.get("/-stat/urec/state") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let rec_state_name = match rec_state {
        0 => "STOP",
        1 => "PAUSE",
        2 => "PLAY",
        3 => "REC",
        _ => "?",
    };
    let rec_rtime = match app.parameter_values.get("/-stat/urec/rtime") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let rec_etime = match app.parameter_values.get("/-stat/urec/etime") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };

    let rec_col = column![
        text(format!("State: {rec_state_name}"))
            .size(12)
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        text(format!("RTime: {rec_rtime}s"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        text(format!("ETime: {rec_etime}s"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        row![
            button(text("REC").size(11))
                .on_press(Message::RecorderAction("recrun"))
                .padding([4, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x8A, 0x3A, 0x3A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0xC0, 0x5A, 0x5A),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
            button(text("STOP").size(11))
                .on_press(Message::RecorderAction("recstop"))
                .padding([4, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x3A, 0x3A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x5A, 0x5A),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(4),
        row![
            button(text("PLAY").size(11))
                .on_press(Message::RecorderAction("playrun"))
                .padding([4, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x3A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x8A, 0x5A),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
            button(text("P/STOP").size(11))
                .on_press(Message::RecorderAction("playstop"))
                .padding([4, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x3A, 0x3A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x5A, 0x5A),
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(4),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Recorder", rec_col));

    let mono_link = param_bool(app, "/config/mono/link");
    let mono_col = column![param_toggle(
        "Mono Link",
        "/config/mono/link".to_owned(),
        mono_link
    ),]
    .spacing(6);
    panels = panels.push(detail_panel("Mono Link", mono_col));

    let ip_dhcp = param_bool(app, "/-prefs/ip/dhcp");
    let clock_source = match app.parameter_values.get("/-prefs/clocksource") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let clock_rate = match app.parameter_values.get("/-prefs/clockrate") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let clock_mode = match app.parameter_values.get("/-prefs/clockmode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let net_col = column![
        param_toggle("DHCP", "/-prefs/ip/dhcp".to_owned(), ip_dhcp),
        cycle_button(
            "Clock Src",
            "/-prefs/clocksource".to_owned(),
            clock_source,
            &["Int", "AES50A", "AES50B", "Card"]
        ),
        cycle_button(
            "Clock Rate",
            "/-prefs/clockrate".to_owned(),
            clock_rate,
            &["44.1k", "48k"]
        ),
        cycle_button(
            "Clock Mode",
            "/-prefs/clockmode".to_owned(),
            clock_mode,
            &["Single", "Double", "Quad"]
        ),
    ]
    .spacing(6);
    panels = panels.push(detail_panel("Network", net_col));

    let tape_autoplay = param_bool(app, "/config/tape/autoplay");
    let tape_col = column![param_toggle(
        "Autoplay",
        "/config/tape/autoplay".to_owned(),
        tape_autoplay
    ),]
    .spacing(6);
    panels = panels.push(detail_panel("Tape", tape_col));

    let bright = param_float(app, "/-prefs/bright");
    let lcdcont = param_float(app, "/-prefs/lcdcont");
    let ledbright = param_float(app, "/-prefs/ledbright");
    let lamp = param_float(app, "/-prefs/lamp");
    let lampon = param_bool(app, "/-prefs/lampon");
    let confirm_general = param_bool(app, "/-prefs/confirm_general");
    let confirm_overwrite = param_bool(app, "/-prefs/confirm_overwrite");
    let confirm_sceneload = param_bool(app, "/-prefs/confirm_sceneload");
    let remote_enable = param_bool(app, "/-prefs/remote/enable");
    let remote_protocol = match app.parameter_values.get("/-prefs/remote/protocol") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let remote_port = match app.parameter_values.get("/-prefs/remote/port") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let card_ufifc = match app.parameter_values.get("/-prefs/card/UFifc") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let card_ufmode = match app.parameter_values.get("/-prefs/card/UFmode") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let fast_faders = param_bool(app, "/-prefs/fastFaders");
    let hard_mute = param_bool(app, "/-prefs/hardmute");
    let dca_mute = param_bool(app, "/-prefs/dcamute");
    let invert_mutes = param_bool(app, "/-prefs/invertmutes");
    let safe_master = param_bool(app, "/-prefs/safe_masterlevels");
    let view_rtn = param_bool(app, "/-prefs/viewrtn");
    let scene_advance = param_bool(app, "/-prefs/scene_advance");
    let ha_flags = match app.parameter_values.get("/-prefs/haflags") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let show_control = match app.parameter_values.get("/-prefs/show_control") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };
    let rec_control = match app.parameter_values.get("/-prefs/rec_control") {
        Some(OscValue::Int(v)) => *v,
        _ => 0,
    };

    let prefs_col = column![
        param_slider_labeled("Bright", "/-prefs/bright".to_owned(), bright, |v| format!(
            "{:.0}%",
            v * 100.0
        )),
        param_slider_labeled("LCD", "/-prefs/lcdcont".to_owned(), lcdcont, |v| format!(
            "{:.0}%",
            v * 100.0
        )),
        param_slider_labeled(
            "LED",
            "/-prefs/ledbright".to_owned(),
            ledbright,
            |v| format!("{:.0}%", v * 100.0)
        ),
        param_slider_labeled("Lamp", "/-prefs/lamp".to_owned(), lamp, |v| format!(
            "{:.0}%",
            v * 100.0
        )),
        param_toggle("Lamp On", "/-prefs/lampon".to_owned(), lampon),
        param_toggle(
            "Confirm Gen",
            "/-prefs/confirm_general".to_owned(),
            confirm_general
        ),
        param_toggle(
            "Confirm Ovw",
            "/-prefs/confirm_overwrite".to_owned(),
            confirm_overwrite
        ),
        param_toggle(
            "Confirm Scene",
            "/-prefs/confirm_sceneload".to_owned(),
            confirm_sceneload
        ),
        param_toggle(
            "Remote En",
            "/-prefs/remote/enable".to_owned(),
            remote_enable
        ),
        cycle_button(
            "Rem Proto",
            "/-prefs/remote/protocol".to_owned(),
            remote_protocol,
            &["MIDI", "OSC"]
        ),
        param_slider_labeled(
            "Rem Port",
            "/-prefs/remote/port".to_owned(),
            remote_port as f32 / 65535.0,
            |v| format!("{:.0}", v * 65535.0)
        ),
        cycle_button(
            "Card IF",
            "/-prefs/card/UFifc".to_owned(),
            card_ufifc,
            &["USB", "FW"]
        ),
        cycle_button(
            "Card Mode",
            "/-prefs/card/UFmode".to_owned(),
            card_ufmode,
            &["Player", "Rec", "Both"]
        ),
        param_toggle("Fast Faders", "/-prefs/fastFaders".to_owned(), fast_faders),
        param_toggle("Hard Mute", "/-prefs/hardmute".to_owned(), hard_mute),
        param_toggle("DCA Mute", "/-prefs/dcamute".to_owned(), dca_mute),
        param_toggle("Inv Mutes", "/-prefs/invertmutes".to_owned(), invert_mutes),
        param_toggle(
            "Safe Mst",
            "/-prefs/safe_masterlevels".to_owned(),
            safe_master
        ),
        param_toggle("View Rtn", "/-prefs/viewrtn".to_owned(), view_rtn),
        param_toggle(
            "Scene Adv",
            "/-prefs/scene_advance".to_owned(),
            scene_advance
        ),
        param_slider_labeled(
            "HA Flags",
            "/-prefs/haflags".to_owned(),
            ha_flags as f32 / 255.0,
            |v| format!("0x{:02X}", (v * 255.0) as u8)
        ),
        param_slider_labeled(
            "Show Ctrl",
            "/-prefs/show_control".to_owned(),
            show_control as f32 / 10.0,
            |v| format!("{:.0}", v * 10.0)
        ),
        param_slider_labeled(
            "Rec Ctrl",
            "/-prefs/rec_control".to_owned(),
            rec_control as f32 / 10.0,
            |v| format!("{:.0}", v * 10.0)
        ),
    ]
    .spacing(4);
    panels = panels.push(detail_panel(
        "Prefs",
        scrollable(prefs_col).height(Length::Fixed(220.0)),
    ));

    let user_assign_col = ["A", "B", "C"]
        .iter()
        .fold(column!().spacing(6), |col, layer| {
            let color_path = format!("/config/userctrl/{layer}/color");
            let color_val = match app.parameter_values.get(&color_path) {
                Some(OscValue::Int(v)) => *v,
                _ => 0,
            };
            let layer_color = match color_val {
                1 => Color::from_rgb8(0xD0, 0x40, 0x40),
                2 => Color::from_rgb8(0x40, 0xD0, 0x40),
                3 => Color::from_rgb8(0xD0, 0xD0, 0x40),
                4 => Color::from_rgb8(0x40, 0x40, 0xD0),
                5 => Color::from_rgb8(0xD0, 0x40, 0xD0),
                6 => Color::from_rgb8(0x40, 0xD0, 0xD0),
                7 => Color::from_rgb8(0xD0, 0xD0, 0xD0),
                _ => Color::from_rgb8(0x80, 0x80, 0x80),
            };
            let enc_row = (1..=4).fold(row!().spacing(2), |r, n| {
                let path = format!("/config/userctrl/{layer}/enc/{n}");
                let val = match app.parameter_values.get(&path) {
                    Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                    _ => "—".to_owned(),
                };
                r.push(
                    column![
                        text(format!("E{n}")).size(8).color(layer_color),
                        text(val).size(8).color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                    ]
                    .spacing(1)
                    .width(Length::Fixed(32.0)),
                )
            });
            let btn_row = (5..=12).fold(row!().spacing(2), |r, n| {
                let path = format!("/config/userctrl/{layer}/btn/{n}");
                let val = match app.parameter_values.get(&path) {
                    Some(OscValue::String(s)) if !s.trim().is_empty() => s.clone(),
                    _ => "—".to_owned(),
                };
                r.push(
                    column![
                        text(format!("B{n}")).size(8).color(layer_color),
                        text(val).size(8).color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
                    ]
                    .spacing(1)
                    .width(Length::Fixed(32.0)),
                )
            });
            col.push(
                column![
                    text(format!("Layer {layer}")).size(10).color(layer_color),
                    enc_row,
                    btn_row,
                ]
                .spacing(2),
            )
        });
    panels = panels.push(detail_panel(
        "User Assign",
        scrollable(user_assign_col).height(Length::Fixed(180.0)),
    ));

    top_panel_shell(panels)
}
