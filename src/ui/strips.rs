use crate::message::{AppView, Message, SelectedStrip};
use crate::model::{FaderTarget, GainSource};
use crate::state::StatusApp;
use crate::ui::format::*;
use crate::ui::nav::{mixer_accent_color, strip_in_spill, strip_module_item};
use crate::ui::widgets::*;
use maolan_widgets::horizontal_slider::horizontal_slider;
use maolan_widgets::iced::widget::{
    Space, button, column, container, row, scrollable, text, text_input,
};
use maolan_widgets::iced::{Background, Border, Color, Element, Length, Theme};
use maolan_widgets::meters::meters;
use maolan_widgets::slider::slider as vertical_slider;

pub(crate) fn mixer_strips(app: &StatusApp) -> Element<'_, Message> {
    let strips = app.visible_strips().iter().enumerate().fold(
        row!()
            .spacing(0)
            .align_y(maolan_widgets::iced::Alignment::End),
        |strips, (index, target)| {
            let value = app.faders[index];
            let gain_value = app.gain_drag_values[index]
                .or(app.gains[index])
                .unwrap_or(0.0);
            let gain_source = app.gain_sources[index];
            let fader_value = value.unwrap_or(0.0);
            let pan_value = app.pans[index].unwrap_or(0.5);
            let gain_label = format_gain_label(gain_value, gain_source);
            let value_label = value
                .map(format_fader_label)
                .unwrap_or_else(|| "--".to_owned());
            let pan_label = format_pan_label(pan_value);
            let target = *target;
            let is_muted = app.muted[index].unwrap_or(false);
            let is_soloed = app.soloed[index].unwrap_or(false);
            let meter = container(
                meters(1, &[app.meters_db[index]], STRIP_METER_HEIGHT)
                    .map(|()| unreachable!("meter widget does not emit messages")),
            )
            .height(Length::Fill);
            let scale = container(
                fader_ticks(STRIP_METER_HEIGHT)
                    .map(|()| unreachable!("tick widget does not emit messages")),
            )
            .height(Length::Fill)
            .align_y(maolan_widgets::iced::alignment::Vertical::Bottom);
            let sends: Element<'_, Message> = match target {
                FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => app
                    .send_buses()
                    .iter()
                    .enumerate()
                    .fold(
                        column!()
                            .spacing(2)
                            .align_x(maolan_widgets::iced::Alignment::Center),
                        |column, (bus_index, _bus)| {
                            let send_value = app.sends[index][bus_index].unwrap_or(0.0);
                            column.push(
                                horizontal_slider(0.0..=1.0, send_value, move |next| {
                                    Message::SendChanged(index, bus_index, next)
                                })
                                .fill_from_start()
                                .step(0.01)
                                .double_click_reset(0.0)
                                .width(Length::Fixed(72.0))
                                .height(Length::Fixed(10.0)),
                            )
                        },
                    )
                    .into(),
                FaderTarget::Bus(_) | FaderTarget::Main => app
                    .matrix_sends()
                    .iter()
                    .enumerate()
                    .fold(
                        column!()
                            .spacing(2)
                            .align_x(maolan_widgets::iced::Alignment::Center),
                        |column, (bus_index, _bus)| {
                            let send_value = app.sends[index][bus_index].unwrap_or(0.0);
                            column.push(
                                horizontal_slider(0.0..=1.0, send_value, move |next| {
                                    Message::SendChanged(index, bus_index, next)
                                })
                                .fill_from_start()
                                .step(0.01)
                                .double_click_reset(0.0)
                                .width(Length::Fixed(72.0))
                                .height(Length::Fixed(10.0)),
                            )
                        },
                    )
                    .into(),
                FaderTarget::Mtx(_) | FaderTarget::Dca(_) => {
                    Space::new().height(Length::Fixed(0.0)).into()
                }
            };
            let hide_strip_top_controls = app.active_view != AppView::Mixer;
            let top_sends: Element<'_, Message> = if hide_strip_top_controls {
                Space::new().height(Length::Fixed(0.0)).into()
            } else {
                sends
            };
            let top_gain_label = if hide_strip_top_controls {
                String::new()
            } else {
                gain_label
            };
            let top_controls = strip_mixer_top(
                index,
                target,
                gain_value,
                gain_source,
                top_gain_label,
                pan_value,
                if hide_strip_top_controls {
                    String::new()
                } else {
                    pan_label
                },
                top_sends,
            );

            let solo_button: Element<'_, Message> = if matches!(target, FaderTarget::Mtx(_)) {
                Space::new().height(Length::Fixed(0.0)).into()
            } else {
                button(text("SOLO").size(12))
                    .padding([6, 8])
                    .style(move |_theme: &Theme, _status| {
                        toggle_button_style(is_soloed, Color::from_rgb8(0xF0, 0xC0, 0x30))
                    })
                    .on_press(Message::SoloPressed(index))
                    .into()
            };

            let mut strip = column![top_controls]
                .spacing(10)
                .align_x(maolan_widgets::iced::Alignment::Center);
            let strip_color = app.colors[index].unwrap_or(0);
            let color_rgb = x32_color_to_rgb(strip_color);
            let is_inverted = (9..=15).contains(&strip_color);
            let text_color = if is_inverted { Color::BLACK } else { color_rgb };
            let bg = if is_inverted {
                Some(Background::Color(color_rgb))
            } else {
                None
            };
            let is_selected = app.selected_strip == Some(SelectedStrip::Strip(index));
            let is_editing = app
                .editing_name
                .as_ref()
                .map(|(edit_index, _)| *edit_index == index)
                .unwrap_or(false);

            let name_element: Element<'_, Message> = if is_editing {
                let text = app
                    .editing_name
                    .as_ref()
                    .map(|(_, text)| text.clone())
                    .unwrap_or_default();
                text_input("Name", &text)
                    .size(12)
                    .width(Length::Fixed(80.0))
                    .on_input(move |t| Message::NameEditChanged(index, t))
                    .on_submit(Message::NameEditSubmitted(index))
                    .into()
            } else {
                button(
                    container(
                        text(strip_name(app, index, target))
                            .size(14)
                            .color(text_color),
                    )
                    .style(move |_theme: &Theme| container::Style {
                        border: Border {
                            color: color_rgb,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        background: bg,
                        ..Default::default()
                    })
                    .padding([2, 6]),
                )
                .style(button::text)
                .on_press(if is_selected {
                    Message::NameEditStarted(index)
                } else {
                    Message::StripSelected(SelectedStrip::Strip(index))
                })
                .into()
            };
            strip = strip.push(name_element);
            if !matches!(target, FaderTarget::Mtx(_)) {
                strip = strip.push(solo_button);
            }
            strip = strip.push(text(value_label).size(14));
            strip = strip.push(
                row![
                    vertical_slider(0.0..=1.0, fader_value, move |next| Message::FaderChanged(
                        index, next
                    ))
                    .height(Length::Fill)
                    .width(Length::Fixed(20.0))
                    .double_click_reset(0.75)
                    .step(0.01),
                    scale,
                    meter,
                ]
                .spacing(6)
                .height(Length::Fill)
                .align_y(maolan_widgets::iced::Alignment::End),
            );
            strip = strip.push(
                button(text("MUTE").size(12))
                    .padding([6, 8])
                    .style(move |_theme: &Theme, _status| {
                        toggle_button_style(is_muted, Color::from_rgb8(0xE0, 0x50, 0x50))
                    })
                    .on_press(Message::MutePressed(index)),
            );
            strip = strip.push(text(strip_label(target)).size(14));

            let in_spill = strip_in_spill(app, target, index);
            strips.push(
                container(strip)
                    .style(move |_theme: &Theme| container::Style {
                        border: Border {
                            color: if is_selected {
                                mixer_accent_color()
                            } else if in_spill {
                                Color::from_rgb8(0xF0, 0xC0, 0x50)
                            } else {
                                Color::from_rgb8(0x3B, 0x42, 0x52)
                            },
                            width: if is_selected || in_spill { 2.0 } else { 1.0 },
                            radius: 4.0.into(),
                        },
                        background: if in_spill {
                            Some(Background::Color(Color::from_rgb8(0x2A, 0x25, 0x15)))
                        } else {
                            None
                        },
                        ..Default::default()
                    })
                    .padding([0, 7]),
            )
        },
    );

    let master_selected = app.selected_strip == Some(SelectedStrip::Master);
    let master_strip = {
        let value = app.master_fader.unwrap_or(0.0);
        let value_label = app
            .master_fader
            .map(format_fader_label)
            .unwrap_or_else(|| "--".to_owned());
        let is_muted = app.master_muted.unwrap_or(false);
        let is_soloed = app.master_soloed.unwrap_or(false);
        let meter = container(
            meters(2, &app.master_meters_db, STRIP_METER_HEIGHT)
                .map(|()| unreachable!("meter widget does not emit messages")),
        )
        .height(Length::Fill);
        let scale = container(
            fader_ticks(STRIP_METER_HEIGHT)
                .map(|()| unreachable!("tick widget does not emit messages")),
        )
        .height(Length::Fill)
        .align_y(maolan_widgets::iced::alignment::Vertical::Bottom);

        column![
            Space::new().height(Length::Fixed(26.0)),
            Space::new().height(Length::Fixed(0.0)),
            {
                let master_color_val = app.master_color.unwrap_or(0);
                let color_rgb = x32_color_to_rgb(master_color_val);
                let is_inverted = (9..=15).contains(&master_color_val);
                let text_color = if is_inverted { Color::BLACK } else { color_rgb };
                let bg = if is_inverted {
                    Some(Background::Color(color_rgb))
                } else {
                    None
                };
                button(
                    container(text("LR").size(14).color(text_color))
                        .style(move |_theme: &Theme| container::Style {
                            border: Border {
                                color: color_rgb,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            background: bg,
                            ..Default::default()
                        })
                        .padding([2, 6]),
                )
                .style(button::text)
                .on_press(Message::StripSelected(SelectedStrip::Master))
            },
            button(text("SOLO").size(12))
                .padding([6, 8])
                .style(move |_theme: &Theme, _status| toggle_button_style(
                    is_soloed,
                    Color::from_rgb8(0xF0, 0xC0, 0x30)
                ))
                .on_press(Message::MasterSoloPressed),
            text(value_label).size(14),
            row![
                vertical_slider(0.0..=1.0, value, Message::MasterFaderChanged)
                    .height(Length::Fill)
                    .width(Length::Fixed(20.0))
                    .double_click_reset(0.75)
                    .step(0.01),
                scale,
                meter,
            ]
            .spacing(6)
            .height(Length::Fill)
            .align_y(maolan_widgets::iced::Alignment::End),
            button(text("MUTE").size(12))
                .padding([6, 8])
                .style(move |_theme: &Theme, _status| toggle_button_style(
                    is_muted,
                    Color::from_rgb8(0xE0, 0x50, 0x50)
                ))
                .on_press(Message::MasterMutePressed),
            text("LR").size(14),
        ]
        .spacing(10)
        .align_x(maolan_widgets::iced::Alignment::Center)
    };

    let master_strip = container(master_strip)
        .style(move |_theme: &Theme| container::Style {
            border: Border {
                color: if master_selected {
                    mixer_accent_color()
                } else {
                    Color::from_rgb8(0x3B, 0x42, 0x52)
                },
                width: if master_selected { 2.0 } else { 1.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .padding([0, 7]);

    container(
        row![
            scrollable(
                column![
                    strips.height(Length::Fill),
                    Space::new().height(Length::Fixed(18.0))
                ]
                .height(Length::Fill),
            )
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new()
            ))
            .width(Length::Fill)
            .height(Length::Fill),
            master_strip,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(maolan_widgets::iced::Alignment::End),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn strip_mixer_top(
    index: usize,
    target: FaderTarget,
    gain_value: f32,
    gain_source: GainSource,
    gain_label: String,
    pan_value: f32,
    pan_label: String,
    sends: Element<'_, Message>,
) -> Element<'_, Message> {
    let hide_upper_controls = gain_label.is_empty();
    let hide_balance = pan_label.is_empty();
    let gain_block: Element<'static, Message> = if hide_upper_controls
        || matches!(
            target,
            FaderTarget::Bus(_) | FaderTarget::FxRtn(_) | FaderTarget::Mtx(_) | FaderTarget::Dca(_)
        ) {
        Space::new().height(Length::Fixed(26.0)).into()
    } else {
        column![
            text(gain_label).size(12),
            horizontal_slider(gain_range(gain_source), gain_value, move |next| {
                Message::GainChanged(index, next)
            })
            .fill_from_start()
            .filled_color(Color::from_rgb8(0xD9, 0x7A, 0x2B))
            .handle_color(Color::from_rgb8(0xF3, 0xB3, 0x6A))
            .step(gain_step(gain_source))
            .double_click_reset(0.0)
            .on_release(Message::GainReleased(index))
            .width(Length::Fixed(72.0))
            .height(Length::Fixed(10.0)),
        ]
        .spacing(4)
        .align_x(maolan_widgets::iced::Alignment::Center)
        .into()
    };

    let pan_block: Element<'static, Message> =
        if hide_balance || matches!(target, FaderTarget::Dca(_) | FaderTarget::Mtx(_)) {
            Space::new().height(Length::Fixed(0.0)).into()
        } else {
            column![
                text(pan_label).size(12),
                horizontal_slider(0.0..=1.0, pan_value, move |next| Message::PanChanged(
                    index, next
                ))
                .step(0.01)
                .double_click_reset(0.5)
                .width(Length::Fixed(72.0))
                .height(Length::Fixed(12.0)),
            ]
            .spacing(4)
            .align_x(maolan_widgets::iced::Alignment::Center)
            .into()
        };

    let mut top = column![gain_block]
        .spacing(10)
        .align_x(maolan_widgets::iced::Alignment::Center);
    if !hide_upper_controls && !matches!(target, FaderTarget::Mtx(_) | FaderTarget::Dca(_)) {
        top = top.push(sends);
        top = top.push(
            column![
                strip_module_item("Gate"),
                strip_module_item("EQ"),
                strip_module_item("Dyn"),
            ]
            .spacing(4)
            .align_x(maolan_widgets::iced::Alignment::Center),
        );
    }
    top.push(pan_block).into()
}

pub(crate) fn gate_summary<'a>(app: &'a StatusApp, base: &str) -> Element<'a, Message> {
    let on = param_bool(app, &format!("{base}/gate/on"));
    let thr = param_float(app, &format!("{base}/gate/thr"));
    let color = if on {
        Color::from_rgb8(0x7D, 0xD3, 0xA7)
    } else {
        Color::from_rgb8(0x8E, 0x94, 0x9D)
    };
    column![
        text(if on { "ON" } else { "OFF" }).size(14).color(color),
        text(format!("Thr: {thr:.1}"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
    ]
    .spacing(4)
    .align_x(maolan_widgets::iced::Alignment::Center)
    .into()
}

pub(crate) fn eq_summary<'a>(app: &'a StatusApp, base: &str, bands: u8) -> Element<'a, Message> {
    let on = param_bool(app, &format!("{base}/eq/on"));
    let active = if on {
        (1..=bands)
            .filter(|b| param_bool(app, &format!("{base}/eq/{b:02}/on")))
            .count()
    } else {
        0
    };
    let color = if on {
        Color::from_rgb8(0x7D, 0xD3, 0xA7)
    } else {
        Color::from_rgb8(0x8E, 0x94, 0x9D)
    };
    column![
        text(if on { "ON" } else { "OFF" }).size(14).color(color),
        text(format!("{active}/{bands} bands"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
    ]
    .spacing(4)
    .align_x(maolan_widgets::iced::Alignment::Center)
    .into()
}

pub(crate) const DYN_RATIO_NAMES: [&str; 12] = [
    "1.1", "1.3", "1.5", "2.0", "2.5", "3.0", "4.0", "5.0", "7.0", "10", "20", "100",
];

pub(crate) fn dyn_summary<'a>(app: &'a StatusApp, base: &str) -> Element<'a, Message> {
    let on = param_bool(app, &format!("{base}/dyn/on"));
    let thr = param_float(app, &format!("{base}/dyn/thr"));
    let ratio_idx = param_float(app, &format!("{base}/dyn/ratio")) as i32;
    let ratio_name = DYN_RATIO_NAMES
        .get(ratio_idx as usize)
        .copied()
        .unwrap_or("?");
    let color = if on {
        Color::from_rgb8(0x7D, 0xD3, 0xA7)
    } else {
        Color::from_rgb8(0x8E, 0x94, 0x9D)
    };
    column![
        text(if on { "ON" } else { "OFF" }).size(14).color(color),
        text(format!("Thr: {thr:.1}"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
        text(format!("Ratio: {ratio_name}"))
            .size(11)
            .color(Color::from_rgb8(0xA9, 0xAC, 0xB3)),
    ]
    .spacing(4)
    .align_x(maolan_widgets::iced::Alignment::Center)
    .into()
}

pub(crate) fn module_summary_panel<'a>(
    title: &'static str,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        column![
            text(title)
                .size(12)
                .color(Color::from_rgb8(0xC7, 0xC9, 0xD3)),
            content,
        ]
        .spacing(8)
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
    .width(Length::Fixed(140.0))
    .into()
}
