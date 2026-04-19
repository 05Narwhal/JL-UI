// * =========== MODULE: BUTTON =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

/// Draw the button background and its optional built-in label text.
/// `effective_scale` is the accumulated parent*self scale — applied to font size
/// so the text grows/shrinks together with the button when scaled.
pub fn draw(
    layout: ResolvedLayout,
    bg_color: Color,
    text: &str,
    text_color: Color,
    font_size: f32,
    hov: &mut bool,
    prs: &mut bool,
    hovered: bool,
    lmb_down: bool,
) {
    *hov = hovered;
    *prs = hovered && lmb_down;

    let bg = if *prs {
        Color::new(0.55, 0.55, 0.55, 1.0)
    } else if *hov {
        Color::new(0.45, 0.45, 0.45, 1.0)
    } else if bg_color.a > 0.0 {
        bg_color
    } else {
        Color::new(0.3, 0.3, 0.3, 1.0)
    };

    draw_rectangle(layout.x, layout.y, layout.w, layout.h, bg);

    if !text.is_empty() {
        let ts = measure_text(text, None, font_size as u16, 1.0);
        let tx = layout.x + (layout.w - ts.width) / 2.0;
        let ty = layout.y + (layout.h - ts.height) / 2.0 + ts.offset_y;
        draw_text(text, tx, ty, font_size, text_color);
    }
}

pub struct UIButton {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub bg_color: Color,
    /// Custom hover-state background color. Falls back to built-in lighter shade if `None`.
    pub hover_color: Option<Color>,
    /// Custom pressed-state background color. Falls back to built-in darker shade if `None`.
    pub press_color: Option<Color>,
    pub text: String,
    pub text_color: Color,
    pub font_size: f32,
    pub hovered: bool,
    pub pressed: bool,
    pub visible: bool,
    pub enabled: bool,
}

impl UIButton {
    pub fn new(x: f32, y: f32, w: f32, h: f32, text: impl Into<String>, font_size: f32) -> Self {
        Self {
            x, y, w, h,
            bg_color: Color::new(0.3, 0.3, 0.3, 1.0),
            hover_color: None,
            press_color: None,
            text: text.into(),
            text_color: WHITE,
            font_size,
            hovered: false,
            pressed: false,
            visible: true,
            enabled: true,
        }
    }

    // ── Position / size helpers ───────────────────────────────────────────────
    pub fn set_pos(&mut self, x: f32, y: f32)   { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32)  { self.w = w; self.h = h; }
    pub fn bounds(&self) -> (f32, f32, f32, f32) { (self.x, self.y, self.w, self.h) }
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    // ── Style helpers ────────────────────────────────────────────────────────
    pub fn set_text(&mut self, t: impl Into<String>) { self.text = t.into(); }
    pub fn set_bg_color(&mut self, c: Color)         { self.bg_color = c; }
    pub fn set_text_color(&mut self, c: Color)       { self.text_color = c; }
    pub fn set_font_size(&mut self, s: f32)          { self.font_size = s; }

    // ── Input state ──────────────────────────────────────────────────────────

    fn in_bounds(&self) -> bool {
        let (mx, my) = mouse_position();
        self.enabled && self.visible && self.contains_point(mx, my)
    }

    /// `true` while the cursor is over the button.
    pub fn on_hover(&self)   -> bool { self.in_bounds() }
    /// `true` the frame the left mouse button is pressed over the button.
    pub fn on_click(&self)   -> bool { self.in_bounds() && is_mouse_button_pressed(MouseButton::Left) }
    /// `true` the frame the left mouse button is released over the button.
    pub fn on_release(&self) -> bool { self.in_bounds() && is_mouse_button_released(MouseButton::Left) }
    /// `true` while the left mouse button is held down over the button.
    pub fn on_held(&self)    -> bool { self.in_bounds() && is_mouse_button_down(MouseButton::Left) }

    // ── Draw ─────────────────────────────────────────────────────────────────

    /// Renders the button and updates hover/press state.
    /// Returns `true` the frame the button is released (clicked).
    pub fn draw(&mut self) -> bool {
        if !self.visible { return false; }
        let (mx, my)  = mouse_position();
        let hovered   = self.enabled && self.contains_point(mx, my);
        let lmb_down  = is_mouse_button_down(MouseButton::Left);
        let clicked   = hovered && is_mouse_button_released(MouseButton::Left);

        // Override colours if set
        let pressed_now = hovered && lmb_down;
        let effective_bg = if pressed_now {
            self.press_color.unwrap_or(Color::new(0.55, 0.55, 0.55, 1.0))
        } else if hovered {
            self.hover_color.unwrap_or(Color::new(0.45, 0.45, 0.45, 1.0))
        } else {
            self.bg_color
        };

        let layout = ResolvedLayout { x: self.x, y: self.y, w: self.w, h: self.h, ..Default::default() };
        // Pass transparent so the `draw` fn uses our pre-computed effective_bg
        draw(layout, effective_bg, &self.text, self.text_color, self.font_size,
             &mut self.hovered, &mut self.pressed, hovered, lmb_down);
        clicked
    }
}
