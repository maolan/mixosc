use crate::message::{AppView, SelectedStrip};
use crate::model::{FaderTarget, GainSource, MixerModel};
use crate::state::StatusApp;
use crate::ui::format::*;
use crate::ui::panel_channel::eq_band_count;

pub(crate) fn panel_parameter_paths(app: &StatusApp) -> Option<Vec<String>> {
    let selected = app.selected_strip?;
    let (index, target, base) = match selected {
        SelectedStrip::Strip(index) => {
            let target = app.visible_strips()[index];
            (index, target, strip_base_path(target, app.mixer_model))
        }
        SelectedStrip::Master => {
            let base = main_base_path(app.mixer_model);
            return match app.active_view {
                AppView::Eq => {
                    let mut p = vec![format!("{base}/eq/on")];
                    for band in 1..=6 {
                        p.push(format!("{base}/eq/{band:02}/on"));
                        p.push(format!("{base}/eq/{band:02}/f"));
                        p.push(format!("{base}/eq/{band:02}/g"));
                        p.push(format!("{base}/eq/{band:02}/q"));
                    }
                    Some(p)
                }
                AppView::Dyn => Some(vec![
                    format!("{base}/dyn/on"),
                    format!("{base}/dyn/thr"),
                    format!("{base}/dyn/ratio"),
                    format!("{base}/dyn/knee"),
                    format!("{base}/dyn/mgain"),
                    format!("{base}/dyn/attack"),
                    format!("{base}/dyn/hold"),
                    format!("{base}/dyn/release"),
                    format!("{base}/dyn/mix"),
                ]),
                AppView::Config => {
                    let mut paths = vec![
                        format!("{base}/insert/on"),
                        format!("{base}/insert/pos"),
                        format!("{base}/insert/sel"),
                    ];
                    if app.mixer_model == MixerModel::X32 {
                        paths.push("/main/m/insert/on".to_string());
                        paths.push("/main/m/insert/pos".to_string());
                        paths.push("/main/m/insert/sel".to_string());
                    }
                    Some(paths)
                }
                AppView::Sends => {
                    let mut p = Vec::new();
                    if app.mixer_model == MixerModel::X32 {
                        for mtx in 1..=6 {
                            p.push(format!("{base}/mix/{mtx:02}/level"));
                            p.push(format!("{base}/mix/{mtx:02}/on"));
                        }
                        for mtx in (1..=6).step_by(2) {
                            p.push(format!("{base}/mix/{mtx:02}/pan"));
                        }
                    }
                    p.push(format!("{base}/mix/on"));
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/pan"));
                    if app.mixer_model == MixerModel::X32 {
                        p.push("/main/m/mix/on".to_string());
                        p.push("/main/m/mix/fader".to_string());
                    }
                    Some(p)
                }
                AppView::Main => {
                    let mut paths = vec![
                        format!("{base}/mix/on"),
                        format!("{base}/mix/fader"),
                        format!("{base}/mix/pan"),
                    ];
                    if app.mixer_model == MixerModel::X32 {
                        paths.push("/main/m/mix/on".to_string());
                        paths.push("/main/m/mix/fader".to_string());
                    }
                    Some(paths)
                }
                _ => None,
            };
        }
    };

    let paths: Vec<String> = match app.active_view {
        AppView::Eq => {
            let bands = eq_band_count(target);
            if bands == 0 {
                return None;
            }
            let mut p = vec![format!("{base}/eq/on")];
            for band in 1..=bands {
                p.push(format!("{base}/eq/{band:02}/on"));
                p.push(format!("{base}/eq/{band:02}/f"));
                p.push(format!("{base}/eq/{band:02}/g"));
                p.push(format!("{base}/eq/{band:02}/q"));
            }
            p
        }
        AppView::Gate => {
            if !matches!(target, FaderTarget::Channel(_)) {
                return None;
            }
            vec![
                format!("{base}/gate/on"),
                format!("{base}/gate/mode"),
                format!("{base}/gate/auto"),
                format!("{base}/gate/thr"),
                format!("{base}/gate/range"),
                format!("{base}/gate/attack"),
                format!("{base}/gate/hold"),
                format!("{base}/gate/release"),
                format!("{base}/gate/keysrc"),
                format!("{base}/gate/filter/on"),
                format!("{base}/gate/filter/type"),
                format!("{base}/gate/filter/f"),
            ]
        }
        AppView::Dyn => {
            if matches!(
                target,
                FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Dca(_)
            ) {
                return None;
            }
            vec![
                format!("{base}/dyn/on"),
                format!("{base}/dyn/thr"),
                format!("{base}/dyn/ratio"),
                format!("{base}/dyn/knee"),
                format!("{base}/dyn/mgain"),
                format!("{base}/dyn/attack"),
                format!("{base}/dyn/hold"),
                format!("{base}/dyn/release"),
                format!("{base}/dyn/mix"),
                format!("{base}/dyn/auto"),
                format!("{base}/dyn/mode"),
                format!("{base}/dyn/det"),
                format!("{base}/dyn/env"),
                format!("{base}/dyn/pos"),
                format!("{base}/dyn/keysrc"),
                format!("{base}/dyn/filter/on"),
                format!("{base}/dyn/filter/type"),
                format!("{base}/dyn/filter/f"),
            ]
        }
        AppView::Config => {
            let mut p = Vec::new();
            match target {
                FaderTarget::Channel(_) | FaderTarget::Aux(_) => {
                    p.push(format!("{base}/preamp/trim"));
                    p.push(format!("{base}/preamp/invert"));
                    if matches!(target, FaderTarget::Channel(_)) {
                        p.push(format!("{base}/preamp/hpon"));
                        p.push(format!("{base}/preamp/hpf"));
                        p.push(format!("{base}/preamp/hpslope"));
                    }
                    p.push(format!("{base}/delay/on"));
                    p.push(format!("{base}/delay/time"));
                }
                FaderTarget::Mtx(_) => {
                    p.push(format!("{base}/preamp/invert"));
                }
                _ => {}
            }
            if matches!(
                target,
                FaderTarget::Channel(_)
                    | FaderTarget::Bus(_)
                    | FaderTarget::Mtx(_)
                    | FaderTarget::Main
            ) {
                p.push(format!("{base}/insert/on"));
                p.push(format!("{base}/insert/pos"));
                p.push(format!("{base}/insert/sel"));
            }
            if let FaderTarget::Channel(ch) = target {
                let headamp_index = match app.gain_sources[index] {
                    GainSource::Headamp(idx) => idx,
                    _ => ch - 1,
                };
                let (hp, hg) = match app.mixer_model {
                    MixerModel::X32 => (
                        format!("/headamp/{headamp_index:03}/phantom"),
                        format!("/headamp/{headamp_index:03}/gain"),
                    ),
                    MixerModel::XR18 => (
                        format!("/headamp/{headamp_index:02}/phantom"),
                        format!("/headamp/{headamp_index:02}/gain"),
                    ),
                };
                p.push(hp);
                p.push(hg);
            }
            if matches!(
                target,
                FaderTarget::Channel(_)
                    | FaderTarget::Aux(_)
                    | FaderTarget::FxRtn(_)
                    | FaderTarget::Bus(_)
            ) {
                p.push(format!("{base}/grp/dca"));
                p.push(format!("{base}/grp/mute"));
            }
            if app.mixer_model == MixerModel::X32
                && let FaderTarget::Channel(ch) = target
                && ch <= 8
            {
                p.push(format!("{base}/amix/on"));
                p.push(format!("{base}/amix/group"));
                p.push(format!("{base}/amix/weight"));
            }
            if !matches!(target, FaderTarget::Main) {
                p.push(format!("{base}/config/color"));
                if !matches!(target, FaderTarget::Dca(_)) {
                    p.push(format!("{base}/config/icon"));
                }
            }
            if p.is_empty() {
                return None;
            }
            p
        }
        AppView::Sends => {
            let mut p = Vec::new();
            match target {
                FaderTarget::Channel(_) | FaderTarget::Aux(_) | FaderTarget::FxRtn(_) => {
                    let send_count = if app.mixer_model == MixerModel::X32 {
                        16
                    } else {
                        6
                    };
                    for bus in 1..=send_count {
                        p.push(format!("{base}/mix/{bus:02}/level"));
                        p.push(format!("{base}/mix/{bus:02}/on"));
                    }
                    for bus in (1..=send_count).step_by(2) {
                        p.push(format!("{base}/mix/{bus:02}/pan"));
                        p.push(format!("{base}/mix/{bus:02}/type"));
                    }
                    for bus in (1..=send_count).step_by(2) {
                        let bus_st_path = if app.mixer_model == MixerModel::X32 {
                            format!("/bus/{bus:02}/mix/st")
                        } else {
                            format!("/bus/{bus}/mix/st")
                        };
                        p.push(bus_st_path);
                    }
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/st"));
                    p.push(format!("{base}/mix/pan"));
                    if app.mixer_model == MixerModel::X32 {
                        p.push(format!("{base}/mix/mono"));
                        p.push(format!("{base}/mix/mlevel"));
                    }
                }
                FaderTarget::Bus(_) => {
                    if app.mixer_model == MixerModel::X32 {
                        for mtx in 1..=6 {
                            p.push(format!("{base}/mix/{mtx:02}/level"));
                            p.push(format!("{base}/mix/{mtx:02}/on"));
                        }
                        for mtx in (1..=6).step_by(2) {
                            p.push(format!("{base}/mix/{mtx:02}/pan"));
                        }
                        for mtx in (1..=6).step_by(2) {
                            p.push(format!("/mtx/{mtx:02}/mix/st"));
                        }
                        p.push(format!("{base}/mix/mono"));
                        p.push(format!("{base}/mix/mlevel"));
                    }
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/st"));
                    p.push(format!("{base}/mix/pan"));
                }
                FaderTarget::Mtx(_) => {
                    p.push(format!("{base}/mix/on"));
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/pan"));
                }
                FaderTarget::Dca(_) => {
                    p.push(format!("{base}/on"));
                    p.push(format!("{base}/fader"));
                }
                FaderTarget::Main => {}
            }
            p
        }
        AppView::Main => {
            let mut p = Vec::new();
            match target {
                FaderTarget::Channel(_)
                | FaderTarget::Aux(_)
                | FaderTarget::FxRtn(_)
                | FaderTarget::Bus(_) => {
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/st"));
                    p.push(format!("{base}/mix/pan"));
                    p.push(format!("{base}/mix/mono"));
                    p.push(format!("{base}/mix/mlevel"));
                }
                FaderTarget::Mtx(_) => {
                    p.push(format!("{base}/mix/on"));
                    p.push(format!("{base}/mix/fader"));
                    p.push(format!("{base}/mix/pan"));
                }
                FaderTarget::Dca(_) => {
                    p.push(format!("{base}/on"));
                    p.push(format!("{base}/fader"));
                }
                FaderTarget::Main => {}
            }
            p
        }
        AppView::Fx => {
            let mut p = Vec::new();
            let fx_slots = if app.mixer_model == MixerModel::X32 {
                8
            } else {
                4
            };
            for slot in 1..=fx_slots {
                let fx_base = format!("/fx/{slot:02}");
                p.push(format!("{fx_base}/type"));
                if slot <= 4 {
                    p.push(format!("{fx_base}/source/l"));
                    p.push(format!("{fx_base}/source/r"));
                }
                for par in 1..=8 {
                    p.push(format!("{fx_base}/par/{par:02}"));
                }
            }
            p
        }
        _ => return None,
    };

    Some(paths)
}
