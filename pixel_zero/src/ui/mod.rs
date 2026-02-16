use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::RangeInclusive;
use std::sync::Arc;

use glam::{IVec2, UVec2, ivec2, uvec2};
use num_traits::{Num, NumCast, ToPrimitive};
use parking_lot::Mutex;
use strum::IntoEnumIterator as _;

use crate::graphics::frame::DrawCommand;
use crate::graphics::{Color, Font, Frame};
use crate::input::{Button, Input};
use crate::{HEIGHT, WIDTH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    pub fn new(source: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug, Clone)]
pub struct Ui(Arc<Mutex<UiInner>>);

impl Ui {
    #[must_use]
    pub fn new(font: Font) -> Self {
        Self(Arc::new(Mutex::new(UiInner::new(font))))
    }

    pub fn update_input(&self, input: &Input) {
        let mut inner = self.0.lock();
        inner.input = UiInput::from_input(input);
    }

    pub fn begin_frame(&self) {
        let mut inner = self.0.lock();
        let mut focus_index = inner.focus_index;
        if inner.last_widget_count > 0 {
            if inner.input.just_pressed(Button::Up) {
                focus_index = focus_index.saturating_sub(1);
            }
            if inner.input.just_pressed(Button::Down) {
                focus_index = (focus_index + 1).min(inner.last_widget_count - 1);
            }
        }

        inner.frame_focus_index = focus_index;
        inner.widget_index = 0;
        inner.cursor = ivec2(inner.style.padding, inner.style.padding);
        inner.layout_width = WIDTH.saturating_sub((inner.style.padding * 2).max(0).cast_unsigned());
        inner.draw_commands.clear();
    }

    pub fn clear(&self) {
        let mut inner = self.0.lock();
        inner.draw_commands.clear();
        inner.widget_index = 0;
        inner.cursor = ivec2(inner.style.padding, inner.style.padding);
    }

    pub fn label(&self, text: &str) {
        let mut inner = self.0.lock();
        inner.label(text);
    }

    pub fn button(&self, text: &str) -> bool {
        let mut inner = self.0.lock();
        inner.button(text)
    }

    pub fn checkbox(&self, text: &str, value: &mut bool) -> bool {
        let mut inner = self.0.lock();
        inner.checkbox(text, value)
    }

    pub fn slider<T>(&self, text: &str, value: &mut T, range: RangeInclusive<T>) -> bool
    where
        T: Num + Copy + PartialOrd + ToPrimitive + NumCast,
    {
        let mut inner = self.0.lock();
        inner.slider(text, value, range)
    }

    pub fn render(&self, frame: &mut Frame) {
        let mut inner = self.0.lock();
        let widget_count = inner.widget_index;
        if widget_count > 0 {
            inner.focus_index = inner.frame_focus_index.min(widget_count - 1);
        } else {
            inner.focus_index = 0;
        }
        inner.last_widget_count = widget_count;
        frame.add_commands(&inner.draw_commands);
    }

    #[must_use]
    pub fn font(&self) -> Font {
        self.0.lock().font.clone()
    }
}

#[derive(Debug)]
pub struct UiInner {
    font: Font,
    input: UiInput,
    focus_index: usize,
    last_widget_count: usize,
    style: UiStyle,
    draw_commands: Vec<DrawCommand>,
    cursor: IVec2,
    layout_width: u32,
    frame_focus_index: usize,
    widget_index: usize,
}

impl UiInner {
    fn new(font: Font) -> Self {
        Self {
            font,
            input: UiInput::default(),
            focus_index: 0,
            last_widget_count: 0,
            style: UiStyle::default(),
            draw_commands: Vec::new(),
            cursor: ivec2(0, 0),
            layout_width: WIDTH,
            frame_focus_index: 0,
            widget_index: 0,
        }
    }
}

impl UiInner {
    fn label(&mut self, text: &str) {
        let position = self.cursor;
        self.draw_commands.push(DrawCommand::Text {
            font: self.font.clone(),
            text: text.to_owned(),
            position,
        });
        let height = self.font.glyph_size().y.cast_signed();
        self.advance(height + self.style.spacing);
    }

    fn button(&mut self, text: &str) -> bool {
        let is_focused = self.widget_index == self.frame_focus_index;
        let button_size = self.button_size();
        let position = self.cursor;

        let fill = if is_focused {
            self.style.widget_bg_focused
        } else {
            self.style.widget_bg
        };

        self.draw_commands.push(DrawCommand::Rect {
            position,
            size: button_size,
            color: fill,
            filled: true,
        });
        self.draw_commands.push(DrawCommand::Rect {
            position,
            size: button_size,
            color: self.style.widget_border,
            filled: false,
        });

        let text_size = self.font.text_size(text);
        let text_x =
            position.x + ((button_size.x.cast_signed() - text_size.x.cast_signed()) / 2).max(0);
        let text_y =
            position.y + ((button_size.y.cast_signed() - text_size.y.cast_signed()) / 2).max(0);
        self.draw_commands.push(DrawCommand::Text {
            font: self.font.clone(),
            text: text.to_owned(),
            position: ivec2(text_x, text_y),
        });

        if is_focused {
            self.draw_focus_outline(position, button_size);
        }

        self.advance(button_size.y.cast_signed() + self.style.spacing);
        self.widget_index += 1;

        is_focused && self.input.just_pressed(Button::A)
    }

    fn checkbox(&mut self, text: &str, value: &mut bool) -> bool {
        let is_focused = self.widget_index == self.frame_focus_index;
        let size = self.style.checkbox_size;
        let row_height = size.max(self.font.glyph_size().y).cast_signed();
        let position = self.cursor;

        let box_position = position;
        let box_size = uvec2(size, size);

        self.draw_commands.push(DrawCommand::Rect {
            position: box_position,
            size: box_size,
            color: self.style.widget_border,
            filled: false,
        });

        if *value {
            self.draw_commands.push(DrawCommand::Rect {
                position: box_position + 2,
                size: box_size - 4,
                color: self.style.checkbox_fill,
                filled: true,
            });
        }

        let text_size = self.font.text_size(text);
        let text_x = position.x + size.cast_signed() + self.style.spacing;
        let text_y = position.y + ((row_height - text_size.y.cast_signed()) / 2).max(0);
        self.draw_commands.push(DrawCommand::Text {
            font: self.font.clone(),
            text: text.to_owned(),
            position: ivec2(text_x, text_y),
        });

        if is_focused {
            let width = (size.cast_signed() + self.style.spacing + text_size.x.cast_signed())
                .min(self.layout_width.cast_signed())
                .max(size.cast_signed());
            let outline_offset = ivec2(-1, -1);
            let outline_size = uvec2(
                (width + 2).cast_unsigned(),
                (row_height + 1).cast_unsigned(),
            );
            self.draw_focus_outline(position + outline_offset, outline_size);
        }

        let mut changed = false;
        if is_focused && self.input.just_pressed(Button::A) {
            *value = !*value;
            changed = true;
        }

        self.advance(row_height + self.style.spacing);
        self.widget_index += 1;
        changed
    }

    fn slider<T>(&mut self, text: &str, value: &mut T, range: RangeInclusive<T>) -> bool
    where
        T: Num + Copy + PartialOrd + ToPrimitive + NumCast,
    {
        self.label(text);

        let is_focused = self.widget_index == self.frame_focus_index;
        let slider_height = self.style.slider_height.cast_signed();
        let position = self.cursor;
        let size = uvec2(self.layout_width, self.style.slider_height);

        let track_height = self.style.slider_track_height.cast_signed();
        let track_y = position.y + ((slider_height - track_height) / 2).max(0);
        let track_position = ivec2(position.x, track_y);
        let track_size = uvec2(size.x, track_height.cast_unsigned());

        self.draw_commands.push(DrawCommand::Rect {
            position: track_position,
            size: track_size,
            color: self.style.slider_track,
            filled: true,
        });

        let (min, max) = normalized_range(range);
        let Some(min_f) = min.to_f32() else {
            return false;
        };
        let Some(max_f) = max.to_f32() else {
            return false;
        };
        let Some(value_f) = value.to_f32() else {
            return false;
        };

        let range_size = (max_f - min_f).max(0.0001);
        let normalized = ((value_f - min_f) / range_size).clamp(0.0, 1.0);
        let knob_x = position.x + (normalized * (size.x.saturating_sub(1)) as f32) as i32;
        let knob_half = (self.style.slider_knob_width / 2).cast_signed();
        let knob_position = ivec2(
            knob_x - knob_half,
            position.y + ((slider_height - self.style.slider_knob_height.cast_signed()) / 2).max(0),
        );
        let knob_size = uvec2(self.style.slider_knob_width, self.style.slider_knob_height);

        let fill_width = (normalized * size.x as f32) as u32;
        if fill_width > 0 {
            self.draw_commands.push(DrawCommand::Rect {
                position: track_position,
                size: uvec2(fill_width, track_height.cast_unsigned()),
                color: self.style.slider_fill,
                filled: true,
            });
        }

        self.draw_commands.push(DrawCommand::Rect {
            position: knob_position,
            size: knob_size,
            color: self.style.slider_knob,
            filled: true,
        });

        if is_focused {
            self.draw_focus_outline(position, size);
        }

        let mut changed = false;
        if is_focused {
            let step = (range_size / 100.0).max(0.01);
            let mut next_value = value_f;
            if self.input.is_pressed(Button::Left) {
                next_value -= step;
            }
            if self.input.is_pressed(Button::Right) {
                next_value += step;
            }
            next_value = next_value.clamp(min_f, max_f);
            if (next_value - value_f).abs() > f32::EPSILON
                && let Some(next_value) = NumCast::from(next_value)
            {
                *value = next_value;
                changed = true;
            }
        }

        self.advance(slider_height + self.style.spacing);
        self.widget_index += 1;
        changed
    }

    fn button_size(&self) -> UVec2 {
        let height = self.style.button_height.max(self.font.glyph_size().y + 6);
        uvec2(self.layout_width, height)
    }

    fn advance(&mut self, delta: i32) {
        let max_y = HEIGHT.cast_signed() - self.style.padding;
        self.cursor.y = (self.cursor.y + delta).min(max_y);
    }

    fn draw_focus_outline(&mut self, position: IVec2, size: UVec2) {
        self.draw_commands.push(DrawCommand::Rect {
            position,
            size,
            color: self.style.focus_outline,
            filled: false,
        });
    }
}

#[derive(Debug, Clone, Copy)]
struct UiInput {
    pressed: [bool; Button::BUTTON_COUNT],
    just_pressed: [bool; Button::BUTTON_COUNT],
}

impl UiInput {
    fn from_input(input: &Input) -> Self {
        let pressed = *input.state();
        let mut just_pressed = [false; Button::BUTTON_COUNT];
        for button in Button::iter() {
            just_pressed[button.index()] = input.just_pressed(button);
        }

        Self {
            pressed,
            just_pressed,
        }
    }

    fn is_pressed(&self, button: Button) -> bool {
        self.pressed[button.index()]
    }

    fn just_pressed(&self, button: Button) -> bool {
        self.just_pressed[button.index()]
    }
}

impl Default for UiInput {
    fn default() -> Self {
        Self {
            pressed: [false; Button::BUTTON_COUNT],
            just_pressed: [false; Button::BUTTON_COUNT],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct UiStyle {
    padding: i32,
    spacing: i32,
    button_height: u32,
    checkbox_size: u32,
    slider_height: u32,
    slider_track_height: u32,
    slider_knob_width: u32,
    slider_knob_height: u32,
    widget_bg: Color,
    widget_bg_focused: Color,
    widget_border: Color,
    checkbox_fill: Color,
    slider_track: Color,
    slider_fill: Color,
    slider_knob: Color,
    focus_outline: Color,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            padding: 8,
            spacing: 6,
            button_height: 16,
            checkbox_size: 12,
            slider_height: 12,
            slider_track_height: 2,
            slider_knob_width: 6,
            slider_knob_height: 12,
            widget_bg: Color::rgb(50, 50, 50),
            widget_bg_focused: Color::rgb(70, 70, 70),
            widget_border: Color::rgb(90, 90, 90),
            checkbox_fill: Color::rgb(220, 220, 220),
            slider_track: Color::rgb(60, 60, 60),
            slider_fill: Color::rgb(120, 120, 120),
            slider_knob: Color::rgb(220, 220, 220),
            focus_outline: Color::YELLOW,
        }
    }
}

fn normalized_range<T: PartialOrd + Copy>(range: RangeInclusive<T>) -> (T, T) {
    let start = *range.start();
    let end = *range.end();
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}
