//! 3D physics container helpers.

use crate::modules::ui::types::ui::ResolvedLayout;
use macroquad::prelude::*;

/// Draw the optional background for a 3D physics container.
///
/// The embedded engine content is rendered through callbacks managed by `UI`.
pub fn draw(layout: ResolvedLayout, bg_color: Color) {
    if bg_color.a > 0.0 {
        draw_rectangle(layout.x, layout.y, layout.w, layout.h, bg_color);
    }
}

/// Lightweight standalone 3D physics container model.
#[derive(Debug, Clone, Copy)]
pub struct UIPhysicsContainer3D {
    /// Left position in pixels.
    pub x: f32,
    /// Top position in pixels.
    pub y: f32,
    /// Width in pixels.
    pub w: f32,
    /// Height in pixels.
    pub h: f32,
    /// Background color for the container.
    pub bg_color: Color,
    /// Visibility flag.
    pub visible: bool,
}

impl UIPhysicsContainer3D {
    /// Create a new container.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            bg_color: Color::new(0.0, 0.0, 0.0, 0.0),
            visible: true,
        }
    }

    /// Update container position.
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    /// Update container size.
    pub fn set_size(&mut self, w: f32, h: f32) {
        self.w = w;
        self.h = h;
    }

    /// Update background color.
    pub fn set_bg_color(&mut self, c: Color) {
        self.bg_color = c;
    }

    /// Render the container background.
    pub fn draw(&self) {
        if !self.visible {
            return;
        }
        let layout = ResolvedLayout {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
            ..Default::default()
        };
        draw(layout, self.bg_color);
    }
}
