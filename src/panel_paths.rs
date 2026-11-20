use crate::message::{AppView, SelectedStrip};
use crate::model::{FaderTarget, GainSource, MixerModel};
use crate::state::StatusApp;
use crate::ui::format::*;
use crate::ui::panel_channel::eq_band_count;

fn append_insert_fx_paths(app: &StatusApp, paths: &mut Vec<String>, base: &str) {
    if app.mixer_model != MixerModel::X32 {
        return;
    }
    let insert = match app.parameter_values.get(&format!("{base}/insert/sel")) {
        Some(crate::parameters::OscValue::Int(value)) => *value,
        _ => 0,
    };
    if !(1..=16).contains(&insert) {
        return;
    }
    let slot = (insert as u8).div_ceil(2);
    let fx_base = format!("/fx/{slot:02}");
    paths.push(format!("{fx_base}/type"));
    paths.extend((1..=64).map(|parameter| format!("{fx_base}/par/{parameter:02}")));
}

pub(crate) fn panel_parameter_paths(app: &StatusApp) -> Option<Vec<String>> {
    if app.active_view == AppView::Fx {
        let fx_slots = if app.mixer_model == MixerModel::X32 {
            8
        } else {
            4
        };
        let parameter_count = if app.mixer_model == MixerModel::X32 {
            64
        } else {
            8
        };
        let mut paths = Vec::new();
        for slot in 1..=fx_slots {
            let fx_base = format!("/fx/{slot:02}");
            paths.push(format!("{fx_base}/type"));
            if slot <= 4 {
                paths.push(format!("{fx_base}/source/l"));
                paths.push(format!("{fx_base}/source/r"));
            }
            for par in 1..=parameter_count {
                paths.push(format!("{fx_base}/par/{par:02}"));
            }
        }
        return Some(paths);
    }

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
                        p.push(format!("{base}/eq/{band}/type"));
                        p.push(format!("{base}/eq/{band}/f"));
                        p.push(format!("{base}/eq/{band}/g"));
                        p.push(format!("{base}/eq/{band}/q"));
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
                    append_insert_fx_paths(app, &mut paths, &base);
                    append_insert_fx_paths(app, &mut paths, "/main/m");
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
        AppView::Channel => {
            let mut paths = Vec::new();
            if matches!(target, FaderTarget::Channel(_)) {
                paths.push(format!("{base}/gate/on"));
                paths.push(format!("{base}/gate/thr"));
            }
            let bands = eq_band_count(target);
            if bands > 0 {
                paths.push(format!("{base}/eq/on"));
            }
            if !matches!(
                target,
                FaderTarget::Aux(_) | FaderTarget::FxRtn(_) | FaderTarget::Dca(_)
            ) {
                paths.push(format!("{base}/dyn/on"));
                paths.push(format!("{base}/dyn/thr"));
                paths.push(format!("{base}/dyn/ratio"));
            }
            if matches!(
                target,
                FaderTarget::Channel(_)
                    | FaderTarget::Aux(_)
                    | FaderTarget::FxRtn(_)
                    | FaderTarget::Bus(_)
            ) {
                paths.push(format!("{base}/grp/dca"));
                paths.push(format!("{base}/grp/mute"));
            }
            if paths.is_empty() {
                return None;
            }
            paths
        }
        AppView::Eq => {
            let bands = eq_band_count(target);
            if bands == 0 {
                return None;
            }
            let mut p = vec![format!("{base}/eq/on")];
            for band in 1..=bands {
                p.push(format!("{base}/eq/{band}/type"));
                p.push(format!("{base}/eq/{band}/f"));
                p.push(format!("{base}/eq/{band}/g"));
                p.push(format!("{base}/eq/{band}/q"));
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
                append_insert_fx_paths(app, &mut p, &base);
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
        _ => return None,
    };

    Some(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x32_fx_loads_every_parameter_even_with_master_selected() {
        let app = StatusApp {
            active_view: AppView::Fx,
            selected_strip: Some(SelectedStrip::Master),
            ..Default::default()
        };

        let paths = panel_parameter_paths(&app).expect("FX parameter paths");

        assert_eq!(paths.len(), 528);
        assert!(paths.contains(&"/fx/01/source/l".to_owned()));
        assert!(paths.contains(&"/fx/08/par/64".to_owned()));
    }

    #[test]
    fn channel_view_loads_aux_eq_and_sends_integer_enable() {
        let app = StatusApp {
            active_view: AppView::Channel,
            selected_strip: Some(SelectedStrip::Strip(32)),
            ..Default::default()
        };

        let paths = panel_parameter_paths(&app).expect("Channel parameter paths");

        assert!(paths.contains(&"/auxin/01/eq/on".to_owned()));
        assert!(!paths.contains(&"/auxin/01/eq/1/on".to_owned()));
        assert!(!paths.contains(&"/auxin/01/eq/4/on".to_owned()));
        assert!(!paths.iter().any(|path| path.contains("/dyn/")));
        assert_eq!(
            crate::parameters::path::auxin_eq_f(1, 1),
            "/auxin/01/eq/1/f"
        );
        assert_eq!(
            crate::parameters::AuxInParam::EqQ { index: 4 }.path(1),
            "/auxin/01/eq/4/q"
        );

        let packet = crate::parameters::build_set(
            "/auxin/01/eq/on",
            crate::parameters::OscValue::Bool(true),
        );
        assert_eq!(
            crate::parameters::parse_osc_value(&packet),
            Some((
                "/auxin/01/eq/on".to_owned(),
                crate::parameters::OscValue::Int(1),
            ))
        );
    }

    #[test]
    fn config_loads_current_channel_insert_effect() {
        let mut app = StatusApp {
            active_view: AppView::Config,
            selected_strip: Some(SelectedStrip::Strip(0)),
            ..Default::default()
        };
        app.parameter_values.insert(
            "/ch/01/insert/sel".to_owned(),
            crate::parameters::OscValue::Int(16),
        );

        let paths = panel_parameter_paths(&app).expect("Config parameter paths");

        assert!(paths.contains(&"/fx/08/type".to_owned()));
        assert!(paths.contains(&"/fx/08/par/01".to_owned()));
        assert!(paths.contains(&"/fx/08/par/64".to_owned()));
        assert!(!paths.iter().any(|path| path.starts_with("/fx/07/")));
    }

    #[test]
    fn config_loads_both_main_insert_effects() {
        let mut app = StatusApp {
            active_view: AppView::Config,
            selected_strip: Some(SelectedStrip::Master),
            ..Default::default()
        };
        for (path, insert) in [("/main/st/insert/sel", 1), ("/main/m/insert/sel", 3)] {
            app.parameter_values
                .insert(path.to_owned(), crate::parameters::OscValue::Int(insert));
        }

        let paths = panel_parameter_paths(&app).expect("Config parameter paths");

        assert!(paths.contains(&"/fx/01/type".to_owned()));
        assert!(paths.contains(&"/fx/01/par/64".to_owned()));
        assert!(paths.contains(&"/fx/02/type".to_owned()));
        assert!(paths.contains(&"/fx/02/par/64".to_owned()));
    }

    #[test]
    fn master_eq_loads_band_types() {
        let app = StatusApp {
            active_view: AppView::Eq,
            selected_strip: Some(SelectedStrip::Master),
            ..Default::default()
        };

        let paths = panel_parameter_paths(&app).expect("EQ parameter paths");

        assert_eq!(paths.len(), 25);
        assert!(paths.contains(&"/main/st/eq/1/type".to_owned()));
        assert!(paths.contains(&"/main/st/eq/6/type".to_owned()));
    }
}
