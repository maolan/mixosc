use crate::message::Message;
use crate::parameters::OscValue;
use crate::state::StatusApp;
use crate::ui::format::*;
use maolan_widgets::horizontal_slider::horizontal_slider;
use maolan_widgets::iced::widget::{
    Space, button, canvas,
    canvas::{Geometry, Path, Text},
    column, row, text,
};
use maolan_widgets::iced::{
    Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Theme, mouse,
};

pub(crate) const STRIP_METER_HEIGHT: f32 = 260.0;
pub(crate) const FADER_SCALE_WIDTH: f32 = 22.0;
pub(crate) const FADER_SCALE_GAP: f32 = 3.0;
pub(crate) const FADER_SCALE_OUTER_PAD_Y: f32 = 7.0;

#[derive(Clone, Copy)]
pub(crate) struct FaderTicks;

impl FaderTicks {
    const VALUES: [f32; 9] = [-50.0, -40.0, -30.0, -20.0, -10.0, -5.0, 0.0, 5.0, 10.0];

    fn label(db: f32) -> String {
        if db == 0.0 {
            "0".to_owned()
        } else {
            format!("{db:+.0}")
        }
    }
}

impl<Message> canvas::Program<Message> for FaderTicks {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return vec![];
        }

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let effective_height = (bounds.height - FADER_SCALE_OUTER_PAD_Y * 2.0).max(1.0);
        let tick_x = FADER_SCALE_GAP;

        for db in Self::VALUES {
            let normalized = x32_fader_normalized_for_db(db);
            let y = effective_height * (1.0 - normalized);
            let label_y = (y - 4.0).clamp(0.0, (effective_height - 10.0).max(0.0));
            frame.fill(
                &Path::rectangle(
                    Point::new(tick_x, FADER_SCALE_OUTER_PAD_Y + label_y + 4.0),
                    maolan_widgets::iced::Size::new(4.0, 1.0),
                ),
                Color::from_rgba(0.62, 0.67, 0.77, 0.78),
            );
            frame.fill_text(Text {
                content: Self::label(db),
                position: Point::new(tick_x + 6.0, FADER_SCALE_OUTER_PAD_Y + label_y),
                color: Color::from_rgba(0.9, 0.92, 0.96, 0.9),
                size: 8.0.into(),
                ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }
}

pub(crate) fn fader_ticks<'a, Message>(_height: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    canvas(FaderTicks)
        .width(Length::Fixed(FADER_SCALE_GAP + FADER_SCALE_WIDTH))
        .height(Length::Fill)
        .into()
}

pub(crate) const INSERT_NAMES: [&str; 23] = [
    "OFF", "FX1L", "FX1R", "FX2L", "FX2R", "FX3L", "FX3R", "FX4L", "FX4R", "FX5L", "FX5R", "FX6L",
    "FX6R", "FX7L", "FX7R", "FX8L", "FX8R", "AUX1", "AUX2", "AUX3", "AUX4", "AUX5", "AUX6",
];

pub(crate) fn insert_selector<'a>(path: String, current: i32) -> Element<'a, Message> {
    let name = INSERT_NAMES.get(current as usize).copied().unwrap_or("OFF");
    let prev = (current - 1).max(0);
    let next = (current + 1).min(22);
    row![
        button(text("-").size(12))
            .on_press(Message::ParameterChanged(path.clone(), OscValue::Int(prev)))
            .padding([2, 6]),
        text(name).size(11).width(Length::Fixed(48.0)),
        button(text("+").size(12))
            .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
            .padding([2, 6]),
    ]
    .spacing(4)
    .align_y(maolan_widgets::iced::Alignment::Center)
    .into()
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

pub(crate) fn tap_type_selector<'a>(bus: u8, path: String, current: i32) -> Element<'a, Message> {
    const TAP_LABELS: [&str; 6] = ["IN/LC", "<-EQ", "EQ->", "PRE", "POST", "GRP"];
    let label = TAP_LABELS.get(current as usize).unwrap_or(&"POST");
    let next = (current + 1) % 6;
    row![
        text(format!("{bus:02}/{:02}", bus + 1))
            .size(11)
            .width(Length::Fixed(36.0))
            .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
        button(text(*label).size(11))
            .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
            .padding([2, 6])
            .style(
                move |_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x2A, 0x2D, 0x33))),
                    text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                    border: Border {
                        color: Color::from_rgb8(0x4A, 0x4D, 0x52),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            ),
    ]
    .spacing(4)
    .align_y(maolan_widgets::iced::Alignment::Center)
    .into()
}

pub(crate) fn param_float(app: &StatusApp, path: &str) -> f32 {
    match app.parameter_values.get(path) {
        Some(OscValue::Float(v)) => *v,
        Some(OscValue::Int(v)) => *v as f32,
        _ => 0.0,
    }
}

pub(crate) fn param_bool(app: &StatusApp, path: &str) -> bool {
    match app.parameter_values.get(path) {
        Some(OscValue::Bool(v)) => *v,
        Some(OscValue::Int(v)) => *v != 0,
        Some(OscValue::Float(v)) => *v != 0.0,
        _ => false,
    }
}

pub(crate) fn param_int(app: &StatusApp, path: &str) -> i32 {
    match app.parameter_values.get(path) {
        Some(OscValue::Int(v)) => *v,
        Some(OscValue::Float(v)) => *v as i32,
        _ => 0,
    }
}

pub(crate) fn param_slider_labeled<'a>(
    label: &'a str,
    path: String,
    value: f32,
    format_fn: fn(f32) -> String,
) -> Element<'a, Message> {
    column![
        row![
            text(label)
                .size(11)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            text(format_fn(value))
                .size(11)
                .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        ]
        .spacing(6)
        .align_y(maolan_widgets::iced::Alignment::Center),
        horizontal_slider(0.0..=1.0, value, move |next| {
            Message::ParameterChanged(path.clone(), OscValue::Float(next))
        })
        .fill_from_start()
        .step(0.01)
        .width(Length::Fill)
        .height(Length::Fixed(16.0)),
    ]
    .spacing(2)
    .into()
}

pub(crate) fn param_toggle<'a>(label: &'a str, path: String, active: bool) -> Element<'a, Message> {
    let color = if active {
        Color::from_rgb8(0x7D, 0xD3, 0xA7)
    } else {
        Color::from_rgb8(0xF0, 0x7C, 0x82)
    };
    button(text(label).size(12))
        .on_press(Message::ParameterChanged(
            path.clone(),
            OscValue::Bool(!active),
        ))
        .style(move |_theme: &Theme, _status: button::Status| toggle_button_style(active, color))
        .into()
}

pub(crate) fn toggle_button_style(active: bool, color: Color) -> button::Style {
    if active {
        button::Style {
            background: Some(Background::Color(color)),
            text_color: Color::from_rgb8(0x14, 0x18, 0x20),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color,
            },
            ..Default::default()
        }
    } else {
        button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: color,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color,
            },
            ..Default::default()
        }
    }
}

pub(crate) fn color_selector(path: String, current: u8) -> Element<'static, Message> {
    const COLORS: [(u8, Color); 15] = [
        (0, Color::from_rgb8(0x3B, 0x42, 0x52)),
        (1, Color::from_rgb8(0xFF, 0x45, 0x45)),
        (2, Color::from_rgb8(0x32, 0xCD, 0x32)),
        (3, Color::from_rgb8(0xFF, 0xD7, 0x00)),
        (4, Color::from_rgb8(0x41, 0x69, 0xE1)),
        (5, Color::from_rgb8(0xFF, 0x00, 0xFF)),
        (6, Color::from_rgb8(0x00, 0xFF, 0xFF)),
        (7, Color::from_rgb8(0xFF, 0xFF, 0xFF)),
        (9, Color::from_rgb8(0xCC, 0x33, 0x33)),
        (10, Color::from_rgb8(0x28, 0xA4, 0x28)),
        (11, Color::from_rgb8(0xCC, 0xAC, 0x00)),
        (12, Color::from_rgb8(0x33, 0x55, 0xB4)),
        (13, Color::from_rgb8(0xCC, 0x00, 0xCC)),
        (14, Color::from_rgb8(0x00, 0xCC, 0xCC)),
        (15, Color::from_rgb8(0xDD, 0xDD, 0xDD)),
    ];

    COLORS
        .into_iter()
        .fold(row!().spacing(2), |r, (val, color)| {
            let selected = current == val;
            r.push(
                button(
                    Space::new()
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0)),
                )
                .on_press(Message::ParameterChanged(
                    path.clone(),
                    OscValue::Int(val as i32),
                ))
                .style(
                    move |_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(color)),
                        border: Border {
                            color: if selected {
                                Color::WHITE
                            } else {
                                Color::TRANSPARENT
                            },
                            width: if selected { 2.0 } else { 0.0 },
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    },
                ),
            )
        })
        .into()
}

pub(crate) fn icon_selector(path: String, current: i32) -> Element<'static, Message> {
    let prev = (current - 1).max(0);
    let next = (current + 1).min(74);
    row![
        button(text("-").size(12))
            .on_press(Message::ParameterChanged(path.clone(), OscValue::Int(prev)))
            .padding([2, 6]),
        text(format!("Icon {current}"))
            .size(11)
            .width(Length::Fixed(48.0)),
        button(text("+").size(12))
            .on_press(Message::ParameterChanged(path, OscValue::Int(next)))
            .padding([2, 6]),
    ]
    .spacing(4)
    .align_y(maolan_widgets::iced::Alignment::Center)
    .into()
}
