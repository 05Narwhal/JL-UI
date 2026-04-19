// * =========== MODULE: GRADIENT =========== *
use macroquad::prelude::*;
use crate::modules::ui::types::ui::{GradientType, ResolvedLayout};

// ---- helpers ----------------------------------------------------------------

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn color_bytes(c: Color) -> [u8; 4] {
    [
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
        (c.a * 255.0).round() as u8,
    ]
}

// ---- public entry point -----------------------------------------------------

/// Draw a gradient filling `layout`.
///
/// `angle_deg` is in Cartesian degrees: 0° = right, 90° = up (CCW positive).
/// Only used for `GradientType::Line`; ignored for `GradientType::Radial`.
pub fn draw(
    layout: ResolvedLayout,
    gradient_type: GradientType,
    angle_deg: f32,
    color1: Color,
    color2: Color,
) {
    match gradient_type {
        GradientType::Linear   => draw_linear(layout, angle_deg, color1, color2),
        GradientType::Radial => draw_radial(layout, color1, color2),
    }
}

// ---- linear gradient --------------------------------------------------------
// Uses a single quad (4-vertex mesh) with per-vertex colors determined by
// projecting each corner onto the gradient axis.  The GPU interpolates
// exactly, so quality is perfect for any angle.

fn draw_linear(layout: ResolvedLayout, angle_deg: f32, color1: Color, color2: Color) {
    let (x, y, w, h) = (layout.x, layout.y, layout.w, layout.h);

    // Convert Cartesian angle to screen-space direction.
    // Cartesian: 0°=right (+X), 90°=up (+Y in math = decreasing screen-Y).
    let angle_rad = angle_deg.to_radians();
    let dir_x =  angle_rad.cos();
    let dir_y = -angle_rad.sin(); // flip Y: screen Y increases downward

    // Corner positions (local, relative to rect top-left): TL, TR, BR, BL
    let corners = [(0.0f32, 0.0f32), (w, 0.0), (w, h), (0.0, h)];

    // Project each corner onto the gradient axis (using rect-centre-relative coords).
    let (cx, cy) = (w / 2.0, h / 2.0);
    let projs: [f32; 4] = std::array::from_fn(|i| {
        (corners[i].0 - cx) * dir_x + (corners[i].1 - cy) * dir_y
    });

    let min_p = projs.iter().cloned().fold(f32::INFINITY,     f32::min);
    let max_p = projs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = max_p - min_p;

    let vertices: Vec<Vertex> = (0..4)
        .map(|i| {
            let t = if range > 0.0 { (projs[i] - min_p) / range } else { 0.0 };
            Vertex {
                position: Vec3::new(x + corners[i].0, y + corners[i].1, 0.0),
                uv: Vec2::ZERO,
                normal: Vec4::ZERO,
                color: color_bytes(lerp_color(color1, color2, t)),
            }
        })
        .collect();

    let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];
    draw_mesh(&Mesh { vertices, indices, texture: None });
}

// ---- radial gradient --------------------------------------------------------
// Uses a 16×16 subdivided grid mesh so the circular blend looks smooth inside
// a rectangular region.  color1 is at the centre, color2 at the edges.

fn draw_radial(layout: ResolvedLayout, color1: Color, color2: Color) {
    let (x, y, w, h) = (layout.x, layout.y, layout.w, layout.h);
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    // Normalise by half-diagonal so all four corners reach full color2.
    let max_dist = ((w / 2.0).powi(2) + (h / 2.0).powi(2)).sqrt().max(0.001);

    const GRID: usize = 16;
    let mut vertices: Vec<Vertex> = Vec::with_capacity((GRID + 1) * (GRID + 1));
    let mut indices: Vec<u16>     = Vec::with_capacity(GRID * GRID * 6);

    for row in 0..=(GRID) {
        for col in 0..=(GRID) {
            let fx = x + (col as f32 / GRID as f32) * w;
            let fy = y + (row as f32 / GRID as f32) * h;
            let dist = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            let t    = (dist / max_dist).clamp(0.0, 1.0);
            vertices.push(Vertex {
                position: Vec3::new(fx, fy, 0.0),
                uv: Vec2::ZERO,
                normal: Vec4::ZERO,
                color: color_bytes(lerp_color(color1, color2, t)),
            });
        }
    }

    let stride = (GRID + 1) as u16;
    for row in 0..(GRID as u16) {
        for col in 0..(GRID as u16) {
            let tl = row * stride + col;
            let tr = tl + 1;
            let bl = tl + stride;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, tr, br, tl, br, bl]);
        }
    }

    draw_mesh(&Mesh { vertices, indices, texture: None });
}

pub struct UIGradient {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub gradient_type: GradientType,
    /// Angle in Cartesian degrees (0° = right, 90° = up). Only used for `GradientType::Linear`.
    pub angle_deg: f32,
    pub color1: Color,
    pub color2: Color,
    pub visible: bool,
}

impl UIGradient {
    pub fn new(x: f32, y: f32, w: f32, h: f32, color1: Color, color2: Color) -> Self {
        Self { x, y, w, h, gradient_type: GradientType::Linear, angle_deg: 0.0, color1, color2, visible: true }
    }

    /// Shorthand constructor for a top-to-bottom linear gradient (90° up in Cartesian = downward in screen space).
    pub fn vertical(x: f32, y: f32, w: f32, h: f32, top: Color, bottom: Color) -> Self {
        Self { x, y, w, h, gradient_type: GradientType::Linear, angle_deg: 270.0, color1: top, color2: bottom, visible: true }
    }

    /// Shorthand for a left-to-right linear gradient (0°).
    pub fn horizontal(x: f32, y: f32, w: f32, h: f32, left: Color, right: Color) -> Self {
        Self { x, y, w, h, gradient_type: GradientType::Linear, angle_deg: 0.0, color1: left, color2: right, visible: true }
    }

    /// Shorthand for a radial gradient.
    pub fn radial(x: f32, y: f32, w: f32, h: f32, center: Color, edge: Color) -> Self {
        Self { x, y, w, h, gradient_type: GradientType::Radial, angle_deg: 0.0, color1: center, color2: edge, visible: true }
    }

    pub fn set_pos(&mut self, x: f32, y: f32)  { self.x = x; self.y = y; }
    pub fn set_size(&mut self, w: f32, h: f32) { self.w = w; self.h = h; }
    pub fn set_colors(&mut self, c1: Color, c2: Color) { self.color1 = c1; self.color2 = c2; }
    pub fn set_angle(&mut self, deg: f32) { self.angle_deg = deg; }

    pub fn draw(&self) {
        if !self.visible { return; }
        let layout = ResolvedLayout { x: self.x, y: self.y, w: self.w, h: self.h, ..Default::default() };
        draw(layout, self.gradient_type, self.angle_deg, self.color1, self.color2);
    }

    pub fn draw_ex(&self, _flipped_x: bool, _rotation: f32, pos: Vec2, size: Vec2) {
        crate::ui_trace!("UIGradient::draw_ex is not fully implemented; ignoring flipped_x and rotation");
        if !self.visible { return; }
        let layout = ResolvedLayout { x: pos.x, y: pos.y, w: size.x, h: size.y, ..Default::default() };
        draw(layout, self.gradient_type, self.angle_deg, self.color1, self.color2);
    }
}
