// * =========== MODULE: PROGRESS BAR =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

pub fn draw(layout: ResolvedLayout, value: f32, max: f32, color: Color) {
    draw_rectangle(layout.x, layout.y, layout.w, layout.h, Color::new(0.2, 0.2, 0.2, 1.0));
    let pct = if max > 0.0 { value / max } else { 0.0 };
    draw_rectangle(layout.x, layout.y, layout.w * pct, layout.h, color);
}

pub struct UIProgressBar {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub value: f32,
    pub max: f32,
    pub color: Color,
    /// Background track color (defaults to dark grey).
    pub track_color: Color,
    pub visible: bool,
}

impl UIProgressBar {
    pub fn new(x: f32, y: f32, w: f32, h: f32, max: f32) -> Self {
        Self {
            x, y, w, h, value: 0.0, max,
            color: Color::new(0.3, 0.7, 1.0, 1.0),
            track_color: Color::new(0.2, 0.2, 0.2, 1.0),
            visible: true,
        }
    }

    // ── Value helpers ───────────────────────────────────────────────────────

    pub fn set_value(&mut self, v: f32)   { self.value = v.clamp(0.0, self.max); }
    pub fn set_max(&mut self, m: f32)     { self.max = m.max(0.0); self.value = self.value.min(self.max); }
    /// Set value as a fraction [0, 1] of `max`.
    pub fn set_pct(&mut self, pct: f32)   { self.value = pct.clamp(0.0, 1.0) * self.max; }
    /// Returns the fill fraction in [0, 1].
    pub fn pct(&self) -> f32 {
        if self.max > 0.0 { self.value / self.max } else { 0.0 }
    }
    pub fn is_full(&self) -> bool  { (self.value - self.max).abs() < f32::EPSILON }
    pub fn is_empty(&self) -> bool { self.value <= 0.0 }
    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }

    // ── Draw ────────────────────────────────────────────────────────────────

    pub fn draw(&self) {
        if !self.visible { return; }
        draw_rectangle(self.x, self.y, self.w, self.h, self.track_color);
        let pct = self.pct();
        draw_rectangle(self.x, self.y, self.w * pct, self.h, self.color);
    }
}
