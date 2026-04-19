// * =========== MODULE: RECT =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

/// Draw a rectangle with optional fill, border, and corner radius.
/// Fill color comes from `bg_color` (the component's `BasicParams.bg_color`).
/// Corner radius is stored but macroquad's built-in primitives don't have rounded-rect,
/// so we fall back to a plain rectangle for now.
pub fn draw(
    layout: ResolvedLayout,
    bg_color: Color,
    border_color: Color,
    border_width: f32,
    _corner_radius: f32,
) {
    if bg_color.a > 0.0 {
        draw_rectangle(layout.x, layout.y, layout.w, layout.h, bg_color);
    }
    if border_color.a > 0.0 && border_width > 0.0 {
        draw_rectangle_lines(layout.x, layout.y, layout.w, layout.h, border_width, border_color);
    }
}

pub struct UIRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub bg_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    /// Corner radius (stored for future use — no-op with macroquad's built-in primitives).
    pub corner_radius: f32,
    pub visible: bool,
}

impl UIRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x, y, w, h,
            bg_color: Color::new(0.0, 0.0, 0.0, 0.0),
            border_color: Color::new(0.0, 0.0, 0.0, 0.0),
            border_width: 0.0,
            corner_radius: 0.0,
            visible: true,
        }
    }

    /// Create a filled rect with no border.
    pub fn filled(x: f32, y: f32, w: f32, h: f32, color: Color) -> Self {
        let mut r = Self::new(x, y, w, h); r.bg_color = color; r
    }

    /// Create an outline-only rect.
    pub fn outlined(x: f32, y: f32, w: f32, h: f32, border_color: Color, border_width: f32) -> Self {
        let mut r = Self::new(x, y, w, h);
        r.border_color = border_color; r.border_width = border_width; r
    }

    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }
    pub fn set_bg_color(&mut self, c: Color)      { self.bg_color = c; }
    pub fn set_border(&mut self, color: Color, width: f32) { self.border_color = color; self.border_width = width; }
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    pub fn draw(&self) {
        if !self.visible { return; }
        let layout = ResolvedLayout { x: self.x, y: self.y, w: self.w, h: self.h, ..Default::default() };
        draw(layout, self.bg_color, self.border_color, self.border_width, self.corner_radius);
    }
}
