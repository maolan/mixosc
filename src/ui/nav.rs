use crate::message::{AppView, Message};
use crate::model::FaderTarget;
use crate::parameters::OscValue;
use crate::state::StatusApp;
use crate::ui::format::*;
use crate::ui::widgets::*;
use maolan_widgets::iced::widget::{Space, button, container, row, text};
use maolan_widgets::iced::{Background, Border, Color, Element, Fill, Length, Theme};
use maolan_widgets::iced_fonts::lucide::{
    activity, audio_lines, audio_waveform, equal, file_input, git_merge, panel_left, save, send,
    settings, shield, sliders_vertical, toggle_left,
};

#[derive(Clone, Copy)]
pub struct NavTab {
    icon: fn() -> maolan_widgets::iced::widget::Text<'static, Theme>,
    label: &'static str,
    view: AppView,
}

pub(crate) fn spill_bar(app: &StatusApp) -> Element<'static, Message> {
    let mute_count = app.config().mute_group_count;
    let mute_buttons: Element<'_, Message> = (1..=mute_count)
        .fold(row!().spacing(4), |row, grp| {
            let path = format!("/config/mute/{grp}");
            let active = param_bool(app, &path);
            let (bg, border) = if active {
                (
                    Color::from_rgb8(0x8A, 0x3A, 0x3A),
                    Color::from_rgb8(0xC0, 0x5A, 0x5A),
                )
            } else {
                (
                    Color::from_rgb8(0x2A, 0x2A, 0x2C),
                    Color::from_rgb8(0x4A, 0x4A, 0x4C),
                )
            };
            row.push(
                button(text(format!("M{grp}")).size(11))
                    .on_press(Message::ParameterChanged(path, OscValue::Bool(!active)))
                    .padding([3, 8])
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

    let dca_count = app.config().dca_count;
    let spill_buttons: Element<'_, Message> = if let Some(dca) = app.dca_spill {
        (1..=dca_count)
            .fold(row!().spacing(2), |row, n| {
                let n = n as u8;
                let active = dca == n;
                let (bg, border) = if active {
                    (
                        Color::from_rgb8(0x5A, 0x3A, 0x8A),
                        Color::from_rgb8(0x8A, 0x5A, 0xC0),
                    )
                } else {
                    (
                        Color::from_rgb8(0x1A, 0x1A, 0x1C),
                        Color::from_rgb8(0x2A, 0x2A, 0x2C),
                    )
                };
                row.push(
                    button(
                        text(if active {
                            format!("d{n}")
                        } else {
                            "·".to_owned()
                        })
                        .size(8),
                    )
                    .on_press(Message::DcaSpill(n))
                    .padding([1, 4])
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
            .into()
    } else {
        Space::new().width(Length::Fixed(0.0)).into()
    };

    let clear_btn: Element<'static, Message> =
        if app.dca_spill.is_some() || app.mute_spill.is_some() {
            button(text("Clear").size(10))
                .on_press(Message::ClearSpill)
                .padding([3, 8])
                .style(|_theme: &Theme, _status: button::Status| button::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x3A, 0x5A, 0x8A))),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb8(0x5A, 0x8A, 0xC0),
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            Space::new().width(Length::Fixed(0.0)).into()
        };

    let dca_count = app.config().dca_count;
    let dca_buttons: Element<'_, Message> = if app.dca_spill.is_none() && app.mute_spill.is_none() {
        (1..=dca_count)
            .fold(row!().spacing(4), |row, dca| {
                let dca = dca as u8;
                row.push(
                    button(text(format!("D{dca}")).size(11))
                        .on_press(Message::DcaSpill(dca))
                        .padding([3, 8])
                        .style(|_theme: &Theme, _status: button::Status| button::Style {
                            background: Some(Background::Color(Color::from_rgb8(0x2A, 0x2A, 0x2C))),
                            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
                            border: Border {
                                color: Color::from_rgb8(0x4A, 0x4A, 0x4C),
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        }),
                )
            })
            .into()
    } else {
        Space::new().width(Length::Fixed(0.0)).into()
    };

    let clear_solo_btn = button(text("Clear Solo").size(10))
        .on_press(Message::ClearSolo)
        .padding([3, 8])
        .style(|_theme: &Theme, _status: button::Status| button::Style {
            background: Some(Background::Color(Color::from_rgb8(0x8A, 0x6A, 0x2A))),
            text_color: Color::from_rgb8(0xC7, 0xC9, 0xD3),
            border: Border {
                color: Color::from_rgb8(0xC0, 0xA0, 0x5A),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        });

    row![
        text("Mute:")
            .size(11)
            .color(Color::from_rgb8(0x8E, 0x94, 0x9D)),
        mute_buttons,
        Space::new().width(Length::Fixed(8.0)),
        spill_buttons,
        clear_btn,
        dca_buttons,
        Space::new().width(Length::Fixed(12.0)),
        clear_solo_btn,
    ]
    .spacing(4)
    .align_y(maolan_widgets::iced::Alignment::Center)
    .into()
}

pub(crate) fn top_nav_bar(app: &StatusApp) -> Element<'static, Message> {
    const TABS: [NavTab; 13] = [
        NavTab {
            icon: sliders_vertical,
            label: "Mixer",
            view: AppView::Mixer,
        },
        NavTab {
            icon: panel_left,
            label: "Channel",
            view: AppView::Channel,
        },
        NavTab {
            icon: file_input,
            label: "Config",
            view: AppView::Config,
        },
        NavTab {
            icon: toggle_left,
            label: "Gate",
            view: AppView::Gate,
        },
        NavTab {
            icon: audio_waveform,
            label: "Dyn",
            view: AppView::Dyn,
        },
        NavTab {
            icon: equal,
            label: "EQ",
            view: AppView::Eq,
        },
        NavTab {
            icon: send,
            label: "Sends",
            view: AppView::Sends,
        },
        NavTab {
            icon: audio_lines,
            label: "Main",
            view: AppView::Main,
        },
        NavTab {
            icon: shield,
            label: "FX1 - 8",
            view: AppView::Fx,
        },
        NavTab {
            icon: save,
            label: "Scenes",
            view: AppView::Scenes,
        },
        NavTab {
            icon: settings,
            label: "Setup",
            view: AppView::Setup,
        },
        NavTab {
            icon: git_merge,
            label: "Routing",
            view: AppView::Routing,
        },
        NavTab {
            icon: activity,
            label: "RTA",
            view: AppView::Rta,
        },
    ];

    let tabs = TABS.into_iter().fold(
        row!()
            .spacing(4)
            .padding([3, 3])
            .align_y(maolan_widgets::iced::Alignment::Center),
        |row, tab| row.push(nav_button(tab, app.active_view == tab.view)),
    );

    let disconnect_btn = button(text("Disconnect").size(12))
        .on_press(Message::Disconnect)
        .style(|_theme: &Theme, _status: button::Status| button::Style {
            background: Some(Background::Color(Color::from_rgb8(0xF0, 0x7C, 0x82))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb8(0xF0, 0x7C, 0x82),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    let bar = row![tabs, Space::new().width(Length::Fill), disconnect_btn]
        .spacing(8)
        .padding([4, 8])
        .align_y(maolan_widgets::iced::Alignment::Center)
        .width(Length::Fill);

    container(bar)
        .height(Length::Shrink)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x1C, 0x1C, 0x1C))),
            border: Border {
                color: Color::from_rgb8(0x2A, 0x2A, 0x2A),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

pub(crate) fn nav_button(tab: NavTab, selected: bool) -> Element<'static, Message> {
    let accent = mixer_accent_color();
    let active_text = accent;
    let inactive_text = Color::from_rgb8(0xA9, 0xAC, 0xB3);

    let icon =
        container(
            (tab.icon)()
                .size(17)
                .color(if selected { active_text } else { inactive_text }),
        )
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .padding(0)
        .center_x(Fill)
        .center_y(Fill)
        .style(move |_theme: &Theme| container::Style {
            border: Border {
                color: if selected {
                    accent
                } else {
                    Color::from_rgb8(0x6B, 0x6F, 0x76)
                },
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        });

    button(
        row![
            icon,
            text(tab.label)
                .size(14)
                .color(if selected { active_text } else { inactive_text }),
        ]
        .spacing(8)
        .align_y(maolan_widgets::iced::Alignment::Center),
    )
    .padding([4, 10])
    .width(Length::Fixed(108.0))
    .height(Length::Fixed(36.0))
    .style(move |_theme: &Theme, _status| button::Style {
        background: Some(Background::Color(if selected {
            Color::from_rgb8(0x2A, 0x2A, 0x2A)
        } else {
            Color::from_rgb8(0x24, 0x24, 0x24)
        })),
        border: Border {
            color: if selected {
                Color::from_rgb8(0x4B, 0x4B, 0x4B)
            } else {
                Color::from_rgb8(0x3A, 0x3A, 0x3A)
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        text_color: if selected { active_text } else { inactive_text },
        ..Default::default()
    })
    .on_press(Message::NavSelected(tab.view))
    .into()
}

pub(crate) fn mixer_accent_color() -> Color {
    Color::from_rgb8(0x29, 0xE6, 0xF2)
}

pub(crate) fn strip_in_spill(app: &StatusApp, target: FaderTarget, _index: usize) -> bool {
    if let Some(dca) = app.dca_spill {
        let base = strip_base_path(target, app.mixer_model);
        let dca_path = format!("{base}/grp/dca");
        let dca_val = param_int(app, &dca_path);
        return (dca_val & (1 << (dca - 1))) != 0;
    }
    if let Some(grp) = app.mute_spill {
        let base = strip_base_path(target, app.mixer_model);
        let mute_path = format!("{base}/grp/mute");
        let mute_val = param_int(app, &mute_path);
        return (mute_val & (1 << (grp - 1))) != 0;
    }
    false
}

pub(crate) fn strip_module_item(label: &'static str) -> Element<'static, Message> {
    text(label)
        .size(12)
        .color(Color::from_rgb8(0xC7, 0xC9, 0xD3))
        .into()
}
