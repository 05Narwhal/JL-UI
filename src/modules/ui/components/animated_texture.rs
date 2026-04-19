// * =========== MODULE: ANIMATED TEXTURE =========== *
use std::{collections::HashMap, f32::consts::PI, path::PathBuf};
use macroquad::prelude::*;
use crate::modules::ui::types::ui::{ObjectFit, ResolvedLayout};

// ── conf.jsonc serde structs ──────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct AnimConfRaw {
    #[serde(default)]
    timing_function: String,
    frames: AnimFramesRaw,
}

#[derive(serde::Deserialize)]
struct AnimFramesRaw {
    path:   String,
    format: String,
    count:  u32,
}

// ── Cubic-bezier easing (identical to the renderer's) ────────────────────────

fn cubic_bezier_easing(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    if t <= 0.0 { return 0.0; }
    if t >= 1.0 { return 1.0; }
    let cx = 3.0 * x1; let bx = 3.0 * (x2 - x1) - cx; let ax = 1.0 - cx - bx;
    let cy = 3.0 * y1; let by_ = 3.0 * (y2 - y1) - cy; let ay = 1.0 - cy - by_;
    let mut s = t;
    for _ in 0..8 {
        let x_err = ((ax * s + bx) * s + cx) * s - t;
        let dx    = (3.0 * ax * s + 2.0 * bx) * s + cx;
        if dx.abs() < 1e-6 { break; }
        s = (s - x_err / dx).clamp(0.0, 1.0);
    }
    ((ay * s + by_) * s + cy) * s
}

// ── Helpers matching the XML loader ──────────────────────────────────────────

fn strip_jsonc_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escaped   = false;
    let mut chars     = text.chars().peekable();
    while let Some(c) = chars.next() {
        if escaped { escaped = false; out.push(c); continue; }
        if in_string {
            if c == '\\' { escaped = true; out.push(c); continue; }
            if c == '"'  { in_string = false; }
            out.push(c); continue;
        }
        if c == '"' { in_string = true; out.push(c); continue; }
        if c == '/' && chars.peek() == Some(&'/') {
            while let Some(nc) = chars.next() { if nc == '\n' { out.push('\n'); break; } }
            continue;
        }
        out.push(c);
    }
    out
}

fn parse_timing_function(s: &str) -> [f32; 4] {
    let v: Vec<f32> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    if v.len() == 4 { [v[0], v[1], v[2], v[3]] } else { [0.42, 0.0, 0.58, 1.0] }
}

fn expand_frame_format(format: &str, i: u32) -> String {
    if let Some(pos) = format.find('$') {
        let rest    = &format[pos + 1..];
        let num_end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        let start: u32 = rest[..num_end].parse().unwrap_or(0);
        format!("{}{}{}", &format[..pos], start + i, &rest[num_end..])
    } else {
        format.to_string()
    }
}

// ── UIAnimatedTexture ─────────────────────────────────────────────────────────

pub struct UIAnimatedTexture {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub fps: f32,
    pub loop_anim: bool,
    pub autoplay: bool,
    pub opacity: f32,
    pub object_fit: ObjectFit,
    pub bg_color: Color,
    pub playing: bool,
    pub paused: bool,
    pub current_frame: u32,
    pub elapsed: f32,
    /// CSS cubic-bezier control points [x1, y1, x2, y2] read from `conf.jsonc`.
    pub timing: [f32; 4],
    frames: Vec<Texture2D>,
    flip_x: bool,
    rotation: f32,
}

impl UIAnimatedTexture {
    /// Create an empty instance. Use `load()` to load from a `conf.jsonc` folder.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x, y, w, h,
            fps: 12.0,
            loop_anim: true,
            autoplay: false,
            opacity: 1.0,
            object_fit: ObjectFit::default(),
            bg_color: Color::new(0.0, 0.0, 0.0, 0.0),
            playing: false,
            paused: false,
            current_frame: 0,
            elapsed: 0.0,
            timing: [0.42, 0.0, 0.58, 1.0],
            frames: Vec::new(),
            flip_x: false,
            rotation: 0.0,
        }
    }

    /// Load an animated texture from a folder that contains a `conf.jsonc` file.
    ///
    /// The `conf.jsonc` format mirrors the XML system:
    /// ```jsonc
    /// {
    ///   "timing_function": "0.42,0,0.58,1",  // CSS cubic-bezier
    ///   "frames": {
    ///     "path": "./frames",   // sub-folder relative to the conf directory
    ///     "format": "$0.png",   // $0 = frame index starting at 0, $1 = starting at 1
    ///     "count": 8
    ///   }
    /// }
    /// ```
    pub async fn load(x: f32, y: f32, w: f32, h: f32, folder: impl Into<PathBuf>, fps: f32, loop_anim: bool, autoplay: bool) -> Self {
        let folder: PathBuf = folder.into();
        let conf_path = folder.join("conf.jsonc");

        let conf_text = match std::fs::read_to_string(&conf_path) {
            Ok(t)  => t,
            Err(e) => {
                crate::ui_error!("[UIAnimatedTexture] Cannot read conf.jsonc at {:?}: {}", conf_path, e);
                return Self::new(x, y, w, h);
            }
        };

        let stripped = strip_jsonc_comments(&conf_text);
        let conf: AnimConfRaw = match serde_json::from_str(&stripped) {
            Ok(c)  => c,
            Err(e) => {
                crate::ui_error!("[UIAnimatedTexture] Cannot parse conf.jsonc: {}", e);
                return Self::new(x, y, w, h);
            }
        };

        let timing     = parse_timing_function(&conf.timing_function);
        let count      = conf.frames.count;
        let frames_sub = conf.frames.path.trim_start_matches("./").trim_start_matches('/');
        let frames_dir = if frames_sub.is_empty() { folder.clone() } else { folder.join(frames_sub) };

        let mut frames = Vec::with_capacity(count as usize);
        for i in 0..count {
            let filename   = expand_frame_format(&conf.frames.format, i);
            let frame_path = frames_dir.join(&filename);
            match load_texture(&frame_path.to_string_lossy()).await {
                Ok(tex) => {
                    tex.set_filter(FilterMode::Nearest);
                    frames.push(tex);
                    crate::ui_trace!("[UIAnimatedTexture] Loaded frame {}", i);
                }
                Err(e) => {
                    crate::ui_error!("[UIAnimatedTexture] Frame {} failed ({:?}): {:?}", i, frame_path, e);
                    break;
                }
            }
        }

        Self {
            x, y, w, h, fps, loop_anim, autoplay,
            opacity: 1.0,
            object_fit: ObjectFit::default(),
            bg_color: Color::new(0.0, 0.0, 0.0, 0.0),
            playing: autoplay && !frames.is_empty(),
            paused: false,
            current_frame: 0,
            elapsed: 0.0,
            timing,
            frames,
            flip_x: false,
            rotation: 0.0,
        }
    }

    // ── Playback controls ────────────────────────────────────────────────────

    pub fn play(&mut self)  { self.playing = true;  self.paused = false; }
    pub fn pause(&mut self) { self.paused = true;  self.playing = false; }
    pub fn resume(&mut self) { if self.paused { self.playing = true; self.paused = false; } }
    pub fn stop(&mut self)  {
        self.playing = false; self.paused = false;
        self.current_frame = 0; self.elapsed = 0.0;
    }
    pub fn restart(&mut self) { self.stop(); self.play(); }

    pub fn frame_count(&self) -> usize { self.frames.len() }
    pub fn is_finished(&self) -> bool {
        !self.loop_anim && !self.playing && self.current_frame + 1 >= self.frames.len() as u32
    }
    pub fn is_loaded(&self) -> bool { !self.frames.is_empty() }

    pub fn rotate_deg(&mut self, angle_deg: f32) { self.rotation = (angle_deg % 360.0).to_radians(); }
    pub fn rotate_rad(&mut self, angle_rad: f32) { self.rotation = angle_rad % (2.0 * PI); }
    pub fn set_flip_x(&mut self, flip: bool) { self.flip_x = flip; }

    /// Seek to a specific frame index (clamped). Updates `elapsed` for smooth resume.
    pub fn seek(&mut self, frame: u32) {
        let total = self.frames.len() as u32;
        self.current_frame = frame.min(total.saturating_sub(1));
        if self.fps > 0.0 { self.elapsed = self.current_frame as f32 / self.fps; }
    }

    pub fn set_fps(&mut self, fps: f32) { self.fps = fps.max(0.0); }
    pub fn set_opacity(&mut self, opacity: f32) { self.opacity = opacity.clamp(0.0, 1.0); }

    // ── Per-frame advance using cubic-bezier easing ──────────────────────────

    fn advance(&mut self) {
        if !self.playing || self.paused || self.fps <= 0.0 || self.frames.is_empty() { return; }
        let total = self.frames.len() as u32;
        if total == 0 { return; }

        self.elapsed += get_frame_time();
        let total_dur = total as f32 / self.fps;

        let raw_t = if self.loop_anim {
            (self.elapsed % total_dur) / total_dur
        } else {
            (self.elapsed / total_dur).min(1.0)
        };

        let eased = cubic_bezier_easing(
            raw_t, self.timing[0], self.timing[1], self.timing[2], self.timing[3],
        ).clamp(0.0, 1.0);

        self.current_frame = ((eased * total as f32) as u32).min(total.saturating_sub(1));

        if !self.loop_anim && raw_t >= 1.0 { self.playing = false; }
    }

    // ── Render ───────────────────────────────────────────────────────────────

    /// Advance the animation and draw the current frame.
    pub fn draw(&mut self) {
        self.advance();

        if self.frames.is_empty() {
            if self.bg_color.a > 0.0 {
                draw_rectangle(self.x, self.y, self.w, self.h, self.bg_color);
            }
            return;
        }

        let idx     = (self.current_frame as usize).min(self.frames.len() - 1);
        let texture = &self.frames[idx];
        let tint    = Color::new(1.0, 1.0, 1.0, self.opacity.clamp(0.0, 1.0));
        let tex_w   = texture.width();
        let tex_h   = texture.height();

        match self.object_fit {
            ObjectFit::Warp => {
                draw_texture_ex(
                    texture, self.x, self.y, tint,
                    DrawTextureParams { dest_size: Some(vec2(self.w, self.h)), rotation: self.rotation, flip_x: self.flip_x, ..Default::default() },
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
                        rotation: self.rotation,
                        flip_x: self.flip_x,
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
                    DrawTextureParams { dest_size: Some(vec2(dw, dh)), rotation: self.rotation, flip_x: self.flip_x, ..Default::default() },
                );
            }
        }
    }

    pub fn draw_ex(&mut self, _flipped_x: bool, _rotation: f32, pos: Vec2, size: Vec2) {
        self.x = pos.x; self.y = pos.y; self.w = size.x; self.h = size.y;
        self.draw();
    }
}

/// Draw the current animation frame. `frame_key` is the `texture_cache` key
/// for the pre-loaded frame (format: `"{src}/frame_{i}"`).
/// Falls back to a solid `bg_color` rectangle when the frame is not yet loaded.
pub fn draw(
    layout:        ResolvedLayout,
    frame_key:     &str,
    object_fit:    ObjectFit,
    opacity:       f32,
    texture_cache: &HashMap<String, Texture2D>,
    bg_color:      Color,
) {
    let tint = Color::new(1.0, 1.0, 1.0, opacity.clamp(0.0, 1.0));

    if let Some(texture) = texture_cache.get(frame_key) {
        let tex_w = texture.width();
        let tex_h = texture.height();
        match object_fit {
            ObjectFit::Warp => {
                draw_texture_ex(
                    texture, layout.x, layout.y, tint,
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
                    texture, layout.x, layout.y, tint,
                    DrawTextureParams {
                        dest_size: Some(vec2(layout.w, layout.h)),
                        source:    Some(Rect::new(src_x, src_y, vis_w, vis_h)),
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
                    texture, dx, dy, tint,
                    DrawTextureParams { dest_size: Some(vec2(dw, dh)), ..Default::default() },
                );
            }
        }
    } else if bg_color.a > 0.0 {
        draw_rectangle(layout.x, layout.y, layout.w, layout.h, bg_color);
    }
}
