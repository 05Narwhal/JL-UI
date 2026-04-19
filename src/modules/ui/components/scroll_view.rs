// * =========== MODULE: SCROLL VIEW =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

pub fn draw(layout: ResolvedLayout, bg_color: Color) {
    if bg_color.a > 0.0 {
        draw_rectangle(layout.x, layout.y, layout.w, layout.h, bg_color);
    }
}

pub struct UIScrollView {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub bg_color: Color,
    pub visible: bool,
    /// Accumulated vertical scroll offset in pixels (positive = scrolled down).
    pub scroll_y: f32,
    /// Accumulated horizontal scroll offset in pixels (positive = scrolled right).
    pub scroll_x: f32,
    pub scroll_speed: f32,
    pub scroll_x_enabled: bool,
    pub scroll_y_enabled: bool,
}

impl UIScrollView {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x, y, w, h,
            bg_color: Color::new(0.0, 0.0, 0.0, 0.0),
            visible: true,
            scroll_y: 0.0,
            scroll_x: 0.0,
            scroll_speed: 30.0,
            scroll_x_enabled: false,
            scroll_y_enabled: true,
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────
    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }
    pub fn set_bg_color(&mut self, c: Color)   { self.bg_color = c; }
    pub fn reset_scroll(&mut self)             { self.scroll_x = 0.0; self.scroll_y = 0.0; }

    /// Returns `true` if the pointer is currently inside the scroll view.
    pub fn is_hovered(&self) -> bool {
        let (mx, my) = mouse_position();
        mx >= self.x && mx <= self.x + self.w && my >= self.y && my <= self.y + self.h
    }

    // ── Draw (processes scroll wheel while hovered) ───────────────────────

    /// Draws the background and updates scroll offsets from mouse wheel input.
    /// Use `scroll_x` / `scroll_y` to offset the positions of child widgets.
    pub fn draw(&mut self) {
        if !self.visible { return; }
        let layout = ResolvedLayout { x: self.x, y: self.y, w: self.w, h: self.h, ..Default::default() };
        draw(layout, self.bg_color);

        if self.is_hovered() {
            let (wx, wy) = mouse_wheel();
            if self.scroll_y_enabled { self.scroll_y -= wy * self.scroll_speed; }
            if self.scroll_x_enabled { self.scroll_x -= wx * self.scroll_speed; }
        }
    }
}
