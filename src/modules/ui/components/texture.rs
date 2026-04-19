// * =========== MODULE: TEXTURE =========== *
use std::collections::HashMap;
use macroquad::prelude::*;
use crate::modules::ui::types::ui::{ObjectFit, ResolvedLayout};

pub fn draw(
    layout: ResolvedLayout,
    src: &str,
    object_fit: ObjectFit,
    texture_cache: &HashMap<String, Texture2D>,
    bg_color: Color,
) {
    if let Some(texture) = texture_cache.get(src) {
        let tex_w = texture.width();
        let tex_h = texture.height();
        match object_fit {
            ObjectFit::Warp => {
                draw_texture_ex(
                    texture, layout.x, layout.y, WHITE,
                    DrawTextureParams { dest_size: Some(vec2(layout.w, layout.h)), ..Default::default() },
                );
            }
            ObjectFit::Cover => {
                let s     = (layout.w / tex_w).max(layout.h / tex_h);
                let vis_w = layout.w / s;
                let vis_h = layout.h / s;
                let src_x = (tex_w - vis_w) / 2.0;
                let src_y = (tex_h - vis_h) / 2.0;
                draw_texture_ex(
                    texture, layout.x, layout.y, WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(layout.w, layout.h)),
                        source: Some(Rect::new(src_x, src_y, vis_w, vis_h)),
                        ..Default::default()
                    },
                );
            }
            ObjectFit::Contain => {
                let s  = (layout.w / tex_w).min(layout.h / tex_h);
                let dw = tex_w * s;
                let dh = tex_h * s;
                let dx = layout.x + (layout.w - dw) / 2.0;
                let dy = layout.y + (layout.h - dh) / 2.0;
                draw_texture_ex(
                    texture, dx, dy, WHITE,
                    DrawTextureParams { dest_size: Some(vec2(dw, dh)), ..Default::default() },
                );
            }
        }
    } else if bg_color.a > 0.0 {
        draw_rectangle(layout.x, layout.y, layout.w, layout.h, bg_color);
    }
}

pub struct UITexture {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub object_fit: ObjectFit,
    pub bg_color: Color,
    /// Draw opacity: 0.0 (invisible) – 1.0 (fully opaque).
    pub opacity: f32,
    pub visible: bool,
    texture: Option<Texture2D>,
}

impl UITexture {
    /// Create an empty texture placeholder (renders `bg_color` until a texture is loaded).
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x, y, w, h,
            object_fit: ObjectFit::default(),
            bg_color: Color::new(0.0, 0.0, 0.0, 0.0),
            opacity: 1.0,
            visible: true,
            texture: None,
        }
    }

    /// Load a texture from `path` asynchronously and return a ready-to-draw instance.
    pub async fn load(x: f32, y: f32, w: f32, h: f32, path: &str) -> Self {
        let texture = match load_texture(path).await {
            Ok(t) => {
                t.set_filter(FilterMode::Nearest);
                crate::ui_trace!("[UITexture] Loaded '{}'", path);
                Some(t)
            }
            Err(e) => {
                crate::ui_error!("[UITexture] Failed to load '{}': {:?}", path, e);
                None
            }
        };
        Self { x, y, w, h, object_fit: ObjectFit::default(), bg_color: Color::new(0.0, 0.0, 0.0, 0.0), opacity: 1.0, visible: true, texture }
    }

    // ── Texture helpers ────────────────────────────────────────────────────

    pub fn is_loaded(&self) -> bool { self.texture.is_some() }

    /// Attach an already-loaded `Texture2D` directly.
    pub fn set_texture_raw(&mut self, tex: Texture2D) { self.texture = Some(tex); }
    pub fn clear_texture(&mut self) { self.texture = None; }
    pub fn set_opacity(&mut self, opacity: f32) { self.opacity = opacity.clamp(0.0, 1.0); }
    pub fn set_object_fit(&mut self, fit: ObjectFit) { self.object_fit = fit; }
    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }

    /// Returns the natural (unscaled) size of the loaded texture, or (0, 0) if not loaded.
    pub fn natural_size(&self) -> (f32, f32) {
        match &self.texture {
            Some(t) => (t.width(), t.height()),
            None    => (0.0, 0.0),
        }
    }

    /// Replace the current texture by loading a new one from `path`.
    pub async fn set_texture(&mut self, path: &str) {
        self.texture = match load_texture(path).await {
            Ok(t) => { t.set_filter(FilterMode::Nearest); Some(t) }
            Err(e) => { crate::ui_error!("[UITexture] Failed to swap '{}': {:?}", path, e); None }
        };
    }

    // ── Draw ────────────────────────────────────────────────────────────────

    pub fn draw(&self) {
        if !self.visible { return; }
        let tint = Color::new(1.0, 1.0, 1.0, self.opacity.clamp(0.0, 1.0));
        if let Some(ref texture) = self.texture {
            let tex_w = texture.width();
            let tex_h = texture.height();
            match self.object_fit {
                ObjectFit::Warp => {
                    draw_texture_ex(
                        texture, self.x, self.y, tint,
                        DrawTextureParams { dest_size: Some(vec2(self.w, self.h)), ..Default::default() },
                    );
                }
                ObjectFit::Cover => {
                    let s     = (self.w / tex_w).max(self.h / tex_h);
                    let vis_w = self.w / s;
                    let vis_h = self.h / s;
                    let src_x = (tex_w - vis_w) / 2.0;
                    let src_y = (tex_h - vis_h) / 2.0;
                    draw_texture_ex(
                        texture, self.x, self.y, tint,
                        DrawTextureParams {
                            dest_size: Some(vec2(self.w, self.h)),
                            source:    Some(Rect::new(src_x, src_y, vis_w, vis_h)),
                            ..Default::default()
                        },
                    );
                }
                ObjectFit::Contain => {
                    let s  = (self.w / tex_w).min(self.h / tex_h);
                    let dw = tex_w * s;
                    let dh = tex_h * s;
                    let dx = self.x + (self.w - dw) / 2.0;
                    let dy = self.y + (self.h - dh) / 2.0;
                    draw_texture_ex(
                        texture, dx, dy, tint,
                        DrawTextureParams { dest_size: Some(vec2(dw, dh)), ..Default::default() },
                    );
                }
            }
        } else if self.bg_color.a > 0.0 {
            draw_rectangle(self.x, self.y, self.w, self.h, self.bg_color);
        }
    }
}
