use std::net::SocketAddr;

use crate::message::{AppView, ConnectionStatus, SelectedStrip};
use crate::model::{
    DiscoveredMixer, FaderTarget, GainSource, MixerConfig, MixerModel, config_for_model,
};
use crate::parameters::OscValue;

pub(crate) const MAX_STRIP_COUNT: usize = 74;
pub(crate) const MAX_SEND_BUS_COUNT: usize = 16;

#[derive(Debug)]
pub struct MixOscApp {
    pub(crate) mixers: Vec<StatusApp>,
    pub(crate) active_mixer: usize,
    pub(crate) discovered_mixers: Vec<DiscoveredMixer>,
    pub(crate) manual_target: bool,
    pub(crate) discovery_in_flight: bool,
}

impl Default for MixOscApp {
    fn default() -> Self {
        Self {
            mixers: vec![StatusApp::default()],
            active_mixer: 0,
            discovered_mixers: Vec::new(),
            manual_target: false,
            discovery_in_flight: false,
        }
    }
}

#[derive(Debug)]
pub struct StatusApp {
    pub(crate) mixer_addr: Option<SocketAddr>,
    pub(crate) discovered_mixer: Option<DiscoveredMixer>,
    pub(crate) probe_in_flight: bool,
    pub(crate) names: [Option<String>; MAX_STRIP_COUNT],
    pub(crate) colors: [Option<u8>; MAX_STRIP_COUNT],
    pub(crate) gains: [Option<f32>; MAX_STRIP_COUNT],
    pub(crate) gain_sources: [GainSource; MAX_STRIP_COUNT],
    pub(crate) gain_drag_values: [Option<f32>; MAX_STRIP_COUNT],
    pub(crate) sends: [[Option<f32>; MAX_SEND_BUS_COUNT]; MAX_STRIP_COUNT],
    pub(crate) pans: [Option<f32>; MAX_STRIP_COUNT],
    pub(crate) faders: [Option<f32>; MAX_STRIP_COUNT],
    pub(crate) meters_db: [f32; MAX_STRIP_COUNT],
    pub(crate) master_meters_db: [f32; 2],
    pub(crate) rta_meters_db: [f32; 100],
    pub(crate) muted: [Option<bool>; MAX_STRIP_COUNT],
    pub(crate) soloed: [Option<bool>; MAX_STRIP_COUNT],
    pub(crate) master_fader: Option<f32>,
    pub(crate) master_muted: Option<bool>,
    pub(crate) master_soloed: Option<bool>,
    pub(crate) master_color: Option<u8>,
    pub(crate) active_view: AppView,
    pub(crate) selected_strip: Option<SelectedStrip>,
    pub(crate) status: ConnectionStatus,
    pub(crate) last_error: Option<String>,
    pub(crate) parameter_values: std::collections::HashMap<String, OscValue>,
    pub(crate) editing_name: Option<(usize, String)>,
    pub(crate) editing_scene: Option<(usize, String)>,
    pub(crate) copy_buffer: Option<FaderTarget>,
    pub(crate) dca_spill: Option<u8>,
    pub(crate) mute_spill: Option<u8>,
    pub(crate) show_file_name: String,
    pub(crate) editing_scene_safes: Option<i32>,
    pub(crate) editing_snippet_filters: Option<i32>,
    pub(crate) mixer_model: MixerModel,
}

impl Default for StatusApp {
    fn default() -> Self {
        Self {
            mixer_addr: None,
            discovered_mixer: None,
            probe_in_flight: false,
            names: std::array::from_fn(|_| None),
            colors: [None; MAX_STRIP_COUNT],
            gains: [None; MAX_STRIP_COUNT],
            gain_sources: [GainSource::Trim; MAX_STRIP_COUNT],
            gain_drag_values: [None; MAX_STRIP_COUNT],
            sends: [[None; MAX_SEND_BUS_COUNT]; MAX_STRIP_COUNT],
            pans: [None; MAX_STRIP_COUNT],
            faders: [None; MAX_STRIP_COUNT],
            meters_db: [-90.0; MAX_STRIP_COUNT],
            master_meters_db: [-90.0, -90.0],
            rta_meters_db: [-128.0; 100],
            muted: [None; MAX_STRIP_COUNT],
            soloed: [None; MAX_STRIP_COUNT],
            master_fader: None,
            master_muted: None,
            master_soloed: None,
            master_color: None,
            active_view: AppView::Mixer,
            selected_strip: Some(SelectedStrip::Strip(0)),
            status: ConnectionStatus::Disconnected,
            last_error: None,
            parameter_values: std::collections::HashMap::new(),
            editing_name: None,
            editing_scene: None,
            copy_buffer: None,
            dca_spill: None,
            mute_spill: None,
            show_file_name: String::new(),
            editing_scene_safes: None,
            editing_snippet_filters: None,
            mixer_model: MixerModel::X32,
        }
    }
}

impl StatusApp {
    pub(crate) fn config(&self) -> MixerConfig {
        config_for_model(self.mixer_model)
    }

    pub(crate) fn visible_strips(&self) -> &[FaderTarget] {
        self.config().visible_strips
    }

    pub(crate) fn send_buses(&self) -> &[u8] {
        self.config().send_buses
    }

    pub(crate) fn matrix_sends(&self) -> &[u8] {
        self.config().matrix_sends
    }
}
