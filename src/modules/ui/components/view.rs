// * =========== MODULE: VIEW =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

pub fn draw(layout: ResolvedLayout, bg_color: Color) {
    if bg_color.a > 0.0 {
        draw_rectangle(layout.x, layout.y, layout.w, layout.h, bg_color);
    }
}

pub struct UIView {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub bg_color: Color,
    pub visible: bool,
}

impl UIView {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h, bg_color: Color::new(0.0, 0.0, 0.0, 0.0), visible: true }
    }

    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }
    pub fn set_bg_color(&mut self, c: Color)   { self.bg_color = c; }
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    pub fn draw(&self) {
        if !self.visible { return; }
        let layout = ResolvedLayout { x: self.x, y: self.y, w: self.w, h: self.h, ..Default::default() };
        draw(layout, self.bg_color);
    }
}
