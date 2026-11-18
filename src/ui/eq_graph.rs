use crate::message::Message;
use crate::parameters::OscValue;
use crate::state::StatusApp;
use crate::ui::format::{format_hz, logf_value};
use crate::ui::widgets::{param_float, param_int};
use maolan_widgets::iced::widget::canvas::{
    self, Action as CanvasAction, Frame, Geometry, Path, Program, Stroke, Text,
};
use maolan_widgets::iced::{
    Color, Element, Event, Length, Point, Rectangle, Renderer, Theme, mouse,
};
use std::time::{Duration, Instant};

const MIN_GAIN_DB: f32 = -15.0;
const MAX_GAIN_DB: f32 = 15.0;
const BAND_COLORS: [Color; 6] = [
    Color::from_rgb8(0x29, 0xE6, 0xF2),
    Color::from_rgb8(0x7D, 0xD3, 0xA7),
    Color::from_rgb8(0xF0, 0xC8, 0x64),
    Color::from_rgb8(0xE8, 0x7D, 0xC8),
    Color::from_rgb8(0x75, 0xA7, 0xFF),
    Color::from_rgb8(0xF0, 0x7C, 0x82),
];

#[derive(Debug, Clone, Copy)]
struct EqBand {
    index: u8,
    frequency: f32,
    gain: f32,
    q: f32,
    kind: i32,
}

#[derive(Clone)]
pub(crate) struct EqGraph {
    base: String,
    bands: Vec<EqBand>,
}

#[derive(Default)]
pub(crate) struct EqGraphState {
    dragging: Option<usize>,
    last_click: Option<(usize, Instant, Point)>,
}

impl EqGraph {
    pub(crate) fn new(app: &StatusApp, base: String, band_count: u8) -> Self {
        let bands = (1..=band_count)
            .map(|index| {
                let band_base = format!("{base}/eq/{index}");
                EqBand {
                    index,
                    frequency: param_float(app, &format!("{band_base}/f")),
                    gain: param_float(app, &format!("{band_base}/g")),
                    q: param_float(app, &format!("{band_base}/q")),
                    kind: param_int(app, &format!("{band_base}/type")),
                }
            })
            .collect();
        Self { base, bands }
    }

    fn band_path(&self, path: &str, band: EqBand) -> String {
        format!("{}/eq/{}/{}", self.base, band.index, path)
    }

    fn hit_band(&self, point: Point, bounds: Rectangle) -> Option<usize> {
        let mut closest = None;
        let mut closest_distance = 14.0_f32.powi(2);
        for (index, band) in self.bands.iter().enumerate() {
            let node = band_point(*band, bounds);
            let distance = (point.x - node.x).powi(2) + (point.y - node.y).powi(2);
            if distance <= closest_distance {
                closest_distance = distance;
                closest = Some(index);
            }
        }
        closest
    }
}

pub(crate) fn eq_graph(app: &StatusApp, base: String, band_count: u8) -> Element<'_, Message> {
    canvas::Canvas::new(EqGraph::new(app, base, band_count))
        .width(Length::Fill)
        .height(Length::Fixed(240.0))
        .into()
}

impl Program<Message> for EqGraph {
    type State = EqGraphState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<CanvasAction<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let point = cursor.position_in(bounds)?;
                let band_index = self.hit_band(point, bounds)?;
                let now = Instant::now();
                if is_double_click(state.last_click, band_index, now, point) {
                    let band = self.bands.get(band_index).copied()?;
                    let (frequency, gain, q) = reset_values(self.bands.len(), band.index);
                    state.dragging = None;
                    state.last_click = None;
                    return Some(
                        CanvasAction::publish(Message::EqBandReset {
                            base: self.base.clone(),
                            band: band.index,
                            frequency,
                            gain,
                            q,
                        })
                        .and_capture(),
                    );
                }
                state.last_click = Some((band_index, now, point));
                state.dragging = Some(band_index);
                Some(CanvasAction::capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging.take().is_some() {
                    Some(CanvasAction::capture())
                } else {
                    None
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let band_index = state.dragging?;
                let point = cursor.position_in(bounds)?;
                let band = self.bands.get(band_index).copied()?;
                let frequency = quantize(point.x / bounds.width, 200);
                let gain = quantize(1.0 - point.y / bounds.height, 120);
                Some(
                    CanvasAction::publish(Message::EqBandChanged {
                        base: self.base.clone(),
                        band: band.index,
                        frequency,
                        gain,
                    })
                    .and_capture(),
                )
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let point = cursor.position_in(bounds)?;
                let band_index = self.hit_band(point, bounds)?;
                let band = self.bands.get(band_index).copied()?;
                let step = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 50.0,
                };
                let q = scrolled_q(band.q, step);
                Some(
                    CanvasAction::publish(Message::ParameterChanged(
                        self.band_path("q", band),
                        OscValue::Float(q),
                    ))
                    .and_capture(),
                )
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill(
            &Path::rectangle(Point::ORIGIN, bounds.size()),
            Color::from_rgb8(0x14, 0x16, 0x1C),
        );
        draw_grid(&mut frame, bounds);
        draw_bands(&mut frame, &self.bands, bounds);
        vec![frame.into_geometry()]
    }
}

fn draw_grid(frame: &mut Frame, bounds: Rectangle) {
    for db in [-15.0, -7.5, 0.0, 7.5, 15.0] {
        let y = gain_to_y(db, bounds);
        frame.stroke(
            &Path::line(Point::new(0.0, y), Point::new(bounds.width, y)),
            Stroke::default()
                .with_color(if db == 0.0 {
                    Color::from_rgba(0.8, 0.84, 0.9, 0.3)
                } else {
                    Color::from_rgba(0.7, 0.75, 0.82, 0.12)
                })
                .with_width(1.0),
        );
        frame.fill_text(Text {
            content: format!("{db:+.1}"),
            position: Point::new(4.0, y + 2.0),
            color: Color::from_rgba(0.75, 0.78, 0.84, 0.55),
            size: 9.0.into(),
            ..Default::default()
        });
    }

    for hz in [
        20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0,
    ] {
        let x = frequency_to_x(hz, bounds);
        frame.stroke(
            &Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
            Stroke::default()
                .with_color(Color::from_rgba(0.7, 0.75, 0.82, 0.10))
                .with_width(1.0),
        );
        frame.fill_text(Text {
            content: format_hz(hz),
            position: Point::new(x + 3.0, bounds.height - 14.0),
            color: Color::from_rgba(0.75, 0.78, 0.84, 0.55),
            size: 9.0.into(),
            ..Default::default()
        });
    }
}

fn draw_bands(frame: &mut Frame, bands: &[EqBand], bounds: Rectangle) {
    let samples = (bounds.width as usize).clamp(2, 600);
    for (index, band) in bands.iter().copied().enumerate() {
        let color = BAND_COLORS[index % BAND_COLORS.len()];
        let path = response_path(bounds, samples, |frequency| {
            band_response_db(band, frequency)
        });
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(Color { a: 0.4, ..color })
                .with_width(1.0),
        );
    }

    let response = response_path(bounds, samples, |frequency| {
        bands
            .iter()
            .map(|band| band_response_db(*band, frequency))
            .sum()
    });
    frame.stroke(
        &response,
        Stroke::default()
            .with_color(Color::from_rgb8(0xE6, 0xE8, 0xEE))
            .with_width(2.0),
    );

    for (index, band) in bands.iter().copied().enumerate() {
        let color = BAND_COLORS[index % BAND_COLORS.len()];
        let point = band_point(band, bounds);
        frame.fill(&Path::circle(point, 6.0), color);
        frame.fill_text(Text {
            content: band.index.to_string(),
            position: Point::new(point.x - 3.0, point.y - 5.0),
            color: Color::from_rgb8(0x14, 0x16, 0x1C),
            size: 9.0.into(),
            ..Default::default()
        });
    }
}

fn response_path(bounds: Rectangle, samples: usize, response: impl Fn(f32) -> f32) -> Path {
    Path::new(|builder| {
        for sample in 0..samples {
            let x = sample as f32 * bounds.width / (samples - 1) as f32;
            let frequency = logf_value(x / bounds.width, 20.0, 20_000.0);
            let y = gain_to_y(response(frequency), bounds);
            let point = Point::new(x, y);
            if sample == 0 {
                builder.move_to(point);
            } else {
                builder.line_to(point);
            }
        }
    })
}

fn band_point(band: EqBand, bounds: Rectangle) -> Point {
    Point::new(
        band.frequency.clamp(0.0, 1.0) * bounds.width,
        (1.0 - band.gain.clamp(0.0, 1.0)) * bounds.height,
    )
}

fn frequency_to_x(frequency: f32, bounds: Rectangle) -> f32 {
    (frequency.max(20.0).ln() - 20.0_f32.ln()) / (20_000.0_f32.ln() - 20.0_f32.ln()) * bounds.width
}

fn gain_to_y(gain: f32, bounds: Rectangle) -> f32 {
    (1.0 - ((gain.clamp(MIN_GAIN_DB, MAX_GAIN_DB) - MIN_GAIN_DB) / (MAX_GAIN_DB - MIN_GAIN_DB)))
        * bounds.height
}

fn quantize(value: f32, steps: u16) -> f32 {
    (value.clamp(0.0, 1.0) * f32::from(steps)).round() / f32::from(steps)
}

fn reset_values(band_count: usize, band: u8) -> (f32, f32, f32) {
    const FOUR_BAND_FREQUENCIES: [f32; 4] = [20.0, 200.0, 1000.0, 10_000.0];
    const SIX_BAND_FREQUENCIES: [f32; 6] = [20.0, 100.0, 500.0, 2000.0, 5000.0, 20_000.0];
    let index = usize::from(band.saturating_sub(1));
    let frequency = match band_count {
        4 => FOUR_BAND_FREQUENCIES[index.min(3)],
        6 => SIX_BAND_FREQUENCIES[index.min(5)],
        _ => 1000.0,
    };
    let frequency = (frequency / 20.0).ln() / (20_000.0_f32 / 20.0).ln();
    let q = (2.0_f32 / 10.0).ln() / (0.3_f32 / 10.0).ln();
    (quantize(frequency, 200), 0.5, quantize(q, 71))
}

fn is_double_click(
    previous: Option<(usize, Instant, Point)>,
    band: usize,
    now: Instant,
    point: Point,
) -> bool {
    previous.is_some_and(|(previous_band, timestamp, previous_point)| {
        previous_band == band
            && now.duration_since(timestamp) <= Duration::from_millis(400)
            && (point.x - previous_point.x).powi(2) + (point.y - previous_point.y).powi(2) <= 64.0
    })
}

fn scrolled_q(normalized: f32, step: f32) -> f32 {
    let current = logf_value(normalized, 10.0, 0.3);
    let next = (current * (1.0 + step * 0.15)).clamp(0.3, 10.0);
    let normalized = (next / 10.0).ln() / (0.3_f32 / 10.0).ln();
    quantize(normalized, 71)
}

fn band_response_db(band: EqBand, frequency: f32) -> f32 {
    let center = logf_value(band.frequency, 20.0, 20_000.0);
    let gain = logf_value(band.gain, MIN_GAIN_DB, MAX_GAIN_DB);
    let q = logf_value(band.q, 10.0, 0.3);
    match band.kind {
        0 => cut_response(frequency, center, q, 12.0, false),
        1 => shelf_response(frequency, center, gain, q, false),
        2 | 3 => peaking_response(frequency, center, gain, q),
        4 => shelf_response(frequency, center, gain, q, true),
        5 => cut_response(frequency, center, q, 12.0, true),
        6 => cut_response(frequency, center, q, 6.0, band.index >= 6),
        7..=9 => cut_response(frequency, center, q, 12.0, band.index >= 6),
        10 => cut_response(frequency, center, q, 18.0, band.index >= 6),
        11..=13 => cut_response(frequency, center, q, 24.0, band.index >= 6),
        _ => 0.0,
    }
}

fn peaking_response(frequency: f32, center: f32, gain: f32, q: f32) -> f32 {
    if gain.abs() < 0.001 {
        return 0.0;
    }
    let amplitude = 10.0_f64.powf(f64::from(gain) / 40.0);
    let omega = 2.0 * std::f64::consts::PI * f64::from(frequency) / 48_000.0;
    let center_omega = 2.0 * std::f64::consts::PI * f64::from(center) / 48_000.0;
    let alpha = center_omega.sin() / (2.0 * f64::from(q));
    let cosine = center_omega.cos();
    let b0 = 1.0 + alpha * amplitude;
    let b1 = -2.0 * cosine;
    let b2 = 1.0 - alpha * amplitude;
    let a0 = 1.0 + alpha / amplitude;
    let a1 = -2.0 * cosine;
    let a2 = 1.0 - alpha / amplitude;
    biquad_magnitude_db(omega, [b0, b1, b2], [a0, a1, a2])
}

fn biquad_magnitude_db(omega: f64, b: [f64; 3], a: [f64; 3]) -> f32 {
    let cosine = omega.cos();
    let cosine_twice = (2.0 * omega).cos();
    let numerator = b[0] * b[0]
        + b[1] * b[1]
        + b[2] * b[2]
        + 2.0 * (b[0] * b[1] + b[1] * b[2]) * cosine
        + 2.0 * b[0] * b[2] * cosine_twice;
    let denominator = a[0] * a[0]
        + a[1] * a[1]
        + a[2] * a[2]
        + 2.0 * (a[0] * a[1] + a[1] * a[2]) * cosine
        + 2.0 * a[0] * a[2] * cosine_twice;
    (10.0 * (numerator.max(1.0e-12) / denominator.max(1.0e-12)).log10()) as f32
}

fn shelf_response(frequency: f32, center: f32, gain: f32, q: f32, high: bool) -> f32 {
    let slope = q.sqrt().max(0.3);
    let ratio = if high {
        center / frequency.max(1.0)
    } else {
        frequency.max(1.0) / center
    };
    gain / (1.0 + ratio.powf(2.0 * slope))
}

fn cut_response(frequency: f32, center: f32, _q: f32, slope_db: f32, high: bool) -> f32 {
    let ratio = if high {
        frequency.max(1.0) / center
    } else {
        center / frequency.max(1.0)
    };
    -10.0 * (1.0 + ratio.powf(slope_db / 3.0)).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_nodes_cover_full_frequency_and_gain_ranges() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 240.0,
        };
        let low = band_point(
            EqBand {
                index: 1,
                frequency: 0.0,
                gain: 0.0,
                q: 0.0,
                kind: 2,
            },
            bounds,
        );
        let high = band_point(
            EqBand {
                index: 1,
                frequency: 1.0,
                gain: 1.0,
                q: 1.0,
                kind: 2,
            },
            bounds,
        );
        assert_eq!(low, Point::new(0.0, 240.0));
        assert_eq!(high, Point::new(800.0, 0.0));
    }

    #[test]
    fn wheel_changes_q_in_the_expected_direction() {
        let initial = scrolled_q(0.5, 0.0);
        let increased = scrolled_q(initial, 1.0);
        let decreased = scrolled_q(initial, -1.0);
        assert!(logf_value(increased, 10.0, 0.3) > logf_value(initial, 10.0, 0.3));
        assert!(logf_value(decreased, 10.0, 0.3) < logf_value(initial, 10.0, 0.3));
    }

    #[test]
    fn double_click_requires_same_band_and_close_timing() {
        let now = Instant::now();
        let point = Point::new(100.0, 80.0);
        let previous = Some((2, now, point));
        assert!(is_double_click(
            previous,
            2,
            now + Duration::from_millis(200),
            point,
        ));
        assert!(!is_double_click(
            previous,
            1,
            now + Duration::from_millis(200),
            point,
        ));
        assert!(!is_double_click(
            previous,
            2,
            now + Duration::from_millis(500),
            point,
        ));
    }

    #[test]
    fn reset_restores_neutral_gain_and_default_q() {
        let (frequency, gain, q) = reset_values(4, 3);
        assert!((logf_value(frequency, 20.0, 20_000.0) - 1000.0).abs() < 10.0);
        assert_eq!(gain, 0.5);
        assert!((logf_value(q, 10.0, 0.3) - 2.0).abs() < 0.05);
    }

    #[test]
    fn peaking_band_has_requested_gain_at_center_frequency() {
        let band = EqBand {
            index: 1,
            frequency: 0.5,
            gain: 0.9,
            q: 0.5,
            kind: 2,
        };
        let center = logf_value(band.frequency, 20.0, 20_000.0);
        let response = band_response_db(band, center);
        assert!((response - 12.0).abs() < 0.01, "response was {response}");
    }
}
