// * =========== MODULE: LABEL =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

/// Draw a label.
/// `effective_scale` is the accumulated parent*self scale — multiplied into the
/// font size so the text grows/shrinks when any ancestor is scaled.
pub fn draw(layout: ResolvedLayout, text: &str, color: Color, font_size: f32, effective_scale: f32) {
    let scaled_font = font_size * effective_scale;
    let ts = measure_text(text, None, scaled_font as u16, 1.0);
    draw_text(text, layout.x, layout.y + ts.offset_y, scaled_font, color);
}

pub struct UILabel {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: Color,
    pub font_size: f32,
    /// Uniform scale multiplier applied to `font_size` at draw time.
    pub scale: f32,
    pub visible: bool,
}

impl UILabel {
    pub fn new(x: f32, y: f32, text: impl Into<String>, font_size: f32) -> Self {
        Self { x, y, text: text.into(), color: WHITE, font_size, scale: 1.0, visible: true }
    }

    // ── Helpers ────────────────────────────────────────────────────────────
    pub fn set_text(&mut self, t: impl Into<String>) { self.text = t.into(); }
    pub fn set_color(&mut self, c: Color)            { self.color = c; }
    pub fn set_pos(&mut self, x: f32, y: f32)        { self.x = x; self.y = y; }
    pub fn set_font_size(&mut self, s: f32)          { self.font_size = s; }

    /// Returns the draw size of the current text at the current scaled font size.
    pub fn measure(&self) -> (f32, f32) {
        let sf = self.font_size * self.scale;
        let m  = measure_text(&self.text, None, sf as u16, 1.0);
        (m.width, m.height)
    }

    pub fn draw(&self) {
        if !self.visible { return; }
        let layout = ResolvedLayout { x: self.x, y: self.y, w: 0.0, h: 0.0, ..Default::default() };
        draw(layout, &self.text, self.color, self.font_size, self.scale);
    }
}
