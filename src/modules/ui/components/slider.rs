// * =========== MODULE: SLIDER =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

pub fn draw(
    layout: ResolvedLayout,
    value: &mut f32,
    min: f32,
    max: f32,
    thumb_color: Color,
    track_color: Color,
    dragging: &mut bool,
    changed: &mut bool,
    hovered: bool,
    lmb_pressed: bool,
    lmb_down: bool,
    mx: f32,
) {
    let prev = *value;
    if hovered && lmb_pressed { *dragging = true; }
    if !lmb_down             { *dragging = false; }
    if *dragging {
        let range  = max - min;
        let usable = (layout.w - 12.0).max(1.0);
        let raw    = ((mx - layout.x - 6.0) / usable).clamp(0.0, 1.0);
        *value = min + raw * range;
    }
    *changed = (*value - prev).abs() > f32::EPSILON;

    let track_y = layout.y + layout.h / 2.0 - 2.0;
    draw_rectangle(layout.x, track_y, layout.w, 4.0, track_color);

    let range = max - min;
    let pct   = if range > 0.0 { (*value - min) / range } else { 0.0 };
    let thumb_x = layout.x + pct * (layout.w - 12.0);
    draw_rectangle(thumb_x, layout.y, 12.0, layout.h, thumb_color);
}

pub struct UISlider {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub thumb_color: Color,
    pub track_color: Color,
    pub dragging: bool,
    /// `true` for exactly one frame after the value changes.
    pub changed: bool,
    pub visible: bool,
    pub enabled: bool,
}

impl UISlider {
    pub fn new(x: f32, y: f32, w: f32, h: f32, min: f32, max: f32) -> Self {
        Self {
            x, y, w, h, value: min, min, max,
            thumb_color: WHITE,
            track_color: Color::new(0.3, 0.3, 0.3, 1.0),
            dragging: false,
            changed: false,
            visible: true,
            enabled: true,
        }
    }

    // ── Value helpers ────────────────────────────────────────────────────────

    /// Set value clamped to [min, max].
    pub fn set_value(&mut self, v: f32) { self.value = v.clamp(self.min, self.max); }
    /// Normalised value in [0, 1].
    pub fn normalised(&self) -> f32 {
        if (self.max - self.min).abs() < f32::EPSILON { 0.0 }
        else { (self.value - self.min) / (self.max - self.min) }
    }
    /// Set value via a normalised position [0, 1].
    pub fn set_normalised(&mut self, t: f32) { self.value = self.min + t.clamp(0.0, 1.0) * (self.max - self.min); }
    pub fn set_range(&mut self, min: f32, max: f32) {
        self.min = min; self.max = max;
        self.value = self.value.clamp(min, max);
    }

    // ── Position / size ──────────────────────────────────────────────────────
    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }

    // ── Callback helper ──────────────────────────────────────────────────────
    /// Calls `callback` with the current value if it changed this frame.
    pub fn on_change<F: FnOnce(f32)>(&self, callback: F) {
        if self.changed { callback(self.value); }
    }

    // ── Draw ─────────────────────────────────────────────────────────────────

    pub fn draw(&mut self) {
        if !self.visible { return; }
        let (mx, my)    = mouse_position();
        let hovered     = self.enabled
            && mx >= self.x && mx <= self.x + self.w
            && my >= self.y && my <= self.y + self.h;
        let lmb_pressed = is_mouse_button_pressed(MouseButton::Left);
        let lmb_down    = is_mouse_button_down(MouseButton::Left);
        let layout      = ResolvedLayout { x: self.x, y: self.y, w: self.w, h: self.h, ..Default::default() };
        draw(
            layout, &mut self.value, self.min, self.max,
            self.thumb_color, self.track_color,
            &mut self.dragging, &mut self.changed,
            hovered, lmb_pressed, lmb_down, mx,
        );
    }
}
