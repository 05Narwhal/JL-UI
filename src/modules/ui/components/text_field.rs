// * =========== MODULE: TEXT FIELD =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

pub fn draw(
    layout: ResolvedLayout,
    font_size: f32,
    text: &mut String,
    placeholder: &str,
    color: Color,
    focused: &mut bool,
    hovered: bool,
    lmb_pressed: bool,
) {
    if lmb_pressed { *focused = hovered; }

    let border = if *focused { Color::new(0.4, 0.7, 1.0, 1.0) } else { Color::new(0.3, 0.3, 0.3, 1.0) };
    draw_rectangle(layout.x, layout.y, layout.w, layout.h, Color::new(0.15, 0.15, 0.15, 1.0));
    draw_rectangle_lines(layout.x, layout.y, layout.w, layout.h, 2.0, border);

    if *focused {
        while let Some(c) = get_char_pressed() {
            if !c.is_control() { text.push(c); }
        }
        if is_key_pressed(KeyCode::Backspace) && !text.is_empty() {
            text.pop();
        }
    }

    let display     = if text.is_empty() { placeholder } else { text.as_str() };
    let text_color  = if text.is_empty() { Color::new(0.5, 0.5, 0.5, 1.0) } else { color };
    draw_text(display, layout.x + 4.0, layout.y + font_size + 2.0, font_size, text_color);

    if *focused {
        let ts      = measure_text(display, None, font_size as u16, 1.0);
        let cx      = layout.x + 4.0 + ts.width + 1.0;
        let cy_top  = layout.y + 4.0;
        let cy_bot  = layout.y + layout.h - 4.0;
        if ((get_time() * 2.0) as u64 % 2) == 0 {
            draw_line(cx, cy_top, cx, cy_bot, 1.5, color);
        }
    }
}

pub struct UITextField {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub font_size: f32,
    pub text: String,
    pub placeholder: String,
    pub color: Color,
    pub focused: bool,
    pub visible: bool,
    pub enabled: bool,
    /// Maximum number of characters (0 = unlimited).
    pub max_length: usize,
}

impl UITextField {
    pub fn new(x: f32, y: f32, w: f32, h: f32, placeholder: impl Into<String>, font_size: f32) -> Self {
        Self {
            x, y, w, h, font_size,
            text: String::new(),
            placeholder: placeholder.into(),
            color: WHITE,
            focused: false,
            visible: true,
            enabled: true,
            max_length: 0,
        }
    }

    // ── Text helpers ─────────────────────────────────────────────────────────
    pub fn set_text(&mut self, t: impl Into<String>) { self.text = t.into(); }
    pub fn clear(&mut self) { self.text.clear(); }
    pub fn get_text(&self) -> &str { &self.text }
    pub fn is_empty(&self) -> bool { self.text.is_empty() }
    pub fn char_count(&self) -> usize { self.text.chars().count() }

    /// Set focus programmatically (without a click).
    pub fn focus(&mut self)   { self.focused = true; }
    pub fn unfocus(&mut self) { self.focused = false; }

    // ── Position / size ──────────────────────────────────────────────────────
    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }

    // ── Draw ─────────────────────────────────────────────────────────────────

    pub fn draw(&mut self) {
        if !self.visible { return; }
        let (mx, my)    = mouse_position();
        let hovered     = self.enabled
            && mx >= self.x && mx <= self.x + self.w
            && my >= self.y && my <= self.y + self.h;
        let lmb_pressed = is_mouse_button_pressed(MouseButton::Left);
        let layout      = ResolvedLayout { x: self.x, y: self.y, w: self.w, h: self.h, ..Default::default() };

        // Enforce max_length before invoking the free-fn
        if self.focused && self.max_length > 0 {
            // Trim before the free-fn processes more key input
            while self.text.chars().count() > self.max_length {
                self.text.pop();
            }
        }

        draw(layout, self.font_size, &mut self.text, &self.placeholder, self.color, &mut self.focused, hovered, lmb_pressed);

        // Re-enforce after the free-fn may have appended characters
        if self.max_length > 0 {
            while self.text.chars().count() > self.max_length {
                self.text.pop();
            }
        }
    }
}
