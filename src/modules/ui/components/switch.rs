// * =========== MODULE: SWITCH =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::ResolvedLayout;

pub fn draw(
    layout: ResolvedLayout,
    checked: &mut bool,
    color: Color,
    hovered: bool,
    lmb_pressed: bool,
) {
    if hovered && lmb_pressed { *checked = !*checked; }

    let bg = if *checked { color } else { Color::new(0.4, 0.4, 0.4, 1.0) };
    draw_rectangle(layout.x, layout.y, layout.w, layout.h, bg);

    let knob_x = if *checked { layout.x + layout.w - layout.h } else { layout.x };
    draw_rectangle(knob_x, layout.y, layout.h, layout.h, WHITE);
}

pub struct UISwitch {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub checked: bool,
    pub color: Color,
    /// Color shown when the switch is off. Defaults to grey.
    pub off_color: Color,
    pub visible: bool,
    pub enabled: bool,
    /// `true` for exactly one frame when the switch changes state.
    pub changed: bool,
}

impl UISwitch {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x, y, w, h,
            checked: false,
            color: Color::new(0.0, 0.8, 0.3, 1.0),
            off_color: Color::new(0.4, 0.4, 0.4, 1.0),
            visible: true,
            enabled: true,
            changed: false,
        }
    }

    // ── State helpers ────────────────────────────────────────────────────────
    pub fn toggle(&mut self)          { self.checked = !self.checked; }
    pub fn set_checked(&mut self, v: bool) { self.checked = v; }
    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }

    /// Calls `callback` with the new `checked` state the frame it changes.
    pub fn on_change<F: FnOnce(bool)>(&self, callback: F) {
        if self.changed { callback(self.checked); }
    }

    // ── Draw ─────────────────────────────────────────────────────────────────

    pub fn draw(&mut self) {
        if !self.visible { return; }
        let (mx, my)    = mouse_position();
        let hovered     = self.enabled
            && mx >= self.x && mx <= self.x + self.w
            && my >= self.y && my <= self.y + self.h;
        let lmb_pressed = is_mouse_button_pressed(MouseButton::Left);

        let before = self.checked;
        let layout = ResolvedLayout { x: self.x, y: self.y, w: self.w, h: self.h, ..Default::default() };

        // Temporarily swap bg so the draw fn uses our off_color
        if !self.checked {
            // The free-fn always uses `color` for on-state; we patch to off_color via direct draw.
            if hovered && lmb_pressed { self.checked = !self.checked; }
            let bg = if self.checked { self.color } else { self.off_color };
            draw_rectangle(self.x, self.y, self.w, self.h, bg);
            let knob_x = if self.checked { self.x + self.w - self.h } else { self.x };
            draw_rectangle(knob_x, self.y, self.h, self.h, WHITE);
        } else {
            draw(layout, &mut self.checked, self.color, hovered, lmb_pressed);
        }

        self.changed = self.checked != before;
    }
}
