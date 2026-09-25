use crate::codec::{
    parse_console_update, parse_input_meter_packet, parse_main_meter_packet, parse_rta_meter_packet,
};
use crate::message::{AppView, Message};
use crate::model::config_for_model;
use crate::model::{
    ConsoleUpdate, FaderTarget, GainSource, MainMeterLevels, MixerModel, StripMeter,
};
use crate::net::parse_target;
use crate::net::{
    ColorBankProbe, ConnectionProbe, DiscoveryProbe, FaderBankProbe, GainBankProbe, MuteBankProbe,
    NameBankProbe, PanBankProbe, SendBankProbe, SoloBankProbe, XREMOTE_REQUEST, XREMOTENFB_REQUEST,
};
use crate::osc::{batchsubscribe_meter_request, osc_meter_group_request, renew_request};
use crate::panel_paths::panel_parameter_paths;
use crate::parameters::OscValue;
use crate::state::StatusApp;
use maolan_widgets::iced::futures::sink::SinkExt;
use maolan_widgets::iced::futures::{StreamExt, channel::mpsc, stream::BoxStream};
use maolan_widgets::iced::stream;
use maolan_widgets::iced::{Subscription, Task};
use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{Instant, sleep};

pub(crate) fn spawn_probe(mixer_addr: SocketAddr) -> Task<Message> {
    Task::perform(
        async move {
            ConnectionProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(400))
                .probe()
                .map_err(|error| error.to_string())
        },
        Message::ProbeFinished,
    )
}

pub(crate) fn spawn_load_faders(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    Task::perform(
        async move {
            FaderBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .load(&[config_for_model(model).visible_strips, &[FaderTarget::Main]].concat())
                .map_err(|error| error.to_string())
        },
        Message::FadersLoaded,
    )
}

pub(crate) fn spawn_load_names(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    Task::perform(
        async move {
            NameBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .load(config_for_model(model).visible_strips)
                .map_err(|error| error.to_string())
        },
        Message::NamesLoaded,
    )
}

pub(crate) fn spawn_load_colors(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    Task::perform(
        async move {
            ColorBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .load(&[config_for_model(model).visible_strips, &[FaderTarget::Main]].concat())
                .map_err(|error| error.to_string())
        },
        Message::ColorsLoaded,
    )
}

pub(crate) fn spawn_load_gains(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    let targets: Vec<FaderTarget> = config_for_model(model)
        .visible_strips
        .iter()
        .filter(|t| {
            !matches!(
                t,
                FaderTarget::Bus(_)
                    | FaderTarget::FxRtn(_)
                    | FaderTarget::Mtx(_)
                    | FaderTarget::Dca(_)
            )
        })
        .cloned()
        .collect();
    Task::perform(
        async move {
            GainBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .load(&targets)
                .map_err(|error| error.to_string())
        },
        Message::GainsLoaded,
    )
}

pub(crate) fn spawn_load_sends(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    let channel_aux_targets: Vec<FaderTarget> = config_for_model(model)
        .visible_strips
        .iter()
        .filter(|t| {
            matches!(
                t,
                FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_)
            )
        })
        .cloned()
        .collect();
    let bus_targets: Vec<FaderTarget> = config_for_model(model)
        .visible_strips
        .iter()
        .filter(|t| matches!(t, FaderTarget::Bus(_) | FaderTarget::Main))
        .cloned()
        .collect();
    Task::batch([
        Task::perform(
            async move {
                SendBankProbe::new(mixer_addr)
                    .with_model(model)
                    .with_timeout(Duration::from_millis(250))
                    .load(&channel_aux_targets, config_for_model(model).send_buses)
                    .map_err(|error| error.to_string())
            },
            Message::SendsLoaded,
        ),
        Task::perform(
            async move {
                SendBankProbe::new(mixer_addr)
                    .with_model(model)
                    .with_timeout(Duration::from_millis(250))
                    .load(&bus_targets, config_for_model(model).matrix_sends)
                    .map_err(|error| error.to_string())
            },
            Message::SendsLoaded,
        ),
    ])
}

pub(crate) fn spawn_load_pans(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    let targets: Vec<FaderTarget> = config_for_model(model)
        .visible_strips
        .iter()
        .filter(|t| !matches!(t, FaderTarget::Dca(_) | FaderTarget::Mtx(_)))
        .cloned()
        .collect();
    Task::perform(
        async move {
            PanBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .load(&targets)
                .map_err(|error| error.to_string())
        },
        Message::PansLoaded,
    )
}

pub(crate) fn spawn_load_mutes(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    Task::perform(
        async move {
            MuteBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .load(&[config_for_model(model).visible_strips, &[FaderTarget::Main]].concat())
                .map_err(|error| error.to_string())
        },
        Message::MutesLoaded,
    )
}

pub(crate) fn spawn_load_solos(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    let targets: Vec<FaderTarget> = config_for_model(model)
        .visible_strips
        .iter()
        .filter(|t| !matches!(t, FaderTarget::Mtx(_) | FaderTarget::Dca(_)))
        .cloned()
        .collect();
    Task::perform(
        async move {
            SoloBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .load(&targets)
                .map_err(|error| error.to_string())
        },
        Message::SolosLoaded,
    )
}

pub(crate) fn spawn_set_fader(
    mixer_addr: SocketAddr,
    target: FaderTarget,
    value: f32,
    model: MixerModel,
) -> Task<Message> {
    Task::perform(
        async move {
            FaderBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .set(target, value)
                .map_err(|error| error.to_string())
        },
        Message::FaderSetFinished,
    )
}

pub(crate) fn spawn_set_pan(
    mixer_addr: SocketAddr,
    target: FaderTarget,
    value: f32,
    model: MixerModel,
) -> Task<Message> {
    Task::perform(
        async move {
            PanBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .set(target, value)
                .map_err(|error| error.to_string())
        },
        Message::PanSetFinished,
    )
}

pub(crate) fn spawn_set_send(
    mixer_addr: SocketAddr,
    target: FaderTarget,
    bus: u8,
    value: f32,
    model: MixerModel,
) -> Task<Message> {
    Task::perform(
        async move {
            SendBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .set(target, bus, value)
                .map_err(|error| error.to_string())
        },
        Message::SendSetFinished,
    )
}

pub(crate) fn spawn_set_gain(
    mixer_addr: SocketAddr,
    target: FaderTarget,
    source: GainSource,
    value: f32,
    model: MixerModel,
) -> Task<Message> {
    Task::perform(
        async move {
            GainBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .set(target, source, value)
                .map_err(|error| error.to_string())
        },
        Message::GainSetFinished,
    )
}

pub(crate) fn spawn_set_mute(
    mixer_addr: SocketAddr,
    target: FaderTarget,
    on: bool,
    model: MixerModel,
) -> Task<Message> {
    Task::perform(
        async move {
            MuteBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .set(target, on)
                .map_err(|error| error.to_string())
        },
        Message::MuteSetFinished,
    )
}

pub(crate) fn spawn_set_solo(
    mixer_addr: SocketAddr,
    target: FaderTarget,
    on: bool,
    model: MixerModel,
) -> Task<Message> {
    Task::perform(
        async move {
            SoloBankProbe::new(mixer_addr)
                .with_model(model)
                .with_timeout(Duration::from_millis(250))
                .set(target, on)
                .map_err(|error| error.to_string())
        },
        Message::SoloSetFinished,
    )
}

pub(crate) fn spawn_set_parameter(
    mixer_addr: SocketAddr,
    path: String,
    value: OscValue,
) -> Task<Message> {
    Task::perform(
        async move {
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(250))
                .set(&path, value)
                .map_err(|error| error.to_string())
        },
        Message::ParameterSetFinished,
    )
}

pub(crate) fn spawn_fetch_snippet_filters(mixer_addr: SocketAddr, snip: i32) -> Task<Message> {
    let paths = vec![
        format!("/-show/showfile/snippet/{snip:03}/eventtyp"),
        format!("/-show/showfile/snippet/{snip:03}/channels"),
        format!("/-show/showfile/snippet/{snip:03}/auxbuses"),
        format!("/-show/showfile/snippet/{snip:03}/maingrps"),
    ];
    Task::perform(
        async move {
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(500))
                .load_batch(&paths)
                .map_err(|error| error.to_string())
        },
        Message::ParametersLoaded,
    )
}

pub(crate) fn spawn_scene_action(
    mixer_addr: SocketAddr,
    action: &'static str,
    index: i32,
) -> Task<Message> {
    Task::perform(
        async move {
            let path = format!("/-action/{action}");
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(500))
                .set(&path, OscValue::Int(index))
                .map_err(|error| error.to_string())
        },
        Message::ParameterSetFinished,
    )
}

pub(crate) fn spawn_snapshot_action(
    mixer_addr: SocketAddr,
    action: &'static str,
    index: i32,
) -> Task<Message> {
    Task::perform(
        async move {
            let path = format!("/-snap/{action}");
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(500))
                .set(&path, OscValue::Int(index))
                .map_err(|error| error.to_string())
        },
        Message::ParameterSetFinished,
    )
}

pub(crate) fn spawn_recorder_action(mixer_addr: SocketAddr, action: &'static str) -> Task<Message> {
    Task::perform(
        async move {
            let path = format!("/-action/{action}");
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(500))
                .set(&path, OscValue::Int(1))
                .map_err(|error| error.to_string())
        },
        Message::ParameterSetFinished,
    )
}

pub(crate) fn target_to_ch_index(target: FaderTarget) -> i32 {
    match target {
        FaderTarget::Channel(n) => n as i32 - 1,
        FaderTarget::Aux(n) => 31 + n as i32,
        FaderTarget::FxRtn(n) => 39 + n as i32,
        FaderTarget::Bus(n) => 47 + n as i32,
        FaderTarget::Mtx(n) => 63 + n as i32,
        FaderTarget::Main => 70,
        FaderTarget::Dca(_) => -1,
    }
}

pub(crate) fn spawn_copy_paste(
    mixer_addr: SocketAddr,
    source: FaderTarget,
    target: FaderTarget,
) -> Task<Message> {
    Task::perform(
        async move {
            let src_index = target_to_ch_index(source);
            let dst_index = target_to_ch_index(target);
            if src_index < 0 || dst_index < 0 {
                return Err("Copy/paste not supported for DCAs".to_owned());
            }
            let probe =
                crate::ParameterProbe::new(mixer_addr).with_timeout(Duration::from_millis(1000));

            probe
                .set_multi(
                    "/save",
                    &[
                        OscValue::String("libchan".to_owned()),
                        OscValue::Int(99),
                        OscValue::String("mixosc_copy".to_owned()),
                        OscValue::Int(src_index),
                    ],
                )
                .map_err(|e| e.to_string())?;

            probe
                .set_multi(
                    "/load",
                    &[
                        OscValue::String("libchan".to_owned()),
                        OscValue::Int(99),
                        OscValue::Int(dst_index),
                    ],
                )
                .map_err(|e| e.to_string())?;
            Ok(())
        },
        Message::ParameterSetFinished,
    )
}

pub(crate) fn spawn_load_mute_groups(mixer_addr: SocketAddr, model: MixerModel) -> Task<Message> {
    let count = if model == MixerModel::X32 { 6 } else { 4 };
    let paths: Vec<String> = (1..=count).map(|n| format!("/config/mute/{n}")).collect();
    Task::perform(
        async move {
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(200))
                .load_batch(&paths)
                .map_err(|error| error.to_string())
        },
        Message::ParametersLoaded,
    )
}

pub(crate) fn spawn_load_panel_parameters(
    app: &StatusApp,
    mixer_addr: SocketAddr,
) -> Option<Task<Message>> {
    match app.active_view {
        AppView::Scenes => {
            let all_paths: Vec<String> = match app.mixer_model {
                MixerModel::X32 => {
                    let mut scene_paths: Vec<String> = (1..=100)
                        .map(|i| format!("/-show/showfile/scene/{i:03}/name"))
                        .chain((1..=100).map(|i| format!("/-show/showfile/scene/{i:03}/hasData")))
                        .chain((1..=100).map(|i| format!("/-show/showfile/scene/{i:03}/safes")))
                        .chain((1..=100).map(|i| format!("/-show/showfile/scene/{i:03}/notes")))
                        .collect();
                    let mut cue_paths: Vec<String> = (0..100)
                        .map(|i| format!("/-show/showfile/cue/{i:03}/name"))
                        .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/scene")))
                        .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/skip")))
                        .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/miditype")))
                        .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/midichan")))
                        .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/midipara1")))
                        .chain((0..100).map(|i| format!("/-show/showfile/cue/{i:03}/midipara2")))
                        .collect();
                    let mut snippet_paths: Vec<String> = (0..100)
                        .map(|i| format!("/-show/showfile/snippet/{i:03}/name"))
                        .chain((0..100).map(|i| format!("/-show/showfile/snippet/{i:03}/hasData")))
                        .collect();
                    scene_paths
                        .drain(..)
                        .chain(cue_paths.drain(..))
                        .chain(snippet_paths.drain(..))
                        .collect()
                }
                MixerModel::XR18 => {
                    let mut snap_paths: Vec<String> = (1..=64)
                        .map(|i| format!("/-snap/{i:02}/name"))
                        .chain((1..=64).map(|i| format!("/-snap/{i:02}/hasdata")))
                        .collect();
                    snap_paths.push("/-snap/index".to_owned());
                    snap_paths.push("/-snap/name".to_owned());
                    snap_paths
                }
            };
            return Some(Task::batch(
                all_paths
                    .chunks(75)
                    .map(|chunk| {
                        let chunk = chunk.to_vec();
                        Task::perform(
                            async move {
                                crate::ParameterProbe::new(mixer_addr)
                                    .with_timeout(Duration::from_millis(1500))
                                    .load_batch(&chunk)
                                    .map_err(|error| error.to_string())
                            },
                            Message::ParametersLoaded,
                        )
                    })
                    .collect::<Vec<_>>(),
            ));
        }
        AppView::Setup => {
            let paths: Vec<String> = [
                "/config/talk/enable",
                "/config/talk/source",
                "/config/talk/A/level",
                "/config/talk/A/latch",
                "/config/talk/A/dim",
                "/config/talk/A/destmap",
                "/config/talk/B/level",
                "/config/talk/B/latch",
                "/config/talk/B/dim",
                "/config/talk/B/destmap",
                "/config/osc/type",
                "/config/osc/f",
                "/config/osc/fsel",
                "/config/osc/level",
                "/config/osc/dest",
                "/config/solo/level",
                "/config/solo/source",
                "/config/solo/sourcetrim",
                "/config/solo/chmode",
                "/config/solo/busmode",
                "/config/solo/dcamode",
                "/config/solo/exclusive",
                "/config/solo/followsel",
                "/config/solo/followsolo",
                "/config/solo/dimatt",
                "/config/solo/dim",
                "/config/solo/mono",
                "/config/solo/delay",
                "/config/solo/delaytime",
                "/config/solo/masterctrl",
                "/config/solo/mute",
                "/config/solo/dimpfl",
                "/-stat/urec/state",
                "/-stat/urec/rtime",
                "/-stat/urec/etime",
                "/-stat/sends on fader",
                "/-stat/geqonfdr",
                "/-stat/geqpos",
                "/config/userctrl/A/color",
                "/config/userctrl/B/color",
                "/config/userctrl/C/color",
                "/config/mono/link",
                "/config/tape/autoplay",
                "/-prefs/ip/dhcp",
                "/-prefs/clocksource",
                "/-prefs/clockrate",
                "/-prefs/clockmode",
                "/-prefs/bright",
                "/-prefs/lcdcont",
                "/-prefs/ledbright",
                "/-prefs/lamp",
                "/-prefs/lampon",
                "/-prefs/confirm_general",
                "/-prefs/confirm_overwrite",
                "/-prefs/confirm_sceneload",
                "/-prefs/remote/enable",
                "/-prefs/remote/protocol",
                "/-prefs/remote/port",
                "/-prefs/card/UFifc",
                "/-prefs/card/UFmode",
                "/-prefs/fastFaders",
                "/-prefs/hardmute",
                "/-prefs/dcamute",
                "/-prefs/invertmutes",
                "/-prefs/safe_masterlevels",
                "/-prefs/viewrtn",
                "/-prefs/scene_advance",
                "/-prefs/haflags",
                "/-prefs/show_control",
                "/-prefs/rec_control",
            ]
            .iter()
            .map(|s| s.to_string())
            .chain(["A", "B", "C"].iter().flat_map(|layer| {
                (1..=4)
                    .map(move |enc| format!("/config/userctrl/{layer}/enc/{enc}"))
                    .chain((5..=12).map(move |btn| format!("/config/userctrl/{layer}/btn/{btn}")))
            }))
            .collect();
            return Some(Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(1500))
                        .load_batch(&paths)
                        .map_err(|error| error.to_string())
                },
                Message::ParametersLoaded,
            ));
        }
        AppView::Routing => {
            let paths: Vec<String> = match app.mixer_model {
                MixerModel::X32 => {
                    let mut p: Vec<String> = (1..=16)
                        .map(|n| format!("/config/chlink/{n:02}"))
                        .chain((1..=4).map(|n| format!("/config/auxlink/{n:02}")))
                        .chain((1..=8).map(|n| format!("/config/buslink/{n:02}")))
                        .chain((1..=4).map(|n| format!("/config/fxlink/{n:02}")))
                        .chain((1..=3).map(|n| format!("/config/mtxlink/{n:02}")))
                        .collect();
                    p.push("/config/linkcfg/hadly".to_owned());
                    p.push("/config/linkcfg/eq".to_owned());
                    p.push("/config/linkcfg/dyn".to_owned());
                    p.push("/config/linkcfg/fdrmute".to_owned());
                    p.extend((1..=32).map(|n| format!("/ch/{n:02}/config/source")));
                    p.extend((1..=8).map(|n| format!("/auxin/{n:02}/config/source")));
                    p.push("/config/routing/OUT/1-4".to_owned());
                    p.push("/config/routing/OUT/5-8".to_owned());
                    p.push("/config/routing/OUT/9-12".to_owned());
                    p.push("/config/routing/OUT/13-16".to_owned());
                    for out in 1..=16 {
                        p.push(format!("/outputs/main/{out:02}/delay/on"));
                        p.push(format!("/outputs/main/{out:02}/delay/time"));
                        p.push(format!("/outputs/main/{out:02}/src"));
                    }
                    for out in 1..=6 {
                        p.push(format!("/outputs/aux/{out:02}/src"));
                    }
                    p
                }
                MixerModel::XR18 => {
                    let mut p: Vec<String> = (1..=8)
                        .map(|n| format!("/config/chlink/{n:02}"))
                        .chain((1..=3).map(|n| format!("/config/buslink/{n:02}")))
                        .chain((1..=2).map(|n| format!("/config/fxlink/{n:02}")))
                        .collect();
                    p.push("/config/linkcfg/hadly".to_owned());
                    p.push("/config/linkcfg/eq".to_owned());
                    p.push("/config/linkcfg/dyn".to_owned());
                    p.push("/config/linkcfg/fdrmute".to_owned());
                    p.extend((1..=16).map(|n| format!("/ch/{n:02}/config/source")));
                    p.push("/config/routing/OUT/1-4".to_owned());
                    p.push("/config/routing/OUT/5-8".to_owned());
                    p.push("/config/routing/OUT/9-12".to_owned());
                    p.push("/config/routing/OUT/13-16".to_owned());
                    for out in 1..=6 {
                        p.push(format!("/outputs/main/{out:02}/delay/on"));
                        p.push(format!("/outputs/main/{out:02}/delay/time"));
                        p.push(format!("/outputs/main/{out:02}/src"));
                    }
                    for out in 1..=2 {
                        p.push(format!("/outputs/aux/{out:02}/src"));
                    }
                    p
                }
            };
            return Some(Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(1000))
                        .load_batch(&paths)
                        .map_err(|error| error.to_string())
                },
                Message::ParametersLoaded,
            ));
        }
        AppView::Rta => {
            let paths = vec![
                "/-prefs/rta/source".to_owned(),
                "/-prefs/rta/gain".to_owned(),
                "/-prefs/rta/autogain".to_owned(),
                "/-prefs/rta/decay".to_owned(),
                "/-prefs/rta/mode".to_owned(),
            ];
            return Some(Task::perform(
                async move {
                    crate::ParameterProbe::new(mixer_addr)
                        .with_timeout(Duration::from_millis(400))
                        .load_batch(&paths)
                        .map_err(|error| error.to_string())
                },
                Message::ParametersLoaded,
            ));
        }
        _ => {}
    }
    let paths = panel_parameter_paths(app)?;
    Some(Task::perform(
        async move {
            crate::ParameterProbe::new(mixer_addr)
                .with_timeout(Duration::from_millis(400))
                .load_batch(&paths)
                .map_err(|error| error.to_string())
        },
        Message::ParametersLoaded,
    ))
}

pub(crate) fn spawn_discovery() -> Task<Message> {
    Task::perform(
        async move {
            DiscoveryProbe::new()
                .with_timeout(Duration::from_millis(900))
                .discover()
                .map_err(|error| error.to_string())
        },
        Message::DiscoveryFinished,
    )
}

pub(crate) fn state_subscription(
    mixer_addr: SocketAddr,
    model: MixerModel,
    index: usize,
) -> Subscription<Message> {
    Subscription::run_with((mixer_addr, model, index), state_worker_indexed)
}

pub(crate) fn state_worker_indexed(
    (mixer_addr, model, index): &(SocketAddr, MixerModel, usize),
) -> BoxStream<'static, Message> {
    let mixer_addr = *mixer_addr;
    let model = *model;
    let index = *index;
    state_worker(&(mixer_addr, model))
        .map(move |r| Message::MixerEvent(index, Box::new(Message::ConsoleUpdateReceived(r))))
        .boxed()
}

pub(crate) fn mixer_addr_from_args_or_env() -> Option<SocketAddr> {
    let candidate = env::args()
        .nth(1)
        .or_else(|| env::var("MIXOSC_MIXER_ADDR").ok());

    candidate.and_then(|candidate| parse_target(&candidate).ok())
}

pub(crate) fn meter_subscription(
    mixer_addr: SocketAddr,
    model: MixerModel,
    index: usize,
) -> Subscription<Message> {
    Subscription::run_with((mixer_addr, model, index), meter_worker_indexed)
}

pub(crate) fn meter_worker_indexed(
    (mixer_addr, model, index): &(SocketAddr, MixerModel, usize),
) -> BoxStream<'static, Message> {
    let mixer_addr = *mixer_addr;
    let model = *model;
    let index = *index;
    meter_worker(&(mixer_addr, model))
        .map(move |r| Message::MixerEvent(index, Box::new(Message::MetersLoaded(r))))
        .boxed()
}

pub(crate) fn master_meter_subscription(
    mixer_addr: SocketAddr,
    model: MixerModel,
    index: usize,
) -> Subscription<Message> {
    Subscription::run_with((mixer_addr, model, index), master_meter_worker_indexed)
}

pub(crate) fn master_meter_worker_indexed(
    (mixer_addr, model, index): &(SocketAddr, MixerModel, usize),
) -> BoxStream<'static, Message> {
    let mixer_addr = *mixer_addr;
    let model = *model;
    let index = *index;
    master_meter_worker(&(mixer_addr, model))
        .map(move |r| {
            Message::MixerEvent(index, Box::new(Message::MasterMetersLoaded(Box::new(r))))
        })
        .boxed()
}

pub(crate) fn rta_meter_subscription(
    mixer_addr: SocketAddr,
    model: MixerModel,
    index: usize,
) -> Subscription<Message> {
    Subscription::run_with((mixer_addr, model, index), rta_meter_worker_indexed)
}

pub(crate) fn rta_meter_worker_indexed(
    (mixer_addr, model, index): &(SocketAddr, MixerModel, usize),
) -> BoxStream<'static, Message> {
    let mixer_addr = *mixer_addr;
    let model = *model;
    let index = *index;
    rta_meter_worker(&(mixer_addr, model))
        .map(move |r| Message::MixerEvent(index, Box::new(Message::RtaMetersLoaded(Box::new(r)))))
        .boxed()
}

pub(crate) fn state_worker(
    (mixer_addr, model): &(SocketAddr, MixerModel),
) -> BoxStream<'static, Result<ConsoleUpdate, String>> {
    let mixer_addr = *mixer_addr;
    let model = *model;
    stream::channel(
        64,
        move |mut output: mpsc::Sender<Result<ConsoleUpdate, String>>| async move {
            let socket = match bind_meter_socket().await {
                Ok(socket) => socket,
                Err(error) => {
                    let _ = output.send(Err(error.to_string())).await;
                    return;
                }
            };

            let (keepalive_request, keepalive_interval) = match model {
                MixerModel::X32 => (XREMOTE_REQUEST, Duration::from_secs(5)),
                MixerModel::XR18 => (XREMOTENFB_REQUEST, Duration::from_secs(3)),
            };

            if let Err(error) = socket.send_to(keepalive_request, mixer_addr).await {
                let _ = output
                    .send(Err(format!("failed to send keepalive: {error}")))
                    .await;
                return;
            }

            let mut last_keepalive = Instant::now();
            let mut buffer = [0_u8; 4096];

            loop {
                if last_keepalive.elapsed() >= keepalive_interval {
                    if let Err(error) = socket.send_to(keepalive_request, mixer_addr).await {
                        let _ = output
                            .send(Err(format!("failed to renew keepalive: {error}")))
                            .await;
                        return;
                    }
                    last_keepalive = Instant::now();
                }

                match tokio::time::timeout(
                    Duration::from_millis(250),
                    socket.recv_from(&mut buffer),
                )
                .await
                {
                    Ok(Ok((received, _))) => {
                        if let Some(update) = parse_console_update(&buffer[..received], model) {
                            let _ = output.send(Ok(update)).await;
                        }
                    }
                    Ok(Err(error)) => {
                        let _ = output
                            .send(Err(format!("failed while receiving state stream: {error}")))
                            .await;
                        return;
                    }
                    Err(_) => {}
                }
            }
        },
    )
    .boxed()
}

pub(crate) fn meter_worker(
    (mixer_addr, model): &(SocketAddr, MixerModel),
) -> BoxStream<'static, Result<Vec<StripMeter>, String>> {
    let mixer_addr = *mixer_addr;
    let model = *model;
    stream::channel(
        32,
        move |mut output: mpsc::Sender<Result<Vec<StripMeter>, String>>| async move {
            let socket = match bind_meter_socket().await {
                Ok(socket) => socket,
                Err(error) => {
                    let _ = output.send(Err(error.to_string())).await;
                    return;
                }
            };

            let (keepalive_request, subscribe, renew) = match model {
                MixerModel::X32 => (
                    XREMOTE_REQUEST,
                    batchsubscribe_meter_request("/meters/0", "/meters/0", 0, 0, 1),
                    Some(renew_request("/meters/0")),
                ),
                MixerModel::XR18 => (
                    XREMOTENFB_REQUEST,
                    osc_meter_group_request("/meters/1"),
                    None,
                ),
            };
            let keepalive_interval = match model {
                MixerModel::X32 => Duration::from_secs(5),
                MixerModel::XR18 => Duration::from_secs(3),
            };

            if model == MixerModel::XR18 {
                if let Err(error) = socket.send_to(b"/info\0\0\0", mixer_addr).await {
                    let _ = output
                        .send(Err(format!("failed to send /info handshake: {error}")))
                        .await;
                    return;
                }
                let mut handshake_buffer = [0_u8; 512];
                let _ = tokio::time::timeout(
                    Duration::from_millis(500),
                    socket.recv_from(&mut handshake_buffer),
                )
                .await;
            }

            if let Err(error) = socket.send_to(keepalive_request, mixer_addr).await {
                let _ = output
                    .send(Err(format!("failed to send keepalive: {error}")))
                    .await;
                return;
            }
            if let Err(error) = socket.send_to(&subscribe, mixer_addr).await {
                let _ = output
                    .send(Err(format!(
                        "failed to send meter subscription for {model:?}: {error}"
                    )))
                    .await;
                return;
            }

            let mut last_keepalive = Instant::now();
            let mut last_renew = renew.as_ref().map(|_| Instant::now());
            let mut buffer = [0_u8; 4096];

            loop {
                if last_keepalive.elapsed() >= keepalive_interval {
                    if let Err(error) = socket.send_to(keepalive_request, mixer_addr).await {
                        let _ = output
                            .send(Err(format!("failed to renew keepalive: {error}")))
                            .await;
                        return;
                    }
                    if let Err(error) = socket.send_to(&subscribe, mixer_addr).await {
                        let _ = output
                            .send(Err(format!(
                                "failed to resend meter subscription for {model:?}: {error}"
                            )))
                            .await;
                        return;
                    }
                    last_keepalive = Instant::now();
                }

                if let Some(ref renew_request) = renew
                    && last_renew.as_ref().unwrap().elapsed() >= Duration::from_secs(5)
                {
                    if let Err(error) = socket.send_to(renew_request, mixer_addr).await {
                        let _ = output
                            .send(Err(format!("failed to renew meter subscription: {error}")))
                            .await;
                        return;
                    }
                    last_renew = Some(Instant::now());
                }

                match tokio::time::timeout(
                    Duration::from_millis(250),
                    socket.recv_from(&mut buffer),
                )
                .await
                {
                    Ok(Ok((received, _))) => {
                        if let Ok(meters) = parse_input_meter_packet(&buffer[..received], model) {
                            let _ = output.send(Ok(meters)).await;
                        }
                    }
                    Ok(Err(error)) => {
                        let _ = output
                            .send(Err(format!("failed while receiving meter stream: {error}")))
                            .await;
                        return;
                    }
                    Err(_) => {}
                }

                sleep(Duration::from_millis(10)).await;
            }
        },
    )
    .boxed()
}

pub(crate) fn master_meter_worker(
    (mixer_addr, model): &(SocketAddr, MixerModel),
) -> BoxStream<'static, Result<MainMeterLevels, String>> {
    let mixer_addr = *mixer_addr;
    let model = *model;
    stream::channel(
        32,
        move |mut output: mpsc::Sender<Result<MainMeterLevels, String>>| async move {
            let socket = match bind_meter_socket().await {
                Ok(socket) => socket,
                Err(error) => {
                    let _ = output.send(Err(error.to_string())).await;
                    return;
                }
            };

            let (keepalive_request, subscribe, renew) = match model {
                MixerModel::X32 => (
                    XREMOTE_REQUEST,
                    batchsubscribe_meter_request("/meters/2", "/meters/2", 0, 0, 1),
                    Some(renew_request("/meters/2")),
                ),
                MixerModel::XR18 => (
                    XREMOTENFB_REQUEST,
                    osc_meter_group_request("/meters/1"),
                    None,
                ),
            };
            let keepalive_interval = match model {
                MixerModel::X32 => Duration::from_secs(5),
                MixerModel::XR18 => Duration::from_secs(3),
            };

            if model == MixerModel::XR18 {
                if let Err(error) = socket.send_to(b"/info\0\0\0", mixer_addr).await {
                    let _ = output
                        .send(Err(format!("failed to send /info handshake: {error}")))
                        .await;
                    return;
                }
                let mut handshake_buffer = [0_u8; 512];
                let _ = tokio::time::timeout(
                    Duration::from_millis(500),
                    socket.recv_from(&mut handshake_buffer),
                )
                .await;
            }

            if let Err(error) = socket.send_to(keepalive_request, mixer_addr).await {
                let _ = output
                    .send(Err(format!("failed to send keepalive: {error}")))
                    .await;
                return;
            }
            if let Err(error) = socket.send_to(&subscribe, mixer_addr).await {
                let _ = output
                    .send(Err(format!(
                        "failed to send main meter subscription for {model:?}: {error}"
                    )))
                    .await;
                return;
            }

            let mut last_keepalive = Instant::now();
            let mut last_renew = renew.as_ref().map(|_| Instant::now());
            let mut buffer = [0_u8; 4096];

            loop {
                if last_keepalive.elapsed() >= keepalive_interval {
                    if let Err(error) = socket.send_to(keepalive_request, mixer_addr).await {
                        let _ = output
                            .send(Err(format!("failed to renew keepalive: {error}")))
                            .await;
                        return;
                    }
                    if let Err(error) = socket.send_to(&subscribe, mixer_addr).await {
                        let _ = output
                            .send(Err(format!(
                                "failed to resend main meter subscription for {model:?}: {error}"
                            )))
                            .await;
                        return;
                    }
                    last_keepalive = Instant::now();
                }

                if let Some(ref renew_request) = renew
                    && last_renew.as_ref().unwrap().elapsed() >= Duration::from_secs(5)
                {
                    if let Err(error) = socket.send_to(renew_request, mixer_addr).await {
                        let _ = output
                            .send(Err(format!(
                                "failed to renew main meter subscription: {error}"
                            )))
                            .await;
                        return;
                    }
                    last_renew = Some(Instant::now());
                }

                match tokio::time::timeout(
                    Duration::from_millis(250),
                    socket.recv_from(&mut buffer),
                )
                .await
                {
                    Ok(Ok((received, _))) => {
                        if let Ok(levels) = parse_main_meter_packet(&buffer[..received], model) {
                            let _ = output.send(Ok(levels)).await;
                        }
                    }
                    Ok(Err(error)) => {
                        let _ = output
                            .send(Err(format!(
                                "failed while receiving main meter stream: {error}"
                            )))
                            .await;
                        return;
                    }
                    Err(_) => {}
                }

                sleep(Duration::from_millis(10)).await;
            }
        },
    )
    .boxed()
}

pub(crate) fn rta_meter_worker(
    (mixer_addr, model): &(SocketAddr, MixerModel),
) -> BoxStream<'static, Result<[f32; 100], String>> {
    let mixer_addr = *mixer_addr;
    let model = *model;
    stream::channel(
        32,
        move |mut output: mpsc::Sender<Result<[f32; 100], String>>| async move {
            let socket = match bind_meter_socket().await {
                Ok(socket) => socket,
                Err(error) => {
                    let _ = output.send(Err(error.to_string())).await;
                    return;
                }
            };

            let subscribe = batchsubscribe_meter_request("/meters/15", "/meters/15", 0, 0, 1);
            if let Err(error) = socket.send_to(XREMOTE_REQUEST, mixer_addr).await {
                let _ = output
                    .send(Err(format!("failed to send /xremote: {error}")))
                    .await;
                return;
            }
            if let Err(error) = socket.send_to(&subscribe, mixer_addr).await {
                let _ = output
                    .send(Err(format!(
                        "failed to send /batchsubscribe for /meters/15: {error}"
                    )))
                    .await;
                return;
            }

            let mut last_renew = Instant::now();
            let mut buffer = [0_u8; 4096];

            loop {
                if last_renew.elapsed() >= Duration::from_secs(5) {
                    let renew = renew_request("/meters/15");
                    if let Err(error) = socket.send_to(&renew, mixer_addr).await {
                        let _ = output
                            .send(Err(format!(
                                "failed to renew meter stream /meters/15: {error}"
                            )))
                            .await;
                        return;
                    }
                    last_renew = Instant::now();
                }

                match tokio::time::timeout(
                    Duration::from_millis(250),
                    socket.recv_from(&mut buffer),
                )
                .await
                {
                    Ok(Ok((received, _))) => {
                        if let Ok(levels) = parse_rta_meter_packet(&buffer[..received], model) {
                            let _ = output.send(Ok(levels)).await;
                        }
                    }
                    Ok(Err(error)) => {
                        let _ = output
                            .send(Err(format!(
                                "failed while receiving rta meter stream: {error}"
                            )))
                            .await;
                        return;
                    }
                    Err(_) => {}
                }

                sleep(Duration::from_millis(10)).await;
            }
        },
    )
    .boxed()
}

pub(crate) async fn bind_meter_socket() -> std::io::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0))).await?;
    Ok(socket)
}
