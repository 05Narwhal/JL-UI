// * =========== MODULE: UI -- MAIN LIB =========== *
// * ============ IMPORTS =========== *
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use macroquad::prelude::*;

use super::parser::{GlobalVarStore, UIParser, VarStore, XmlVariableValue};
use super::renderer::UIRenderer;
use super::types::ui::{
    AlignX, AlignY, Physics2DRenderCallback, Physics3DRenderCallback, PosValue,
    ResolvedLayout, SizeExpr, SizeValue, UIComponent,
};

#[cfg(feature = "debug")]
macro_rules! info {
    ($($arg:tt)*) => { log::trace!($($arg)*) };
}

#[cfg(feature = "debug")]
macro_rules! error {
    ($($arg:tt)*) => { log::error!($($arg)*) };
}

#[cfg(not(feature = "debug"))]
macro_rules! info {
    ($($arg:tt)*) => { () };
}

#[cfg(not(feature = "debug"))]
macro_rules! error {
    ($($arg:tt)*) => { () };
}

// * ============ PARAM INTROSPECTION HELPERS =========== *
fn color_to_hex(c: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
        (c.a * 255.0).round() as u8
    )
}

fn size_expr_to_string(expr: &SizeExpr) -> String {
    match expr {
        SizeExpr::Px(v) => format!("{}px", v),
        SizeExpr::NegPx(v) => format!("-{}px", v),
        SizeExpr::Pct(v) => format!("{}%", v),
        SizeExpr::Vw(v) => format!("{}vw", v),
        SizeExpr::Vh(v) => format!("{}vh", v),
        SizeExpr::Deg(v) => format!("{}deg", v),
        SizeExpr::BinOp(l, op, r) => {
            let op_s = match op {
                super::types::ui::SizeOp::Add => "+",
                super::types::ui::SizeOp::Sub => "-",
                super::types::ui::SizeOp::Mul => "*",
                super::types::ui::SizeOp::Div => "/",
            };
            format!("({} {} {})", size_expr_to_string(l), op_s, size_expr_to_string(r))
        }
    }
}

fn size_value_to_string(v: &SizeValue) -> String {
    match v {
        SizeValue::Auto => "auto".to_string(),
        SizeValue::Expr(expr) => size_expr_to_string(expr),
    }
}

fn pos_value_to_string(v: &PosValue) -> String {
    match v {
        PosValue::Px(expr) => size_expr_to_string(expr),
        PosValue::Auto(expr) => format!("$ + {}", size_expr_to_string(expr)),
    }
}

fn align_x_to_string(v: AlignX) -> &'static str {
    match v {
        AlignX::Left => "left",
        AlignX::Center => "center",
        AlignX::Right => "right",
    }
}

fn align_y_to_string(v: AlignY) -> &'static str {
    match v {
        AlignY::Top => "top",
        AlignY::Center => "center",
        AlignY::Bottom => "bottom",
    }
}

#[derive(Clone, Debug)]
pub struct ElementParams {
    pub element: String,
    pub id: String,
    pub class_name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub scale: f32,
    pub font_size: String,
    pub z_order: f32,
    pub visible: bool,
    pub usr_interact: bool,
    pub bg_color: Color,
    pub transition: String,
    pub transition_timing: String,
    pub values: BTreeMap<String, String>,
}

impl ElementParams {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|v| v.as_str())
    }
}

fn component_params(component: &UIComponent, layout: Option<ResolvedLayout>) -> ElementParams {
    let p = component.params();
    let mut out = BTreeMap::new();

    let (computed_x, computed_y, computed_width, computed_height) = match layout {
        Some(l) => (l.x, l.y, l.w, l.h),
        None => (0.0, 0.0, 0.0, 0.0),
    };

    let transition_timing = match p.transition.easing {
        super::types::ui::TransitionEasing::Ease => "ease".to_string(),
        super::types::ui::TransitionEasing::EaseIn => "ease-in".to_string(),
        super::types::ui::TransitionEasing::EaseOut => "ease-out".to_string(),
        super::types::ui::TransitionEasing::EaseInOut => "ease-in-out".to_string(),
        super::types::ui::TransitionEasing::Linear => "linear".to_string(),
        super::types::ui::TransitionEasing::CubicBezier([x1, y1, x2, y2]) => {
            format!("cubic-bezier({},{},{},{})", x1, y1, x2, y2)
        }
    };

    out.insert("id".to_string(), p.id.clone());
    out.insert("className".to_string(), p.class_name.clone());
    out.insert("x".to_string(), computed_x.to_string());
    out.insert("y".to_string(), computed_y.to_string());
    out.insert("width".to_string(), computed_width.to_string());
    out.insert("height".to_string(), computed_height.to_string());
    out.insert("xRaw".to_string(), pos_value_to_string(&p.x));
    out.insert("yRaw".to_string(), pos_value_to_string(&p.y));
    out.insert("widthRaw".to_string(), size_value_to_string(&p.width));
    out.insert("heightRaw".to_string(), size_value_to_string(&p.height));
    out.insert("rotation".to_string(), p.rotation.to_string());
    out.insert("scale".to_string(), p.scale.to_string());
    out.insert("fontSize".to_string(), p.font_size.to_string());
    out.insert("zOrder".to_string(), p.z_order.to_string());
    out.insert("visible".to_string(), p.visible.to_string());
    out.insert("usrInteract".to_string(), p.usr_interact.to_string());
    out.insert("bgColor".to_string(), color_to_hex(p.bg_color));
    out.insert("transition".to_string(), format!("{}s", p.transition.duration));
    out.insert("transitionTiming".to_string(), transition_timing.clone());

    match component {
        UIComponent::View { align_x, align_y, .. }
        | UIComponent::ScrollView { align_x, align_y, .. }
        | UIComponent::Button { align_x, align_y, .. } => {
            out.insert("alignX".to_string(), align_x_to_string(*align_x).to_string());
            out.insert("alignY".to_string(), align_y_to_string(*align_y).to_string());
        }
        _ => {}
    }

    match component {
        UIComponent::Button { text, color, hovered, pressed, .. } => {
            out.insert("element".to_string(), "button".to_string());
            out.insert("text".to_string(), text.clone());
            out.insert("color".to_string(), color_to_hex(*color));
            out.insert("hovered".to_string(), hovered.to_string());
            out.insert("pressed".to_string(), pressed.to_string());
        }
        UIComponent::Slider { min, max, value, color, track_color, dragging, changed, .. } => {
            out.insert("element".to_string(), "slider".to_string());
            out.insert("min".to_string(), min.to_string());
            out.insert("max".to_string(), max.to_string());
            out.insert("value".to_string(), value.to_string());
            out.insert("color".to_string(), color_to_hex(*color));
            out.insert("trackColor".to_string(), color_to_hex(*track_color));
            out.insert("dragging".to_string(), dragging.to_string());
            out.insert("changed".to_string(), changed.to_string());
        }
        UIComponent::Switch { checked, color, .. } => {
            out.insert("element".to_string(), "switch".to_string());
            out.insert("checked".to_string(), checked.to_string());
            out.insert("color".to_string(), color_to_hex(*color));
        }
        UIComponent::Label { text, color, .. } => {
            out.insert("element".to_string(), "label".to_string());
            out.insert("text".to_string(), text.clone());
            out.insert("color".to_string(), color_to_hex(*color));
        }
        UIComponent::TextField { placeholder, text, color, focused, .. } => {
            out.insert("element".to_string(), "text_field".to_string());
            out.insert("placeholder".to_string(), placeholder.clone());
            out.insert("text".to_string(), text.clone());
            out.insert("color".to_string(), color_to_hex(*color));
            out.insert("focused".to_string(), focused.to_string());
        }
        UIComponent::Texture { src, object_fit, .. } => {
            out.insert("element".to_string(), "texture".to_string());
            out.insert("src".to_string(), src.clone());
            out.insert(
                "object-fit".to_string(),
                match object_fit {
                    super::types::ui::ObjectFit::Warp => "warp",
                    super::types::ui::ObjectFit::Cover => "cover",
                    super::types::ui::ObjectFit::Contain => "contain",
                }
                .to_string(),
            );
        }
        UIComponent::AnimatedTexture { src, fps, loop_anim, autoplay, opacity, object_fit, frame_count, timing, playing, paused, current_frame, elapsed, .. } => {
            out.insert("element".to_string(), "animated_texture".to_string());
            out.insert("src".to_string(), src.clone());
            out.insert("fps".to_string(), fps.to_string());
            out.insert("loop".to_string(), loop_anim.to_string());
            out.insert("autoplay".to_string(), autoplay.to_string());
            out.insert("opacity".to_string(), opacity.to_string());
            out.insert(
                "object-fit".to_string(),
                match object_fit {
                    super::types::ui::ObjectFit::Warp => "warp",
                    super::types::ui::ObjectFit::Cover => "cover",
                    super::types::ui::ObjectFit::Contain => "contain",
                }
                .to_string(),
            );
            out.insert("frameCount".to_string(), frame_count.to_string());
            out.insert("timing_function".to_string(), format!("{},{},{},{}", timing[0], timing[1], timing[2], timing[3]));
            out.insert("playing".to_string(), playing.to_string());
            out.insert("paused".to_string(), paused.to_string());
            out.insert("currentFrame".to_string(), current_frame.to_string());
            out.insert("elapsed".to_string(), elapsed.to_string());
        }
        UIComponent::PhysicsContainer2D { engine, .. } => {
            out.insert("element".to_string(), "physics-container-2d".to_string());
            out.insert(
                "engine".to_string(),
                match engine {
                    super::types::ui::Physics2DEngine::Macroquad => "macroquad",
                    super::types::ui::Physics2DEngine::Bevy => "bevy",
                }
                .to_string(),
            );
        }
        UIComponent::PhysicsContainer3D { .. } => {
            out.insert("element".to_string(), "physics-container-3d".to_string());
            out.insert("engine".to_string(), "bevy".to_string());
        }
        UIComponent::View { .. } => {
            out.insert("element".to_string(), "view".to_string());
        }
        UIComponent::ScrollView { scroll_x, scroll_y, .. } => {
            out.insert("element".to_string(), "scroll_view".to_string());
            out.insert("scrollX".to_string(), scroll_x.to_string());
            out.insert("scrollY".to_string(), scroll_y.to_string());
        }
        UIComponent::ProgressBar { value, max, color, .. } => {
            out.insert("element".to_string(), "progress_bar".to_string());
            out.insert("value".to_string(), value.to_string());
            out.insert("max".to_string(), max.to_string());
            out.insert("color".to_string(), color_to_hex(*color));
        }
        UIComponent::Rect { border_color, border_width, corner_radius, .. } => {
            out.insert("element".to_string(), "rect".to_string());
            out.insert("borderColor".to_string(), color_to_hex(*border_color));
            out.insert("borderWidth".to_string(), border_width.to_string());
            out.insert("cornerRadius".to_string(), corner_radius.to_string());
        }
        UIComponent::Gradient { gradient_type, angle, color1, color2, .. } => {
            out.insert("element".to_string(), "gradient".to_string());
            out.insert(
                "type".to_string(),
                match gradient_type {
                    super::types::ui::GradientType::Linear => "line",
                    super::types::ui::GradientType::Radial => "radial",
                }
                .to_string(),
            );
            out.insert("angle".to_string(), format!("{}deg", angle));
            out.insert("color1".to_string(), color_to_hex(*color1));
            out.insert("color2".to_string(), color_to_hex(*color2));
        }
    }

    let element = out
        .get("element")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    ElementParams {
        element,
        id: p.id.clone(),
        class_name: p.class_name.clone(),
        x: computed_x,
        y: computed_y,
        width: computed_width,
        height: computed_height,
        rotation: p.rotation,
        scale: p.scale,
        font_size: p.font_size.to_string(),
        z_order: p.z_order as f32,
        visible: p.visible,
        usr_interact: p.usr_interact,
        bg_color: p.bg_color,
        transition: format!("{}s", p.transition.duration),
        transition_timing,
        values: out,
    }
}

// * ============ ANIM CONF =========== *
// Deserialized from conf.jsonc files found alongside animated texture folders.
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

// * ============ ORIGINAL COMPONENT DATA =========== *
// Snapshot of resetable fields captured at load time, keyed by component ID.
#[derive(Clone, Debug)]
pub struct OriginalData {
    pub bg_color: Color,
    pub color:    Option<Color>,  // text/fill colour; None for components without one
    pub scale:    f32,
}

/// Collect original data from every component that has an id.
fn snapshot_originals(components: &[UIComponent], out: &mut HashMap<String, OriginalData>) {
    for c in components {
        let id = c.params().id.clone();
        if !id.is_empty() {
            let color = match c {
                UIComponent::Button  { color, .. } => Some(*color),
                UIComponent::Label   { color, .. } => Some(*color),
                UIComponent::Slider  { color, .. } => Some(*color),
                UIComponent::Switch  { color, .. } => Some(*color),
                UIComponent::TextField { color, .. } => Some(*color),
                UIComponent::ProgressBar { color, .. } => Some(*color),
                _ => None,
            };
            out.insert(id, OriginalData {
                bg_color: c.params().bg_color,
                color,
                scale: c.params().scale,
            });
        }
        snapshot_originals(c.children(), out);
    }
}

// * ============ MUTABLE UI HANDLE =========== *
// Wraps a &mut UIComponent with its resolved layout and the originals map.
// Returned by both get_by_id() and get_by_id_mut() — handles all read + write operations.
#[derive(Debug)]
pub struct UIHandleMut<'a> {
    component: &'a mut UIComponent,
    layout:    Option<ResolvedLayout>,
    originals: &'a HashMap<String, OriginalData>,
}

impl<'a> UIHandleMut<'a> {
    pub fn new(
        component: &'a mut UIComponent,
        layout:    Option<ResolvedLayout>,
        originals: &'a HashMap<String, OriginalData>,
    ) -> Self {
        Self { component, layout, originals }
    }

    // * -- Hit-test helpers -- *
    fn in_bounds(&self) -> bool {
        let p = self.component.params();
        if !p.usr_interact || !p.visible { return false; }
        let (mx, my) = mouse_position();
        if let Some(l) = self.layout {
            return mx >= l.x && mx <= l.x + l.w && my >= l.y && my <= l.y + l.h;
        }
        false
    }

    pub fn on_hover(&self)   -> bool { self.in_bounds() }
    pub fn on_click(&self)   -> bool { self.in_bounds() && is_mouse_button_pressed(MouseButton::Left) }
    pub fn on_release(&self) -> bool { self.in_bounds() && is_mouse_button_released(MouseButton::Left) }
    pub fn on_held(&self)    -> bool { self.in_bounds() && is_mouse_button_down(MouseButton::Left) }

    /// Fire `callback` with the current slider value if it changed this frame.
    pub fn on_change<F: FnOnce(f32)>(&self, callback: F) {
        if let UIComponent::Slider { value, changed, .. } = &*self.component {
            if *changed { callback(*value); }
        }
    }

    // * -- Read common attributes -- *
    pub fn id(&self)         -> &str  { &self.component.params().id }
    pub fn class_name(&self) -> &str  { &self.component.params().class_name }
    pub fn visible(&self)    -> bool  { self.component.params().visible }
    pub fn layout(&self)     -> Option<ResolvedLayout> { self.layout }

    /// Returns a complete parameter map for this element:
    /// common params + element-specific params + runtime flags.
    /// Values include parser/default fallbacks when not explicitly set in XML.
    pub fn params(&self) -> ElementParams {
        component_params(self.component, self.layout)
    }

    /// Return the display text for Label, Button, or TextField components.
    pub fn get_text(&self) -> String {
        match &*self.component {
            UIComponent::Label     { text, .. } => text.clone(),
            UIComponent::Button    { text, .. } => text.clone(),
            UIComponent::TextField { text, .. } => text.clone(),
            _ => String::new(),
        }
    }

    // * -- Component-specific state readers -- *
    pub fn slider_value(&self) -> Option<f32> {
        match &*self.component { UIComponent::Slider { value, .. } => Some(*value), _ => None }
    }
    pub fn is_checked(&self) -> Option<bool> {
        match &*self.component { UIComponent::Switch { checked, .. } => Some(*checked), _ => None }
    }
    pub fn text(&self) -> Option<&str> {
        match &*self.component {
            UIComponent::TextField { text, .. } => Some(text),
            UIComponent::Label     { text, .. } => Some(text),
            UIComponent::Button    { text, .. } => Some(text),
            _ => None,
        }
    }
    pub fn is_hovered(&self) -> Option<bool> {
        match &*self.component { UIComponent::Button { hovered, .. } => Some(*hovered), _ => None }
    }
    pub fn is_pressed(&self) -> Option<bool> {
        match &*self.component { UIComponent::Button { pressed, .. } => Some(*pressed), _ => None }
    }
    pub fn is_focused(&self) -> Option<bool> {
        match &*self.component { UIComponent::TextField { focused, .. } => Some(*focused), _ => None }
    }
    pub fn progress(&self) -> Option<f32> {
        match &*self.component {
            UIComponent::ProgressBar { value, max, .. } => {
                if *max > 0.0 { Some(*value / *max) } else { Some(0.0) }
            }
            _ => None,
        }
    }

    pub fn component(&self)     -> &UIComponent     { self.component }
    pub fn component_mut(&mut self) -> &mut UIComponent { self.component }

    // * -- Mutation helpers -- *
    pub fn set_visible(&mut self, v: bool)     { self.component.params_mut().visible = v; }
    pub fn set_x(&mut self, x: f32)           { self.component.params_mut().x = PosValue::Px(SizeExpr::Px(x)); }
    pub fn set_y(&mut self, y: f32)           { self.component.params_mut().y = PosValue::Px(SizeExpr::Px(y)); }
    pub fn set_width(&mut self, w: f32)       { self.component.params_mut().width  = SizeValue::Expr(SizeExpr::Px(w)); }
    pub fn set_height(&mut self, h: f32)      { self.component.params_mut().height = SizeValue::Expr(SizeExpr::Px(h)); }
    pub fn set_bg_color(&mut self, c: Color)  { self.component.params_mut().set_bg_color_with_transition(c); }

    /// Uniformly scale the component (applied at render time as a centred resize).
    pub fn scale(&mut self, s: f32) { self.component.params_mut().set_scale_with_transition(s); }

    /// Set the foreground/text colour of the component.
    pub fn set_color(&mut self, color: Color) {
        self.component.set_color_with_transition(color);
    }

    /// Replace the display text of Label, Button, or TextField.
    pub fn modify_text(&mut self, t: &str) {
        match self.component {
            UIComponent::Label     { text, .. } => *text = t.to_string(),
            UIComponent::Button    { text, .. } => *text = t.to_string(),
            UIComponent::TextField { text, .. } => *text = t.to_string(),
            _ => {}
        }
    }

    /// Reset a named parameter to its original XML-parsed value.
    /// Supported keys: `"bgColor"`, `"color"`, `"scale"`.
    pub fn reset_parameter(&mut self, key: &str) {
        let id = self.component.params().id.clone();
        if let Some(orig) = self.originals.get(&id) {
            match key {
                "bgColor" => { self.component.params_mut().set_bg_color_with_transition(orig.bg_color); }
                "color" => {
                    if let Some(c) = orig.color {
                        self.set_color(c);
                    }
                }
                "scale" => { self.component.params_mut().set_scale_with_transition(orig.scale); }
                _ => {}
            }
        }
    }

    // * -- Slider / Switch / ProgressBar setters -- *
    pub fn set_slider_value(&mut self, v: f32) {
        if let UIComponent::Slider { value, min, max, .. } = self.component {
            *value = v.clamp(*min, *max);
        }
    }
    pub fn toggle_switch(&mut self) {
        if let UIComponent::Switch { checked, .. } = self.component { *checked = !*checked; }
    }
    pub fn set_checked(&mut self, v: bool) {
        if let UIComponent::Switch { checked, .. } = self.component { *checked = v; }
    }
    pub fn set_text(&mut self, t: String) { self.modify_text(&t); }
    pub fn set_progress(&mut self, v: f32) {
        if let UIComponent::ProgressBar { value, max, .. } = self.component {
            *value = v.clamp(0.0, *max);
        }
    }

    // * -- Animated Texture controls -- *

    /// Start or resume the animation.
    pub fn play(&mut self) {
        if let UIComponent::AnimatedTexture { playing, paused, .. } = self.component {
            *playing = true;
            *paused  = false;
        }
    }

    /// Pause the animation, preserving the current frame for resume.
    pub fn pause(&mut self) {
        if let UIComponent::AnimatedTexture { playing, paused, .. } = self.component {
            *playing = false;
            *paused  = true;
        }
    }

    /// Stop the animation and reset to frame 0.
    pub fn stop(&mut self) {
        if let UIComponent::AnimatedTexture { playing, paused, elapsed, current_frame, .. } = self.component {
            *playing       = false;
            *paused        = false;
            *elapsed       = 0.0;
            *current_frame = 0;
        }
    }

    /// Set draw opacity (0.0 = invisible, 1.0 = fully opaque).
    pub fn set_opacity(&mut self, opacity: f32) {
        if let UIComponent::AnimatedTexture { opacity: op, .. } = self.component {
            *op = opacity.clamp(0.0, 1.0);
        }
    }

    /// Get draw opacity, or `None` if the component is not an AnimatedTexture.
    pub fn get_opacity(&self) -> Option<f32> {
        match &*self.component {
            UIComponent::AnimatedTexture { opacity, .. } => Some(*opacity),
            _ => None,
        }
    }

    /// Set playback speed in frames per second.
    pub fn set_fps(&mut self, fps: f32) {
        if let UIComponent::AnimatedTexture { fps: f, .. } = self.component {
            *f = fps.max(0.0);
        }
    }

    /// Get playback speed, or `None` if not an AnimatedTexture.
    pub fn get_fps(&self) -> Option<f32> {
        match &*self.component {
            UIComponent::AnimatedTexture { fps, .. } => Some(*fps),
            _ => None,
        }
    }

    /// Enable or disable looping.
    pub fn set_loop(&mut self, loop_v: bool) {
        if let UIComponent::AnimatedTexture { loop_anim, .. } = self.component {
            *loop_anim = loop_v;
        }
    }

    /// Whether the animation is currently playing.
    pub fn is_playing(&self) -> bool {
        match &*self.component {
            UIComponent::AnimatedTexture { playing, .. } => *playing,
            _ => false,
        }
    }

    /// Whether the animation is paused (not stopped).
    pub fn is_paused(&self) -> bool {
        match &*self.component {
            UIComponent::AnimatedTexture { paused, .. } => *paused,
            _ => false,
        }
    }

    /// Get the current frame index, or `None` if not an AnimatedTexture.
    pub fn get_current_frame(&self) -> Option<u32> {
        match &*self.component {
            UIComponent::AnimatedTexture { current_frame, .. } => Some(*current_frame),
            _ => None,
        }
    }

    /// Seek to a specific frame index (clamped to valid range).
    /// Also updates `elapsed` so timing stays consistent after the seek.
    pub fn set_frame(&mut self, frame: u32) {
        if let UIComponent::AnimatedTexture { current_frame, frame_count, elapsed, fps, .. } = self.component {
            *current_frame = frame.min(frame_count.saturating_sub(1));
            if *fps > 0.0 {
                *elapsed = *current_frame as f32 / *fps;
            }
        }
    }

    /// Replace the animation source folder at runtime and reset playback.
    pub fn set_src(&mut self, src: &str) {
        if let UIComponent::AnimatedTexture { src: s, playing, paused, elapsed, current_frame, frame_count, .. } = self.component {
            *s             = src.to_string();
            *playing       = false;
            *paused        = false;
            *elapsed       = 0.0;
            *current_frame = 0;
            *frame_count   = 0;
        }
    }
}

// * ============ IMMUTABLE UI HANDLE =========== *
// Kept for read-only query paths (get_by_class, get_all_by_class).
#[derive(Clone, Debug)]
pub struct UIHandle<'a> {
    component: &'a UIComponent,
    layout:    Option<ResolvedLayout>,
}

impl<'a> UIHandle<'a> {
    pub fn new(component: &'a UIComponent, layout: Option<ResolvedLayout>) -> Self {
        Self { component, layout }
    }

    fn in_bounds(&self) -> bool {
        let p = self.component.params();
        if !p.usr_interact || !p.visible { return false; }
        let (mx, my) = mouse_position();
        if let Some(l) = self.layout {
            return mx >= l.x && mx <= l.x + l.w && my >= l.y && my <= l.y + l.h;
        }
        false
    }

    pub fn on_hover(&self)   -> bool { self.in_bounds() }
    pub fn on_click(&self)   -> bool { self.in_bounds() && is_mouse_button_pressed(MouseButton::Left) }
    pub fn on_release(&self) -> bool { self.in_bounds() && is_mouse_button_released(MouseButton::Left) }
    pub fn on_held(&self)    -> bool { self.in_bounds() && is_mouse_button_down(MouseButton::Left) }

    pub fn id(&self)         -> &str  { &self.component.params().id }
    pub fn class_name(&self) -> &str  { &self.component.params().class_name }
    pub fn visible(&self)    -> bool  { self.component.params().visible }
    pub fn layout(&self)     -> Option<ResolvedLayout> { self.layout }
    pub fn component(&self)  -> &UIComponent { self.component }

    /// Returns a complete parameter map for this element:
    /// common params + element-specific params + runtime flags.
    /// Values include parser/default fallbacks when not explicitly set in XML.
    pub fn params(&self) -> ElementParams {
        component_params(self.component, self.layout)
    }

    pub fn slider_value(&self) -> Option<f32> {
        match self.component { UIComponent::Slider { value, .. } => Some(*value), _ => None }
    }
    pub fn is_checked(&self) -> Option<bool> {
        match self.component { UIComponent::Switch { checked, .. } => Some(*checked), _ => None }
    }
    pub fn text(&self) -> Option<&str> {
        match self.component {
            UIComponent::TextField { text, .. } => Some(text),
            UIComponent::Label     { text, .. } => Some(text),
            UIComponent::Button    { text, .. } => Some(text),
            _ => None,
        }
    }
    pub fn is_hovered(&self) -> Option<bool> {
        match self.component { UIComponent::Button { hovered, .. } => Some(*hovered), _ => None }
    }
    pub fn is_pressed(&self) -> Option<bool> {
        match self.component { UIComponent::Button { pressed, .. } => Some(*pressed), _ => None }
    }
    pub fn is_focused(&self) -> Option<bool> {
        match self.component { UIComponent::TextField { focused, .. } => Some(*focused), _ => None }
    }
    pub fn progress(&self) -> Option<f32> {
        match self.component {
            UIComponent::ProgressBar { value, max, .. } => {
                if *max > 0.0 { Some(*value / *max) } else { Some(0.0) }
            }
            _ => None,
        }
    }
}

// * ============ MAIN UI STRUCT =========== *
#[derive(Clone, Debug)]
pub struct LoadedSchema {
    pub schema_id: String,
    pub filename: String,
    pub path: PathBuf,
    pub components: Vec<UIComponent>,
    pub originals: HashMap<String, OriginalData>,
    pub variables: VarStore,
}

#[derive(Clone, Debug)]
struct SchemaVarSpec {
    initial: XmlVariableValue,
    reset: XmlVariableValue,
}

pub struct UI {
    components:    Vec<UIComponent>,
    layout_cache:  HashMap<String, ResolvedLayout>,
    params_store:  HashMap<String, ElementParams>,
    click_cache:   HashSet<String>,
    release_cache: HashSet<String>,
    /// Original XML-parsed values, used by reset_parameter().
    originals:     HashMap<String, OriginalData>,
    /// Textures loaded by load_textures(), keyed by the src string from the XML.
    texture_cache: HashMap<String, Texture2D>,
    /// All schemas / routes to be used by the app
    schemas: Vec<LoadedSchema>,
    /// The schema that is in use currently
    schema_history: Vec<String>,
    /// Local vars/consts for the currently active schema.
    active_local_vars: Option<VarStore>,
    /// Global vars/consts (global="true") shared across all loaded XML files.
    globals: GlobalVarStore,
    /// Schema-variable declarations (name -> initial/reset behavior).
    schema_var_specs: HashMap<String, SchemaVarSpec>,
    /// Per-schema runtime values for declared schema vars.
    schema_var_values: HashMap<String, HashMap<String, XmlVariableValue>>,
    /// Registered 2D physics callbacks by component id.
    physics_2d_by_id: HashMap<String, Physics2DRenderCallback>,
    /// Registered 2D physics callbacks by class token.
    physics_2d_by_class: HashMap<String, Physics2DRenderCallback>,
    /// Registered 3D physics callbacks by component id.
    physics_3d_by_id: HashMap<String, Physics3DRenderCallback>,
    /// Registered 3D physics callbacks by class token.
    physics_3d_by_class: HashMap<String, Physics3DRenderCallback>,
}

impl UI {
    fn class_matches(class_name: &str, query: &str) -> bool {
        class_name
            .split(|ch: char| ch.is_whitespace() || ch == ',')
            .any(|token| !token.is_empty() && token == query)
    }

    pub fn init() -> Self {
        Self {
            components:   Vec::new(),
            layout_cache: HashMap::new(),
            params_store: HashMap::new(),
            click_cache:  HashSet::new(),
            release_cache: HashSet::new(),
            originals:    HashMap::new(),
            texture_cache: HashMap::new(),
            schemas: Vec::new(),
            schema_history: Vec::new(),
            active_local_vars: None,
            globals: GlobalVarStore::new(),
            schema_var_specs: HashMap::new(),
            schema_var_values: HashMap::new(),
            physics_2d_by_id: HashMap::new(),
            physics_2d_by_class: HashMap::new(),
            physics_3d_by_id: HashMap::new(),
            physics_3d_by_class: HashMap::new(),
        }
    }

    /// Register a callback for a `physics-container-2d` element by id.
    pub fn register_physics_2d_by_id(&mut self, id: &str, callback: Physics2DRenderCallback) {
        self.physics_2d_by_id.insert(id.to_string(), callback);
    }

    /// Register a callback for a `physics-container-2d` element by class token.
    pub fn register_physics_2d_by_class(&mut self, class_name: &str, callback: Physics2DRenderCallback) {
        self.physics_2d_by_class.insert(class_name.to_string(), callback);
    }

    /// Register a callback for a `physics-container-3d` element by id.
    pub fn register_physics_3d_by_id(&mut self, id: &str, callback: Physics3DRenderCallback) {
        self.physics_3d_by_id.insert(id.to_string(), callback);
    }

    /// Register a callback for a `physics-container-3d` element by class token.
    pub fn register_physics_3d_by_class(&mut self, class_name: &str, callback: Physics3DRenderCallback) {
        self.physics_3d_by_class.insert(class_name.to_string(), callback);
    }

    /// Remove a previously-registered 2D physics callback by id.
    pub fn unregister_physics_2d_by_id(&mut self, id: &str) {
        self.physics_2d_by_id.remove(id);
    }

    /// Remove a previously-registered 2D physics callback by class token.
    pub fn unregister_physics_2d_by_class(&mut self, class_name: &str) {
        self.physics_2d_by_class.remove(class_name);
    }

    /// Remove a previously-registered 3D physics callback by id.
    pub fn unregister_physics_3d_by_id(&mut self, id: &str) {
        self.physics_3d_by_id.remove(id);
    }

    /// Remove a previously-registered 3D physics callback by class token.
    pub fn unregister_physics_3d_by_class(&mut self, class_name: &str) {
        self.physics_3d_by_class.remove(class_name);
    }

    fn initialize_schema_vars_for_schema(&mut self, schema_id: &str) {
        let entry = self
            .schema_var_values
            .entry(schema_id.to_string())
            .or_default();

        for (name, spec) in &self.schema_var_specs {
            entry.entry(name.clone()).or_insert_with(|| spec.initial.clone());
        }
    }

    fn reset_schema_vars_for_schema(&mut self, schema_id: &str) {
        let entry = self
            .schema_var_values
            .entry(schema_id.to_string())
            .or_default();

        for (name, spec) in &self.schema_var_specs {
            entry.insert(name.clone(), spec.reset.clone());
        }
    }

    fn normalize_schema_id(id: &str) -> String {
        id.replace('\\', "/")
    }

    fn has_schema_id(&self, schema_id: &str) -> bool {
        self.schemas.iter().any(|s| s.schema_id == schema_id)
    }

    fn relative_schema_id(base_folder: &PathBuf, file_path: &PathBuf) -> String {
        match file_path.strip_prefix(base_folder) {
            Ok(rel) => Self::normalize_schema_id(rel.to_string_lossy().as_ref()),
            Err(_) => Self::normalize_schema_id(file_path.file_name().unwrap_or_default().to_string_lossy().as_ref()),
        }
    }

    fn collect_schema_files_recursive(folder: &PathBuf, out: &mut Vec<PathBuf>) {
        let entries = match fs::read_dir(folder) {
            Ok(entries) => entries,
            Err(e) => {
                error!("[UI] Failed to read schema folder '{}': {}", folder.to_string_lossy(), e);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::collect_schema_files_recursive(&path, out);
            } else if path
                .extension()
                .map(|ext| ext.to_string_lossy().eq_ignore_ascii_case("xml"))
                .unwrap_or(false)
            {
                out.push(path);
            }
        }
    }

    fn load_schema_with_id(&mut self, path: &PathBuf, schema_id: &str) {
        let schema_id = Self::normalize_schema_id(schema_id);

        if self.has_schema_id(&schema_id) {
            error!(
                "[UI] Duplicate schema id '{}' is not allowed. There cannot be 2 schemas with the same id.",
                schema_id
            );
            return;
        }

        let path_str = path.to_str().unwrap_or_default();
        info!("[UI] Loading schema '{}' from: '{}'", schema_id, path_str);

        match UIParser::load_with_globals(path_str, &mut self.globals) {
            Ok((components, variables)) => {
                let mut schema = LoadedSchema {
                    schema_id: schema_id.clone(),
                    filename: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    path: path.clone(),
                    components: components.clone(),
                    originals: HashMap::new(),
                    variables,
                };

                info!("[UI] Schema loaded — {} root component(s)", components.len());
                schema.originals.clear();
                snapshot_originals(&components, &mut schema.originals);
                schema.components = components;

                self.schemas.push(schema);
                self.initialize_schema_vars_for_schema(&schema_id);
                self.rebuild_params_store();
            }
            Err(e) => {
                error!("[UI] Failed to load schema '{}': {}", path_str, e);
            }
        }
    }

    // * -- Load a UI layout from an XML file with explicit schema id -- *
    pub fn load_single_schema(&mut self, path: &PathBuf, unique_id: &str) {
        let schema_id = Self::normalize_schema_id(unique_id);
        if schema_id.trim().is_empty() {
            error!("[UI] load_single_schema requires a non-empty unique schema id");
            return;
        }
        self.load_schema_with_id(path, &schema_id);
    }

    // * -- Load multiple schemas from folders automatically -- * //
    pub fn load_schemas(&mut self, folder: PathBuf) {
        self.schemas.clear();
        self.globals = GlobalVarStore::new();
        self.schema_history.clear();
        self.active_local_vars = None;
        self.schema_var_values.clear();

        let mut paths = Vec::new();
        Self::collect_schema_files_recursive(&folder, &mut paths);

        // Schema ids are the relative path to the provided UI folder,
        // e.g. "main_menu.xml" or "menus/pause_menu.xml".

        // Guard against duplicate derived ids before loading.
        let mut seen_ids = HashSet::new();
        for path in &paths {
            let id = Self::relative_schema_id(&folder, path);
            if !seen_ids.insert(id.clone()) {
                error!(
                    "[UI] Duplicate schema id '{}' is not allowed. There cannot be 2 schemas with the same id.",
                    id
                );
                return;
            }
        }

        // Pass 1: pre-scan every file's <def> block and register all global="true"
        // entries. This means every file gets all globals regardless of load order.
        for path in &paths {
            if let Some(path_str) = path.to_str() {
                UIParser::collect_globals_from_file(path_str, &mut self.globals);
            }
        }

        // Pass 2: full parse of each file, globals already fully populated.
        for path in paths {
            let id = Self::relative_schema_id(&folder, &path);
            self.load_schema_with_id(&path, &id);
        }
    }

    pub fn use_schema(&mut self, schema_id: &str) {
        let schema_id = Self::normalize_schema_id(schema_id);
        if self.get_active_schema() != Some(schema_id.clone()) {
            if let Some(schema) = self.schemas.iter().find(|s| s.schema_id == schema_id).cloned() {
                self.reset_schema_vars_for_schema(&schema.schema_id);
                self.originals.clear();
                self.components.clear();
                
                self.components = schema.components.clone();
                self.originals = schema.originals.clone();
                self.active_local_vars = Some(schema.variables.clone());
                self.add_to_schema_history(schema.schema_id.clone());
                self.rebuild_params_store();
                info!("[UI] Using schema '{}'", schema.schema_id);
            } else {
                error!("[UI] Schema '{}' not found", schema_id);
            }
        } else {
            info!("[UI] Schema '{}' is already active", schema_id);
        }
    }

    pub fn get_active_schema(&self) -> Option<String> {
        self.schema_history.last().cloned()
    }

    pub fn get_schema_history(&self) -> Vec<String> {
        self.schema_history.clone()
    }

    fn schema_by_id(&self, schema_id: &str) -> Option<&LoadedSchema> {
        self.schemas.iter().find(|schema| schema.schema_id == schema_id)
    }

    fn schema_by_id_mut(&mut self, schema_id: &str) -> Option<&mut LoadedSchema> {
        self.schemas.iter_mut().find(|schema| schema.schema_id == schema_id)
    }

    fn active_local_vars(&self) -> Option<&VarStore> {
        self.active_local_vars.as_ref()
    }

    fn sync_active_local_vars(&mut self) {
        let Some(active_schema_id) = self.get_active_schema() else {
            return;
        };

        if let Some(snapshot) = self.active_local_vars.clone() {
            if let Some(schema) = self.schema_by_id_mut(&active_schema_id) {
                schema.variables = snapshot;
            }
        }
    }

    /// Returns the current computed value of an XML variable.
    /// Prefers the active schema's local vars/consts, then falls back to globals.
    pub fn xml_variable(&self, name: &str) -> Option<XmlVariableValue> {
        if let Some(local_vars) = self.active_local_vars() {
            if let Some(value) = local_vars.get_value(name) {
                return Some(value);
            }
        }

        self.globals.get_value(name)
    }

    /// Returns the computed value of a variable in a specific loaded schema.
    pub fn xml_variable_in_schema(&self, schema_id: &str, name: &str) -> Option<XmlVariableValue> {
        self.schema_by_id(schema_id)
            .and_then(|schema| schema.variables.get_value(name))
            .or_else(|| self.globals.get_value(name))
    }

    /// Sets a mutable XML variable from Rust. Consts are rejected.
    /// Local vars in the active schema are preferred over globals.
    pub fn set_xml_variable(&mut self, name: &str, value: XmlVariableValue) -> bool {
        if let Some(active_vars) = self.active_local_vars.as_ref() {
            if active_vars.has_value(name) {
                let updated = self
                    .active_local_vars
                    .as_mut()
                    .map(|vars| vars.set_value(name, value.clone()))
                    .unwrap_or(false);
                if updated {
                    self.sync_active_local_vars();
                }
                return updated;
            }
        }

        if self.globals.has_value(name) {
            return self.globals.set_value(name, value);
        }

        false
    }

    /// Sets a mutable XML variable in a specific schema.
    pub fn set_xml_variable_in_schema(&mut self, schema_id: &str, name: &str, value: XmlVariableValue) -> bool {
        let updated_local = if let Some(schema) = self.schema_by_id_mut(schema_id) {
            if schema.variables.has_value(name) {
                schema.variables.set_value(name, value.clone())
            } else {
                false
            }
        } else {
            false
        };

        if updated_local {
            if self.get_active_schema().as_deref() == Some(schema_id) {
                self.active_local_vars = self.schema_by_id(schema_id).map(|schema| schema.variables.clone());
            }
            return true;
        }

        if self.globals.has_value(name) {
            return self.globals.set_value(name, value);
        }

        false
    }

    pub fn set_xml_f32_variable(&mut self, name: &str, value: f32) -> bool {
        self.set_xml_variable(name, XmlVariableValue::NumberF32(value))
    }

    pub fn set_xml_i32_variable(&mut self, name: &str, value: i32) -> bool {
        self.set_xml_variable(name, XmlVariableValue::NumberI32(value))
    }

    pub fn set_xml_usize_variable(&mut self, name: &str, value: usize) -> bool {
        self.set_xml_variable(name, XmlVariableValue::NumberUsize(value))
    }

    pub fn set_xml_u32_variable(&mut self, name: &str, value: u32) -> bool {
        self.set_xml_variable(name, XmlVariableValue::NumberU32(value))
    }

    pub fn set_xml_color_variable(&mut self, name: &str, value: Color) -> bool {
        self.set_xml_variable(name, XmlVariableValue::Color(value))
    }

    pub fn set_xml_string_variable(&mut self, name: &str, value: &str) -> bool {
        self.set_xml_variable(name, XmlVariableValue::String(value.to_string()))
    }

    pub fn set_xml_bool_variable(&mut self, name: &str, value: bool) -> bool {
        self.set_xml_variable(name, XmlVariableValue::Bool(value))
    }

    /// Defines a schema-scoped variable with specified initial and reset values.
    /// The variable is initialized to `initial` in all schemas and reset to `reset` when switching schemas.
    pub fn define_schema_variable(&mut self, name: &str, initial: XmlVariableValue, reset: XmlVariableValue) -> bool {
        let same_type = matches!(
            (&initial, &reset),
            (XmlVariableValue::NumberF32(_), XmlVariableValue::NumberF32(_))
                | (XmlVariableValue::NumberI32(_), XmlVariableValue::NumberI32(_))
                | (XmlVariableValue::NumberUsize(_), XmlVariableValue::NumberUsize(_))
                | (XmlVariableValue::NumberU32(_), XmlVariableValue::NumberU32(_))
                | (XmlVariableValue::Color(_), XmlVariableValue::Color(_))
                | (XmlVariableValue::String(_), XmlVariableValue::String(_))
                | (XmlVariableValue::Bool(_), XmlVariableValue::Bool(_))
        );

        if !same_type {
            error!("[UI] Schema variable '{}' must use the same type for initial and reset values", name);
            return false;
        }

        self.schema_var_specs.insert(
            name.to_string(),
            SchemaVarSpec {
                initial: initial.clone(),
                reset: reset.clone(),
            },
        );

        for schema in &self.schemas {
            self.schema_var_values
                .entry(schema.schema_id.clone())
                .or_default()
                .insert(name.to_string(), initial.clone());
        }

        true
    }

    /// Sets the value of a schema-scoped variable in the active schema. Returns false if the variable is not defined or if the type doesn't match the declaration.
    pub fn set_schema_variable(&mut self, name: &str, value: XmlVariableValue) -> bool {
        let Some(active_schema_id) = self.get_active_schema() else {
            return false;
        };

        let Some(spec) = self.schema_var_specs.get(name) else {
            return false;
        };

        let same_type = matches!(
            (&spec.initial, &value),
            (XmlVariableValue::NumberF32(_), XmlVariableValue::NumberF32(_))
                | (XmlVariableValue::NumberI32(_), XmlVariableValue::NumberI32(_))
                | (XmlVariableValue::NumberUsize(_), XmlVariableValue::NumberUsize(_))
                | (XmlVariableValue::NumberU32(_), XmlVariableValue::NumberU32(_))
                | (XmlVariableValue::Color(_), XmlVariableValue::Color(_))
                | (XmlVariableValue::String(_), XmlVariableValue::String(_))
                | (XmlVariableValue::Bool(_), XmlVariableValue::Bool(_))
        );

        if !same_type {
            error!("[UI] Type mismatch while setting schema variable '{}'", name);
            return false;
        }

        self.schema_var_values
            .entry(active_schema_id)
            .or_default()
            .insert(name.to_string(), value);
        true
    }

    pub fn schema_variable(&self, name: &str) -> Option<XmlVariableValue> {
        let active_schema_id = self.get_active_schema()?;
        self.schema_var_values
            .get(&active_schema_id)
            .and_then(|values| values.get(name).cloned())
    }

    pub fn define_schema_var<T: Into<XmlVariableValue>>(&mut self, name: &str, initial: T, reset: T) -> bool {
        self.define_schema_variable(name, initial.into(), reset.into())
    }

    pub fn set_schema_var<T: Into<XmlVariableValue>>(&mut self, name: &str, value: T) -> bool {
        self.set_schema_variable(name, value.into())
    }

    pub fn schema_var<T>(&self, name: &str) -> Option<T>
    where
        T: TryFrom<XmlVariableValue>,
    {
        self.schema_variable(name)
            .and_then(|value| T::try_from(value).ok())
    }

    pub fn move_back_schema(&mut self, amount: i32) {
        if amount >= 0 {
            error!("[UI] move_back_schema expects a negative amount to move back in history");
            return;
        }

        let history_len = self.schema_history.len() as i32;
        if history_len == 0 {
            error!("[UI] No schema history available to move back through");
            return;
        }

        let current_index = history_len - 1;
        let new_index = current_index + amount;
        if new_index < 0 || new_index >= history_len {
            error!("[UI] Cannot move back {} steps in schema history — index out of bounds", -amount);
            return;
        }

        let target_schema_id = self.schema_history[new_index as usize].clone();
        self.use_schema(&target_schema_id);
    }

    fn add_to_schema_history(&mut self, filename: String) {
        let max = 10;

        if self.schema_history.len() >= max {
            self.schema_history.remove(0);
        }

        self.schema_history.push(filename);
    }

    // * -- Render all components (resolves layout + handles input + draws) -- *
    pub fn render(&mut self) {
        let vw = screen_width();
        let vh = screen_height();
        let root = ResolvedLayout { x: 0.0, y: 0.0, w: vw, h: vh, align_x: AlignX::Left, align_y: AlignY::Top };
        self.layout_cache.clear();
        self.click_cache.clear();
        self.release_cache.clear();
        UIRenderer::render(
            &mut self.components,
            root,
            vw,
            vh,
            &mut self.layout_cache,
            &mut self.click_cache,
            &mut self.release_cache,
            &self.texture_cache,
            &mut self.physics_2d_by_id,
            &mut self.physics_2d_by_class,
            &mut self.physics_3d_by_id,
            &mut self.physics_3d_by_class,
        );
        self.rebuild_params_store();
    }

    fn collect_params_snapshot(
        components: &[UIComponent],
        layout_cache: &HashMap<String, ResolvedLayout>,
        out: &mut HashMap<String, ElementParams>,
    ) {
        for component in components {
            let id = component.params().id.clone();
            let layout = if id.is_empty() {
                None
            } else {
                layout_cache.get(&id).copied()
            };

            let snapshot = component_params(component, layout);
            if !snapshot.id.is_empty() {
                out.insert(snapshot.id.clone(), snapshot);
            }

            Self::collect_params_snapshot(component.children(), layout_cache, out);
        }
    }

    fn rebuild_params_store(&mut self) {
        self.params_store.clear();
        Self::collect_params_snapshot(&self.components, &self.layout_cache, &mut self.params_store);
    }

    /// Returns the current computed params of an element by id.
    /// Values come from the live runtime component state, not raw XML text.
    pub fn params_by_id(&mut self, id: &str) -> Option<ElementParams> {
        self.rebuild_params_store();
        self.params_store.get(id).cloned()
    }

    /// Returns true if the component with the given id was clicked this frame.
    /// Safe to call before `get_by_id` since it only borrows `self` immutably.
    pub fn was_clicked(&self, id: &str) -> bool {
        self.click_cache.contains(id)
    }

    pub fn was_released(&self, id: &str) -> bool {
        self.release_cache.contains(id)
    }

    // * ======== TEXTURE LOADING ======== *
    /// Load all textures referenced in the UI schema into the texture cache.
    /// Call once after `load_schema()`. Paths prefixed with `@/` are resolved
    /// relative to the working directory (project root under `cargo run`).
    pub async fn load_textures(&mut self) {
        if self.schemas.is_empty() {
            error!("[UI] No loaded schemas — cannot load textures");
            return;
        }
        let all_components = self.schemas.iter().flat_map(|s| s.components.clone()).collect::<Vec<_>>();

        // --- Static textures ---
        let sources = Self::collect_texture_srcs(&all_components);
        for src in sources {
            if self.texture_cache.contains_key(&src) { continue; }
            let path = Self::resolve_asset_path(&src);
            match load_texture(&path).await {
                Ok(tex) => {
                    info!("[UI::Texture] Loaded '{}'", path);
                    tex.set_filter(FilterMode::Nearest);
                    self.texture_cache.insert(src, tex);
                }
                Err(e) => {
                    error!("[UI::Texture] Failed to load '{}': {:?}", path, e);
                }
            }
        }

        // --- Animated textures: read conf.jsonc, load frames ---
        let anim_srcs = Self::collect_anim_srcs(&all_components);
        for src in anim_srcs {
            let folder = Self::resolve_asset_path(&src);
            let conf_path = format!("{}/conf.jsonc", folder);
            let conf_text = match std::fs::read_to_string(&conf_path) {
                Ok(t)  => t,
                Err(e) => { error!("[UI::Anim] Cannot read '{}': {}", conf_path, e); continue; }
            };
            let stripped = Self::strip_jsonc_comments(&conf_text);
            let conf: AnimConfRaw = match serde_json::from_str(&stripped) {
                Ok(c)  => c,
                Err(e) => { error!("[UI::Anim] Cannot parse '{}': {}", conf_path, e); continue; }
            };
            let timing = Self::parse_timing_function(&conf.timing_function);
            let count  = conf.frames.count;
            let frames_sub = conf.frames.path.trim_start_matches("./").trim_start_matches('/');
            for i in 0..count {
                let filename   = Self::expand_frame_format(&conf.frames.format, i);
                let frame_path = if frames_sub.is_empty() {
                    format!("{}/{}", folder, filename)
                } else {
                    format!("{}/{}/{}", folder, frames_sub, filename)
                };
                let cache_key = format!("{}/frame_{}", src, i);
                if self.texture_cache.contains_key(&cache_key) { continue; }
                match load_texture(&frame_path).await {
                    Ok(tex) => {
                        tex.set_filter(FilterMode::Nearest);
                        self.texture_cache.insert(cache_key, tex);
                        info!("[UI::Anim] Loaded frame {} for '{}'", i, src);
                    }
                    Err(e) => {
                        error!("[UI::Anim] Frame {} for '{}' failed: {:?}", i, src, e);
                    }
                }
            }
            // Propagate frame_count + timing to all matching components
            for schema in self.schemas.iter_mut() {
                Self::update_anim_data(&mut schema.components, &src, count, timing);
            }
            Self::update_anim_data(&mut self.components, &src, count, timing);
        }
    }

    /// Strip `@/` workspace-root prefix, leaving a relative path.
    fn resolve_asset_path(src: &str) -> String {
        src.strip_prefix("@/").unwrap_or(src).to_string()
    }

    fn collect_texture_srcs(components: &[UIComponent]) -> Vec<String> {
        let mut out = Vec::new();
        for c in components {
            if let UIComponent::Texture { src, .. } = c {
                if !src.is_empty() { out.push(src.clone()); }
            }
            out.extend(Self::collect_texture_srcs(c.children()));
        }
        out
    }

    fn collect_anim_srcs(components: &[UIComponent]) -> Vec<String> {
        let mut out = Vec::new();
        for c in components {
            if let UIComponent::AnimatedTexture { src, .. } = c {
                if !src.is_empty() { out.push(src.clone()); }
            }
            out.extend(Self::collect_anim_srcs(c.children()));
        }
        out
    }

    /// Update frame_count + timing on every AnimatedTexture whose src matches.
    fn update_anim_data(components: &mut Vec<UIComponent>, src: &str, count: u32, timing: [f32; 4]) {
        for c in components.iter_mut() {
            if let UIComponent::AnimatedTexture { src: c_src, frame_count, timing: t, .. } = c {
                if c_src == src {
                    *frame_count = count;
                    *t           = timing;
                }
            }
            if let Some(children) = c.children_mut() {
                Self::update_anim_data(children, src, count, timing);
            }
        }
    }

    // * ======== JSONC / ANIM CONF HELPERS ========= *

    /// Strip `//` line comments so the text can be parsed as plain JSON.
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
                while let Some(nc) = chars.next() {
                    if nc == '\n' { out.push('\n'); break; }
                }
                continue;
            }
            out.push(c);
        }
        out
    }

    /// Parse a comma-separated bezier string like `".17,.67,.62,1.03"` into `[x1,y1,x2,y2]`.
    /// Falls back to linear `[0,0,1,1]` if the string is malformed.
    fn parse_timing_function(s: &str) -> [f32; 4] {
        let v: Vec<f32> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        if v.len() == 4 { [v[0], v[1], v[2], v[3]] } else { [0.0, 0.0, 1.0, 1.0] }
    }

    /// Expand a frame format string for frame index `i`.
    /// `"$0.png"` with i=3  →  `"3.png"` (start=0)
    /// `"$1.png"` with i=3  →  `"4.png"` (start=1)
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

    // * ======== QUERY BY ID ======== *
    /// Returns a mutable handle for the component with the given id.
    /// Includes all read + write methods (scale, set_color, reset_parameter, on_change, etc.).
    pub fn get_by_id(&mut self, id: &str) -> Option<UIHandleMut> {
        let layout = self.layout_cache.get(id).copied();
        let originals = &self.originals;
        Self::find_by_id_mut(&mut self.components, id)
            .map(|c| UIHandleMut::new(c, layout, originals))
    }

    /// Alias kept for API compatibility — same as get_by_id().
    pub fn get_by_id_mut(&mut self, id: &str) -> Option<UIHandleMut> {
        self.get_by_id(id)
    }

    // * ======== QUERY BY CLASS ======== *
    pub fn get_by_class(&self, class: &str) -> Option<UIHandle> {
        Self::find_by_class(&self.components, class).map(|c| {
            let layout = self.layout_cache.get(&c.params().id).copied();
            UIHandle::new(c, layout)
        })
    }

    pub fn get_by_class_mut(&mut self, class: &str) -> Option<UIHandleMut> {
        let layout = self.layout_cache.get(class).copied();
        let originals = &self.originals;
        Self::find_by_class_mut(&mut self.components, class)
            .map(|c| UIHandleMut::new(c, layout, originals))
    }

    pub fn get_all_by_class<'a>(&'a self, class: &str) -> Vec<UIHandle<'a>> {
        let mut result = Vec::new();
        Self::collect_by_class(&self.components, class, &self.layout_cache, &mut result);
        result
    }

    pub fn get_all_by_class_mut<'a>(&'a mut self, class: &str) -> Vec<UIHandleMut<'a>> {
        let mut result = Vec::new();
        let originals = &self.originals;
        Self::collect_by_class_mut(&mut self.components, class, &self.layout_cache, &mut result);
        result.into_iter().map(|c| {
            let layout = self.layout_cache.get(&c.params().id).copied();
            UIHandleMut::new(c, layout, originals)
        }).collect()
    }

    // * ======== RECURSIVE SEARCH HELPERS ======== *
    #[allow(dead_code)]
    fn find_by_id<'a>(components: &'a [UIComponent], id: &str) -> Option<&'a UIComponent> {
        for component in components {
            if component.params().id == id { return Some(component); }
            if let Some(found) = Self::find_by_id(component.children(), id) { return Some(found); }
        }
        None
    }

    fn find_by_id_mut<'a>(components: &'a mut [UIComponent], id: &str) -> Option<&'a mut UIComponent> {
        for component in components.iter_mut() {
            if component.params().id == id { return Some(component); }
            if let Some(children) = component.children_mut() {
                if let Some(found) = Self::find_by_id_mut(children, id) { return Some(found); }
            }
        }
        None
    }

    fn find_by_class<'a>(components: &'a [UIComponent], class: &str) -> Option<&'a UIComponent> {
        for component in components {
            if Self::class_matches(&component.params().class_name, class) { return Some(component); }
            if let Some(found) = Self::find_by_class(component.children(), class) { return Some(found); }
        }
        None
    }

    fn find_by_class_mut<'a>(components: &'a mut [UIComponent], class: &str) -> Option<&'a mut UIComponent> {
        for component in components.iter_mut() {
            if Self::class_matches(&component.params().class_name, class) { return Some(component); }
            if let Some(children) = component.children_mut() {
                if let Some(found) = Self::find_by_class_mut(children, class) { return Some(found); }
            }
        }
        None
    }

    fn collect_by_class<'a>(
        components: &'a [UIComponent],
        class: &str,
        cache: &HashMap<String, ResolvedLayout>,
        result: &mut Vec<UIHandle<'a>>,
    ) {
        for component in components {
            if Self::class_matches(&component.params().class_name, class) {
                let layout = cache.get(&component.params().id).copied();
                result.push(UIHandle::new(component, layout));
            }
            Self::collect_by_class(component.children(), class, cache, result);
        }
    }

    fn collect_by_class_mut<'a>(
        components: &'a mut [UIComponent],
        class: &str,
        cache: &HashMap<String, ResolvedLayout>,
        result: &mut Vec<&'a mut UIComponent>,
    ) {
        for component in components.iter_mut() {
            if Self::class_matches(&component.params().class_name, class) {
                result.push(component);
            } else if let Some(children) = component.children_mut() {
                Self::collect_by_class_mut(children, class, cache, result);
            }
        }
    }
}

