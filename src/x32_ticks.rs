use iced::widget::{
    canvas,
    canvas::{Geometry, Path, Text},
};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme, mouse};
use std::{
    cell::Cell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

const SCALE_WIDTH: f32 = 22.0;
const SCALE_GAP: f32 = 3.0;
const OUTER_PAD_Y: f32 = 7.0;

#[derive(Default)]
struct State {
    cache: canvas::Cache,
    last_hash: Cell<u64>,
}

#[derive(Clone)]
struct TicksCanvas {
    fader_height: f32,
}

impl TicksCanvas {
    fn static_hash(&self, bounds: Rectangle) -> u64 {
        let mut hasher = DefaultHasher::new();
        bounds.width.to_bits().hash(&mut hasher);
        bounds.height.to_bits().hash(&mut hasher);
        self.fader_height.to_bits().hash(&mut hasher);
        hasher.finish()
    }

    fn tick_layout(&self) -> Vec<(f32, String)> {
        x32_tick_values()
            .into_iter()
            .map(|db| {
                let normalized = x32_db_to_normalized(db);
                let y = self.fader_height * (1.0 - normalized);
                let y = y.clamp(0.0, self.fader_height - 1.0);
                let label_y = (y - 4.0).clamp(0.0, (self.fader_height - 10.0).max(0.0));
                (label_y, format_tick_label(db))
            })
            .collect()
    }
}

impl<Message> canvas::Program<Message> for TicksCanvas {
    type State = State;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return vec![];
        }

        let static_hash = self.static_hash(bounds);
        if state.last_hash.get() != static_hash {
            state.cache.clear();
            state.last_hash.set(static_hash);
        }

        let static_geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let effective_height = (bounds.height - (OUTER_PAD_Y * 2.0)).max(1.0);
            let layout = Self {
                fader_height: effective_height,
            }
            .tick_layout();

            let tick_x = SCALE_GAP;
            for (label_y, label) in layout {
                frame.fill(
                    &Path::rectangle(
                        Point::new(tick_x, OUTER_PAD_Y + label_y + 4.0),
                        iced::Size::new(4.0, 1.0),
                    ),
                    Color::from_rgba(0.62, 0.67, 0.77, 0.78),
                );
                frame.fill_text(Text {
                    content: label,
                    position: Point::new(tick_x + 6.0, OUTER_PAD_Y + label_y),
                    color: Color::from_rgba(0.9, 0.92, 0.96, 0.9),
                    size: 8.0.into(),
                    ..Default::default()
                });
            }
        });

        vec![static_geometry]
    }
}

pub fn x32_ticks<'a, Message>(fader_height: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    canvas(TicksCanvas { fader_height })
        .width(Length::Fixed(SCALE_GAP + SCALE_WIDTH))
        .height(Length::Fill)
        .into()
}

fn x32_tick_values() -> Vec<f32> {
    vec![-90.0, -60.0, -40.0, -20.0, -10.0, -5.0, 0.0, 5.0, 10.0]
}

fn format_tick_label(value: f32) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        format!("{value:+.0}")
    }
}

fn x32_db_to_normalized(db: f32) -> f32 {
    let db = db.clamp(-90.0, 10.0);
    if db >= -10.0 {
        (db + 30.0) / 40.0
    } else if db >= -30.0 {
        (db + 50.0) / 80.0
    } else if db >= -60.0 {
        (db + 70.0) / 160.0
    } else {
        (db + 90.0) / 480.0
    }
}
