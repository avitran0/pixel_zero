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
        inner.reset_layout();
        inner.draw_commands.clear();
    }

    pub fn clear(&self) {
        let mut inner = self.0.lock();
        inner.draw_commands.clear();
        inner.widget_index = 0;
        inner.reset_layout();
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

    pub fn radio(&self, text: &str, selected: &mut usize, index: usize) -> bool {
        let mut inner = self.0.lock();
        inner.radio(text, selected, index)
    }

    pub fn slider<T>(&self, text: &str, value: &mut T, range: RangeInclusive<T>) -> bool
    where
        T: Num + Copy + PartialOrd + ToPrimitive + NumCast,
    {
        let mut inner = self.0.lock();
        inner.slider(text, value, range)
    }

    pub fn progress_bar<T>(&self, value: T, range: RangeInclusive<T>)
    where
        T: Num + Copy + PartialOrd + ToPrimitive,
    {
        let mut inner = self.0.lock();
        inner.progress_bar(value, range);
    }

    pub fn separator(&self) {
        let mut inner = self.0.lock();
        inner.separator();
    }

    pub fn spacer(&self, height: u32) {
        let mut inner = self.0.lock();
        inner.spacer(height);
    }

    pub fn begin_columns(&self, count: u32) {
        let mut inner = self.0.lock();
        inner.begin_columns(count);
    }

    pub fn next_column(&self) {
        let mut inner = self.0.lock();
        inner.next_column();
    }

    pub fn end_columns(&self) {
        let mut inner = self.0.lock();
        inner.end_columns();
    }

    pub fn set_layout_width(&self, width: u32) {
        let mut inner = self.0.lock();
        inner.style.layout_width = Some(width.max(1));
        inner.layout_width = inner.clamp_layout_width(width);
    }

    pub fn clear_layout_width(&self) {
        let mut inner = self.0.lock();
        inner.style.layout_width = None;
        inner.layout_width = inner.max_layout_width();
    }

    pub fn set_padding(&self, padding: i32) {
        let mut inner = self.0.lock();
        inner.style.padding = padding.max(0);
    }

    pub fn set_spacing(&self, spacing: i32) {
        let mut inner = self.0.lock();
        inner.style.spacing = spacing.max(0);
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
    columns: Option<ColumnsState>,
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
            layout_width: WIDTH / 3,
            frame_focus_index: 0,
            widget_index: 0,
            columns: None,
        }
    }
}

impl UiInner {
    fn label(&mut self, text: &str) {
        let text_size = self.font.text_size(text);
        let position = self.place_widget(text_size);
        self.draw_text(text, position);
    }

    fn button(&mut self, text: &str) -> bool {
        let is_focused = self.widget_index == self.frame_focus_index;
        let button_size = self.button_size();
        let position = self.place_widget(button_size);

        let fill = if is_focused {
            self.style.widget_bg_focused
        } else {
            self.style.widget_bg
        };

        self.draw_rect(position, button_size, fill, true);
        self.draw_rect(position, button_size, self.style.widget_border, false);

        let text_size = self.font.text_size(text);
        let text_x =
            position.x + ((button_size.x.cast_signed() - text_size.x.cast_signed()) / 2).max(0);
        let text_y =
            position.y + ((button_size.y.cast_signed() - text_size.y.cast_signed()) / 2).max(0);
        self.draw_text(text, ivec2(text_x, text_y));

        if is_focused {
            self.draw_focus_outline(position, button_size);
        }

        self.widget_index += 1;

        is_focused && self.input.just_pressed(Button::A)
    }

    fn checkbox(&mut self, text: &str, value: &mut bool) -> bool {
        let is_focused = self.widget_index == self.frame_focus_index;
        let size = self.style.checkbox_size;
        let row_height = size.max(self.font.glyph_size().y).cast_signed();
        let text_size = self.font.text_size(text);
        let width = (size.cast_signed() + self.style.spacing + text_size.x.cast_signed())
            .max(size.cast_signed())
            .cast_unsigned();
        let position = self.place_widget(uvec2(width, row_height.cast_unsigned()));

        let box_position = position;
        let box_size = uvec2(size, size);

        self.draw_rect(box_position, box_size, self.style.widget_border, false);

        if *value {
            self.draw_rect(box_position + 2, box_size - 4, self.style.checkbox_fill, true);
        }

        let text_x = position.x + size.cast_signed() + self.style.spacing;
        let text_y = position.y + ((row_height - text_size.y.cast_signed()) / 2).max(0);
        self.draw_text(text, ivec2(text_x, text_y));

        if is_focused {
            let outline_offset = ivec2(-1, -1);
            let outline_size = uvec2(
                (width.cast_signed() + 2).cast_unsigned(),
                (row_height + 1).cast_unsigned(),
            );
            self.draw_focus_outline(position + outline_offset, outline_size);
        }

        let mut changed = false;
        if is_focused && self.input.just_pressed(Button::A) {
            *value = !*value;
            changed = true;
        }

        self.widget_index += 1;
        changed
    }

    fn radio(&mut self, text: &str, selected: &mut usize, index: usize) -> bool {
        let is_focused = self.widget_index == self.frame_focus_index;
        let size = self.style.radio_size;
        let row_height = size.max(self.font.glyph_size().y).cast_signed();
        let text_size = self.font.text_size(text);
        let width = (size.cast_signed() + self.style.spacing + text_size.x.cast_signed())
            .max(size.cast_signed())
            .cast_unsigned();
        let position = self.place_widget(uvec2(width, row_height.cast_unsigned()));

        let box_position = position;
        let box_size = uvec2(size, size);
        self.draw_rect(box_position, box_size, self.style.widget_border, false);

        if *selected == index {
            let inset = 3u32.min(size.saturating_sub(1));
            let inset_i = inset.cast_signed();
            self.draw_rect(
                box_position + ivec2(inset_i, inset_i),
                box_size.saturating_sub(uvec2(inset * 2, inset * 2)),
                self.style.radio_fill,
                true,
            );
        }

        let text_x = position.x + size.cast_signed() + self.style.spacing;
        let text_y = position.y + ((row_height - text_size.y.cast_signed()) / 2).max(0);
        self.draw_text(text, ivec2(text_x, text_y));

        if is_focused {
            let outline_offset = ivec2(-1, -1);
            let outline_size = uvec2(
                (width.cast_signed() + 2).cast_unsigned(),
                (row_height + 1).cast_unsigned(),
            );
            self.draw_focus_outline(position + outline_offset, outline_size);
        }

        let mut changed = false;
        if is_focused && self.input.just_pressed(Button::A) && *selected != index {
            *selected = index;
            changed = true;
        }

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
        let size = uvec2(self.layout_width, self.style.slider_height);
        let position = self.place_widget(size);

        let track_height = self.style.slider_track_height.cast_signed();
        let track_y = position.y + ((slider_height - track_height) / 2).max(0);
        let track_position = ivec2(position.x, track_y);
        let track_size = uvec2(size.x, track_height.cast_unsigned());

        self.draw_rect(track_position, track_size, self.style.slider_track, true);

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
            self.draw_rect(
                track_position,
                uvec2(fill_width, track_height.cast_unsigned()),
                self.style.slider_fill,
                true,
            );
        }

        self.draw_rect(knob_position, knob_size, self.style.slider_knob, true);

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

        self.widget_index += 1;
        changed
    }

    fn progress_bar<T>(&mut self, value: T, range: RangeInclusive<T>)
    where
        T: Num + Copy + PartialOrd + ToPrimitive,
    {
        let (min, max) = normalized_range(range);
        let Some(min_f) = min.to_f32() else {
            return;
        };
        let Some(max_f) = max.to_f32() else {
            return;
        };
        let Some(value_f) = value.to_f32() else {
            return;
        };
        let range_size = (max_f - min_f).max(0.0001);
        let normalized = ((value_f - min_f) / range_size).clamp(0.0, 1.0);
        let size = uvec2(self.layout_width, self.style.progress_height);
        let position = self.place_widget(size);
        let fill_width = (normalized * size.x as f32) as u32;

        self.draw_rect(position, size, self.style.progress_track, true);

        if fill_width > 0 {
            self.draw_rect(position, uvec2(fill_width, size.y), self.style.progress_fill, true);
        }

        self.draw_rect(position, size, self.style.widget_border, false);
    }

    fn separator(&mut self) {
        let size = uvec2(self.layout_width, self.style.separator_thickness.max(1));
        let position = self.place_widget(size);
        self.draw_rect(position, size, self.style.separator, true);
    }

    fn spacer(&mut self, height: u32) {
        let size = uvec2(self.layout_width, height.max(1));
        self.place_widget(size);
    }

    fn button_size(&self) -> UVec2 {
        let height = self.style.button_height.max(self.font.glyph_size().y + 6);
        uvec2(self.layout_width, height)
    }

    fn place_widget(&mut self, size: UVec2) -> IVec2 {
        let position = self.cursor;
        self.cursor.y = self.clamp_y(self.cursor.y + size.y.cast_signed() + self.style.spacing);

        if let Some(columns) = &mut self.columns {
            columns.max_y = columns.max_y.max(self.cursor.y);
        }

        position
    }

    fn clamp_y(&self, y: i32) -> i32 {
        let max_y = HEIGHT.cast_signed() - self.style.padding;
        y.min(max_y)
    }

    fn begin_columns(&mut self, count: u32) {
        if count < 2 || self.columns.is_some() {
            return;
        }
        let spacing = self.style.spacing.max(0).cast_unsigned();
        let total_spacing = spacing.saturating_mul(count.saturating_sub(1));
        let available = self.layout_width.saturating_sub(total_spacing).max(1);
        let column_width = (available / count).max(1);
        self.columns = Some(ColumnsState {
            count,
            column_width,
            column_index: 0,
            origin: self.cursor,
            max_y: self.cursor.y,
            previous_layout_width: self.layout_width,
        });
        self.layout_width = column_width;
    }

    fn next_column(&mut self) {
        let Some(columns) = &mut self.columns else {
            return;
        };

        columns.max_y = columns.max_y.max(self.cursor.y);
        columns.column_index = (columns.column_index + 1).min(columns.count - 1);
        let spacing = self.style.spacing.max(0);
        let offset =
            columns.column_index.cast_signed() * (columns.column_width.cast_signed() + spacing);
        self.cursor = ivec2(columns.origin.x + offset, columns.origin.y);
    }

    fn end_columns(&mut self) {
        let Some(columns) = self.columns.take() else {
            return;
        };

        let max_y = columns.max_y.max(self.cursor.y);
        let next_y = self.clamp_y(max_y + self.style.spacing);
        self.layout_width = columns.previous_layout_width;
        self.cursor = ivec2(columns.origin.x, next_y);
    }

    fn reset_layout(&mut self) {
        let padding = self.style.padding;
        self.cursor = ivec2(padding, padding);
        self.layout_width = self
            .style
            .layout_width
            .map(|width| self.clamp_layout_width(width))
            .unwrap_or_else(|| self.max_layout_width());
        self.columns = None;
    }

    fn draw_focus_outline(&mut self, position: IVec2, size: UVec2) {
        self.draw_rect(position, size, self.style.focus_outline, false);
    }

    fn draw_rect(&mut self, position: IVec2, size: UVec2, color: Color, filled: bool) {
        self.draw_commands.push(DrawCommand::Rect {
            position,
            size,
            color,
            filled,
        });
    }

    fn draw_text(&mut self, text: &str, position: IVec2) {
        self.draw_commands.push(DrawCommand::Text {
            font: self.font.clone(),
            text: text.to_owned(),
            position,
        });
    }

    fn max_layout_width(&self) -> u32 {
        let padding = self.style.padding;
        WIDTH.saturating_sub((padding * 2).max(0).cast_unsigned())
    }

    fn clamp_layout_width(&self, width: u32) -> u32 {
        width.clamp(1, self.max_layout_width().max(1))
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
    layout_width: Option<u32>,
    button_height: u32,
    checkbox_size: u32,
    radio_size: u32,
    slider_height: u32,
    slider_track_height: u32,
    slider_knob_width: u32,
    slider_knob_height: u32,
    progress_height: u32,
    separator_thickness: u32,
    widget_bg: Color,
    widget_bg_focused: Color,
    widget_border: Color,
    checkbox_fill: Color,
    radio_fill: Color,
    slider_track: Color,
    slider_fill: Color,
    slider_knob: Color,
    progress_track: Color,
    progress_fill: Color,
    separator: Color,
    focus_outline: Color,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            padding: 4,
            spacing: 2,
            layout_width: None,
            button_height: 12,
            checkbox_size: 12,
            radio_size: 12,
            slider_height: 12,
            slider_track_height: 2,
            slider_knob_width: 6,
            slider_knob_height: 12,
            progress_height: 6,
            separator_thickness: 1,
            widget_bg: Color::rgb(50, 50, 50),
            widget_bg_focused: Color::rgb(70, 70, 70),
            widget_border: Color::rgb(90, 90, 90),
            checkbox_fill: Color::rgb(220, 220, 220),
            radio_fill: Color::rgb(220, 220, 220),
            slider_track: Color::rgb(60, 60, 60),
            slider_fill: Color::rgb(120, 120, 120),
            slider_knob: Color::rgb(220, 220, 220),
            progress_track: Color::rgb(60, 60, 60),
            progress_fill: Color::rgb(120, 160, 220),
            separator: Color::rgb(80, 80, 80),
            focus_outline: Color::YELLOW,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ColumnsState {
    count: u32,
    column_width: u32,
    column_index: u32,
    origin: IVec2,
    max_y: i32,
    previous_layout_width: u32,
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
