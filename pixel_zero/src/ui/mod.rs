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

    pub fn slider<T>(&self, text: &str, value: &mut T, range: RangeInclusive<T>, speed: T) -> bool
    where
        T: Num + Copy + PartialOrd + ToPrimitive + NumCast + std::fmt::Display,
    {
        let mut inner = self.0.lock();
        inner.number(text, value, range, speed)
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

        self.draw_widget_background(position, button_size, is_focused);
        self.draw_centered_text(text, position, button_size);

        if is_focused {
            self.draw_focus_outline(position, button_size);
        }

        self.widget_index += 1;

        is_focused && self.input.just_pressed(Button::A)
    }

    fn checkbox(&mut self, text: &str, value: &mut bool) -> bool {
        let is_focused = self.widget_index == self.frame_focus_index;
        let layout = self.labeled_box_layout(text, self.style.checkbox_size);

        self.draw_rect(
            layout.box_position,
            layout.box_size,
            self.style.widget_border,
            false,
        );

        if *value {
            self.draw_rect(
                layout.box_position + 2,
                layout.box_size - 4,
                self.style.checkbox_fill,
                true,
            );
        }

        self.draw_text(text, layout.text_position);

        if is_focused {
            let outline_offset = ivec2(-1, -1);
            let outline_size = uvec2(
                (layout.width.cast_signed() + 2).cast_unsigned(),
                (layout.row_height + 1).cast_unsigned(),
            );
            self.draw_focus_outline(layout.position + outline_offset, outline_size);
        }

        let mut changed = false;
        if is_focused && self.input.just_pressed(Button::A) {
            *value = !*value;
            changed = true;
        }

        self.widget_index += 1;
        changed
    }

    fn number<T>(&mut self, text: &str, value: &mut T, range: RangeInclusive<T>, speed: T) -> bool
    where
        T: Num + Copy + PartialOrd + ToPrimitive + NumCast + std::fmt::Display,
    {
        self.label(text);

        let is_focused = self.widget_index == self.frame_focus_index;
        let size = uvec2(self.layout_width, self.widget_height());
        let position = self.place_widget(size);

        self.draw_widget_background(position, size, is_focused);

        let value_text = format!("< {} >", value);
        self.draw_centered_text(&value_text, position, size);

        if is_focused {
            self.draw_focus_outline(position, size);
        }

        let mut changed = false;
        if is_focused {
            changed = apply_step_input(&self.input, value, range, speed);
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
            self.draw_rect(
                position,
                uvec2(fill_width, size.y),
                self.style.progress_fill,
                true,
            );
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
        uvec2(self.layout_width, self.widget_height())
    }

    fn widget_height(&self) -> u32 {
        self.style.button_height.max(self.font.glyph_size().y + 6)
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

    fn draw_widget_background(&mut self, position: IVec2, size: UVec2, focused: bool) {
        let fill = if focused {
            self.style.widget_bg_focused
        } else {
            self.style.widget_bg
        };
        self.draw_rect(position, size, fill, true);
        self.draw_rect(position, size, self.style.widget_border, false);
    }

    fn draw_centered_text(&mut self, text: &str, position: IVec2, size: UVec2) {
        let text_size = self.font.text_size(text);
        let text_x = position.x + ((size.x.cast_signed() - text_size.x.cast_signed()) / 2).max(0);
        let text_y = position.y + ((size.y.cast_signed() - text_size.y.cast_signed()) / 2).max(0);
        self.draw_text(text, ivec2(text_x, text_y));
    }

    fn labeled_box_layout(&mut self, text: &str, box_size: u32) -> LabeledBoxLayout {
        let row_height = box_size.max(self.font.glyph_size().y).cast_signed();
        let text_size = self.font.text_size(text);
        let width = (box_size.cast_signed() + self.style.spacing + text_size.x.cast_signed())
            .max(box_size.cast_signed())
            .cast_unsigned();
        let position = self.place_widget(uvec2(width, row_height.cast_unsigned()));
        let text_x = position.x + box_size.cast_signed() + self.style.spacing;
        let text_y = position.y + ((row_height - text_size.y.cast_signed()) / 2).max(0);

        LabeledBoxLayout {
            position,
            box_position: position,
            box_size: uvec2(box_size, box_size),
            text_position: ivec2(text_x, text_y),
            width,
            row_height,
        }
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
struct LabeledBoxLayout {
    position: IVec2,
    box_position: IVec2,
    box_size: UVec2,
    text_position: IVec2,
    width: u32,
    row_height: i32,
}

#[derive(Debug, Clone, Copy)]
struct UiInput {
    just_pressed: [bool; Button::BUTTON_COUNT],
}

impl UiInput {
    fn from_input(input: &Input) -> Self {
        let mut just_pressed = [false; Button::BUTTON_COUNT];
        for button in Button::iter() {
            just_pressed[button.index()] = input.just_pressed(button);
        }

        Self { just_pressed }
    }

    fn just_pressed(&self, button: Button) -> bool {
        self.just_pressed[button.index()]
    }
}

impl Default for UiInput {
    fn default() -> Self {
        Self {
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
    progress_height: u32,
    separator_thickness: u32,
    widget_bg: Color,
    widget_bg_focused: Color,
    widget_border: Color,
    checkbox_fill: Color,
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
            progress_height: 6,
            separator_thickness: 1,
            widget_bg: Color::rgb(50, 50, 50),
            widget_bg_focused: Color::rgb(70, 70, 70),
            widget_border: Color::rgb(90, 90, 90),
            checkbox_fill: Color::rgb(220, 220, 220),
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

fn apply_step_input<T>(input: &UiInput, value: &mut T, range: RangeInclusive<T>, speed: T) -> bool
where
    T: Num + Copy + PartialOrd + ToPrimitive + NumCast,
{
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
    let Some(speed_f) = speed.to_f32() else {
        return false;
    };

    let step = speed_f.abs();
    if step == 0.0 {
        return false;
    }

    let mut next_value = value_f;
    if input.just_pressed(Button::Left) {
        next_value -= step;
    }
    if input.just_pressed(Button::Right) {
        next_value += step;
    }
    next_value = next_value.clamp(min_f, max_f);

    if (next_value - value_f).abs() > f32::EPSILON
        && let Some(next_value) = NumCast::from(next_value)
    {
        *value = next_value;
        return true;
    }

    false
}
