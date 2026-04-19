// * ======== UI MODULE ======== *
// * ======== IMPORTS ======== *
use macroquad::color::Color;

// * ======== TRANSITION TYPES ======== *
#[derive(Debug, Clone, Copy)]
pub enum TransitionEasing {
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    Linear,
    CubicBezier([f32; 4]),
}

impl Default for TransitionEasing {
    fn default() -> Self { Self::Ease }
}

#[derive(Debug, Clone, Copy)]
pub struct TransitionConfig {
    pub duration: f32,
    pub easing: TransitionEasing,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration: 0.0,
            easing: TransitionEasing::Ease,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorTween {
    pub from: Color,
    pub to: Color,
    pub elapsed: f32,
    pub duration: f32,
    pub easing: TransitionEasing,
}

#[derive(Debug, Clone, Copy)]
pub struct FloatTween {
    pub from: f32,
    pub to: f32,
    pub elapsed: f32,
    pub duration: f32,
    pub easing: TransitionEasing,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-6
}

fn color_approx_eq(a: Color, b: Color) -> bool {
    approx_eq(a.r, b.r)
        && approx_eq(a.g, b.g)
        && approx_eq(a.b, b.b)
        && approx_eq(a.a, b.a)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        lerp(a.r, b.r, t),
        lerp(a.g, b.g, t),
        lerp(a.b, b.b, t),
        lerp(a.a, b.a, t),
    )
}

fn cubic_bezier_easing(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    if t <= 0.0 { return 0.0; }
    if t >= 1.0 { return 1.0; }

    let cx = 3.0 * x1;
    let bx = 3.0 * (x2 - x1) - cx;
    let ax = 1.0 - cx - bx;

    let cy = 3.0 * y1;
    let by_ = 3.0 * (y2 - y1) - cy;
    let ay = 1.0 - cy - by_;

    let mut s = t;
    for _ in 0..8 {
        let x_err = ((ax * s + bx) * s + cx) * s - t;
        let dx = (3.0 * ax * s + 2.0 * bx) * s + cx;
        if dx.abs() < 1e-6 { break; }
        s = (s - x_err / dx).clamp(0.0, 1.0);
    }
    ((ay * s + by_) * s + cy) * s
}

pub fn apply_easing(t: f32, easing: TransitionEasing) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        TransitionEasing::Linear => t,
        TransitionEasing::Ease => cubic_bezier_easing(t, 0.25, 0.1, 0.25, 1.0),
        TransitionEasing::EaseIn => cubic_bezier_easing(t, 0.42, 0.0, 1.0, 1.0),
        TransitionEasing::EaseOut => cubic_bezier_easing(t, 0.0, 0.0, 0.58, 1.0),
        TransitionEasing::EaseInOut => cubic_bezier_easing(t, 0.42, 0.0, 0.58, 1.0),
        TransitionEasing::CubicBezier(points) => cubic_bezier_easing(t, points[0], points[1], points[2], points[3]),
    }
}

// * ======== SIZE EXPRESSION ======== *
// An expression tree for computed dimensional values.
// Leaf units: "40px"  "50%"  "30vw"  "20vh"  "-20px" (from opposite side)
// Operators:   +  -  *  /   with  ()  for grouping
// Example:  "(40px + 2%) - 2vh + 3vw"
#[derive(Debug, Clone)]
pub enum SizeExpr {
    Px(f32),         // absolute pixels from parent origin       e.g. "40px"
    NegPx(f32),      // pixels from the opposite side            e.g. "-20px" → parent_dim - 20
    Pct(f32),        // % of parent dimension                    e.g. "50%"
    Vw(f32),         // % of viewport width                      e.g. "30vw"
    Vh(f32),         // % of viewport height                     e.g. "20vh"
    Deg(f32),        // angle in degrees, resolves to raw value  e.g. "90deg"
    BinOp(Box<SizeExpr>, SizeOp, Box<SizeExpr>),
}

#[derive(Debug, Clone, Copy)]
pub enum SizeOp { Add, Sub, Mul, Div }

impl SizeExpr {
    pub fn resolve(&self, parent_dim: f32, vw: f32, vh: f32) -> f32 {
        match self {
            SizeExpr::Px(v)    => *v,
            SizeExpr::NegPx(v) => parent_dim - v,
            SizeExpr::Pct(v)   => parent_dim * v / 100.0,
            SizeExpr::Vw(v)    => vw * v / 100.0,
            SizeExpr::Vh(v)    => vh * v / 100.0,
            SizeExpr::Deg(v)   => *v,
            SizeExpr::BinOp(l, op, r) => {
                let lv = l.resolve(parent_dim, vw, vh);
                let rv = r.resolve(parent_dim, vw, vh);
                match op {
                    SizeOp::Add => lv + rv,
                    SizeOp::Sub => lv - rv,
                    SizeOp::Mul => lv * rv,
                    SizeOp::Div => if rv.abs() > f32::EPSILON { lv / rv } else { 0.0 },
                }
            }
        }
    }
}

// * ======== SIZE VALUE ======== *
// Width/height of a UI component.
//   "auto"                         → Auto       (content-measured, e.g. text extents for labels)
//   "40px", "50%", "(40px+2%)-2vh" → Expr(…)   (literal or computed expression)
#[derive(Debug, Clone)]
pub enum SizeValue {
    Auto,           // content-measured
    Expr(SizeExpr), // computed expression
}

impl SizeValue {
    pub fn resolve(&self, parent_dim: f32, vw: f32, vh: f32) -> f32 {
        match self {
            SizeValue::Auto       => 0.0,  // sentinel; renderer overrides with measured size
            SizeValue::Expr(expr) => expr.resolve(parent_dim, vw, vh),
        }
    }

    pub fn is_auto(&self) -> bool { matches!(self, SizeValue::Auto) }
}

impl Default for SizeValue {
    fn default() -> Self { SizeValue::Expr(SizeExpr::Px(50.0)) }
}

// * ======== POS VALUE ======== *
// x/y position of a UI component.
//   "40px"          → Px(SizeExpr::Px(40))         — 40px from parent origin
//   "50%"           → Px(SizeExpr::Pct(50))         — 50% of parent dim from origin
//   "(40px + 2%)"   → Px(SizeExpr::BinOp(…))        — computed position
//   "auto" / "$"    → Auto(SizeExpr::Px(0))          — parent-aligned, no offset
//   "$+20px"        → Auto(SizeExpr::Px(20))         — parent-aligned + 20px offset
//   "$(40px + 2%)"  → Auto(SizeExpr::BinOp(…))       — parent-aligned + computed offset
#[derive(Debug, Clone)]
pub enum PosValue {
    Px(SizeExpr),    // position relative to parent top-left (expr resolved with parent dim)
    Auto(SizeExpr),  // parent-alignment controlled + computed offset
}

impl PosValue {
    pub fn is_auto(&self) -> bool { matches!(self, PosValue::Auto(_)) }
}

impl Default for PosValue {
    fn default() -> Self { PosValue::Px(SizeExpr::Px(0.0)) }
}

// * ======== OBJECT FIT ======== *
// Controls how a texture is sized within its layout bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectFit {
    Warp,    // stretch texture to fill the entire bounds (distorts aspect ratio)
    Cover,   // scale to fill bounds, cropping excess (preserves aspect ratio)
    Contain, // scale to fit inside bounds without cropping (preserves aspect ratio)
}

impl Default for ObjectFit { fn default() -> Self { ObjectFit::Warp } }

// * ======== GRADIENT TYPE ======== *
// Controls whether a gradient is linear (line) or radial (from center outward).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientType {
    Linear,   // linear gradient along an angle
    Radial, // radial gradient emanating from center
}

impl Default for GradientType { fn default() -> Self { GradientType::Linear } }

// * ======== ALIGNMENT ======== *
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignX { Left, Center, Right }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignY { Top, Center, Bottom }

impl Default for AlignX { fn default() -> Self { AlignX::Left } }
impl Default for AlignY { fn default() -> Self { AlignY::Top } }

/// Selects the runtime backend used by a `physics-container-2d` element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Physics2DEngine {
    /// Use a macroquad-based scene callback.
    Macroquad,
    /// Use a bevy-based scene callback.
    Bevy,
}

impl Default for Physics2DEngine {
    fn default() -> Self { Self::Macroquad }
}

/// Runtime rectangle and metadata passed to 2D physics render callbacks.
#[derive(Debug, Clone)]
pub struct PhysicsRenderContext2D {
    /// Component id (can be empty if not provided in XML).
    pub id: String,
    /// Component class list string.
    pub class_name: String,
    /// The resolved container bounds in screen pixels.
    pub layout: ResolvedLayout,
    /// Selected 2D backend for this container.
    pub engine: Physics2DEngine,
    /// Whether the container is currently visible.
    pub visible: bool,
    /// UI z-order value associated with this container.
    pub z_order: i32,
}

/// Runtime rectangle and metadata passed to 3D physics render callbacks.
#[derive(Debug, Clone)]
pub struct PhysicsRenderContext3D {
    /// Component id (can be empty if not provided in XML).
    pub id: String,
    /// Component class list string.
    pub class_name: String,
    /// The resolved container bounds in screen pixels.
    pub layout: ResolvedLayout,
    /// Whether the container is currently visible.
    pub visible: bool,
    /// UI z-order value associated with this container.
    pub z_order: i32,
}

/// Callback signature used to render external 2D engines inside UI physics containers.
pub type Physics2DRenderCallback = Box<dyn FnMut(PhysicsRenderContext2D)>;

/// Callback signature used to render external 3D engines inside UI physics containers.
pub type Physics3DRenderCallback = Box<dyn FnMut(PhysicsRenderContext3D)>;

// * ======== BASIC PARAMS ======== *
#[derive(Debug, Clone)]
pub struct BasicParams {
    // Size — stored as SizeValue so % / vw / vh can be resolved at render time
    pub width: SizeValue,
    pub height: SizeValue,

    // Position — PosValue so "auto" / "&+offset" works
    pub x: PosValue,
    pub y: PosValue,

    pub rotation: f32, // In degrees
    pub scale: f32,
    pub font_size: u32,

    pub id: String,
    pub class_name: String,

    pub z_order: i32,
    pub visible: bool,
    pub usr_interact: bool,
    pub bg_color: Color,
    pub transition: TransitionConfig,
    pub bg_color_tween: Option<ColorTween>,
    pub scale_tween: Option<FloatTween>,
}

impl Default for BasicParams {
    fn default() -> Self {
        Self {
            width: SizeValue::Expr(SizeExpr::Px(50.0)),
            height: SizeValue::Expr(SizeExpr::Px(50.0)),
            x: PosValue::Px(SizeExpr::Px(0.0)),
            y: PosValue::Px(SizeExpr::Px(0.0)),
            rotation: 0.0,
            scale: 1.0,
            font_size: 16,
            id: String::new(),
            class_name: String::new(),
            z_order: 0,
            visible: true,
            usr_interact: true,
            bg_color: Color::new(0.0, 0.0, 0.0, 0.0),
            transition: TransitionConfig::default(),
            bg_color_tween: None,
            scale_tween: None,
        }
    }
}

impl BasicParams {
    pub fn transition_enabled(&self) -> bool {
        self.transition.duration > 0.0
    }

    pub fn set_bg_color_with_transition(&mut self, target: Color) {
        if !self.transition_enabled() {
            self.bg_color = target;
            self.bg_color_tween = None;
            return;
        }

        // Avoid restarting the same tween every frame (e.g. hover loop),
        // which would make perceived duration depend on FPS.
        if let Some(active) = self.bg_color_tween {
            if color_approx_eq(active.to, target) {
                return;
            }
        }

        if color_approx_eq(self.bg_color, target) {
            self.bg_color = target;
            self.bg_color_tween = None;
            return;
        }

        self.bg_color_tween = Some(ColorTween {
            from: self.bg_color,
            to: target,
            elapsed: 0.0,
            duration: self.transition.duration,
            easing: self.transition.easing,
        });
    }

    pub fn set_scale_with_transition(&mut self, target: f32) {
        if !self.transition_enabled() {
            self.scale = target;
            self.scale_tween = None;
            return;
        }

        // Avoid restarting the same tween every frame (e.g. hover loop),
        // which would make perceived duration depend on FPS.
        if let Some(active) = self.scale_tween {
            if approx_eq(active.to, target) {
                return;
            }
        }

        if approx_eq(self.scale, target) {
            self.scale = target;
            self.scale_tween = None;
            return;
        }

        self.scale_tween = Some(FloatTween {
            from: self.scale,
            to: target,
            elapsed: 0.0,
            duration: self.transition.duration,
            easing: self.transition.easing,
        });
    }

    pub fn update_tweens(&mut self, dt: f32) {
        if let Some(mut tw) = self.bg_color_tween {
            tw.elapsed += dt;
            let t = if tw.duration <= 0.0 { 1.0 } else { (tw.elapsed / tw.duration).clamp(0.0, 1.0) };
            let eased = apply_easing(t, tw.easing);
            self.bg_color = lerp_color(tw.from, tw.to, eased);
            if t >= 1.0 {
                self.bg_color_tween = None;
            } else {
                self.bg_color_tween = Some(tw);
            }
        }

        if let Some(mut tw) = self.scale_tween {
            tw.elapsed += dt;
            let t = if tw.duration <= 0.0 { 1.0 } else { (tw.elapsed / tw.duration).clamp(0.0, 1.0) };
            let eased = apply_easing(t, tw.easing);
            self.scale = lerp(tw.from, tw.to, eased);
            if t >= 1.0 {
                self.scale_tween = None;
            } else {
                self.scale_tween = Some(tw);
            }
        }
    }
}

// * ======== RESOLVED LAYOUT ======== *
// Computed absolute pixel values for a component — calculated once per frame by the renderer.
// align_x/align_y on a ResolvedLayout describe how THIS node aligns its OWN children.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedLayout {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// How children are aligned horizontally inside this container
    pub align_x: AlignX,
    /// How children are aligned vertically inside this container
    pub align_y: AlignY,
}

impl Default for ResolvedLayout {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, w: 0.0, h: 0.0, align_x: AlignX::Left, align_y: AlignY::Top }
    }
}

// * ======== UI COMPONENT TREE ======== *
#[derive(Clone, Debug)]
pub enum UIComponent {
    // Layout
    View {
        params: BasicParams,
        align_x: AlignX,
        align_y: AlignY,
        children: Vec<UIComponent>,
    },
    ScrollView {
        params: BasicParams,
        align_x: AlignX,
        align_y: AlignY,
        scroll_x: bool,
        scroll_y: bool,
        children: Vec<UIComponent>,
    },

    // Interactive
    Button {
        params: BasicParams,
        children: Vec<UIComponent>,
        text: String,
        color: Color,
        align_x: AlignX,
        align_y: AlignY,
        // runtime state
        hovered: bool,
        pressed: bool,
        color_tween: Option<ColorTween>,
    },
    Slider {
        params: BasicParams,
        min: f32,
        max: f32,
        value: f32,
        color: Color,
        track_color: Color,
        // runtime state
        dragging: bool,
        changed: bool,
        color_tween: Option<ColorTween>,
    },
    Switch {
        params: BasicParams,
        checked: bool,
        color: Color,
        color_tween: Option<ColorTween>,
    },

    // Text
    Label {
        params: BasicParams,
        text: String,
        color: Color,
        color_tween: Option<ColorTween>,
    },
    TextField {
        params: BasicParams,
        placeholder: String,
        text: String,
        color: Color,
        // runtime state
        focused: bool,
        color_tween: Option<ColorTween>,
    },

    // Visual
    Texture {
        params: BasicParams,
        src: String,
        object_fit: ObjectFit,
    },
    AnimatedTexture {
        params: BasicParams,
        /// Path to the folder containing `conf.jsonc` and the frame images.
        src: String,
        /// Playback speed in frames per second (from XML attribute).
        fps: f32,
        loop_anim: bool,
        autoplay: bool,
        /// Draw opacity: 0.0 (invisible) – 1.0 (fully opaque).
        opacity: f32,
        object_fit: ObjectFit,
        // Populated from conf.jsonc at asset-load time.
        /// Total number of frames (0 until conf.jsonc is loaded).
        frame_count: u32,
        /// Cubic-bezier easing control points [x1, y1, x2, y2].  Defaults to ease-in-out.
        timing: [f32; 4],
        // Runtime state
        playing: bool,
        paused: bool,
        /// Currently displayed frame index (0-based).
        current_frame: u32,
        /// Accumulated elapsed playback time in seconds.
        elapsed: f32,
    },

    // Physics Containers
    PhysicsContainer2D {
        params: BasicParams,
        engine: Physics2DEngine,
        children: Vec<UIComponent>,
    },
    PhysicsContainer3D {
        params: BasicParams,
        children: Vec<UIComponent>,
    },

    // Other
    ProgressBar {
        params: BasicParams,
        value: f32,
        max: f32,
        color: Color,
        color_tween: Option<ColorTween>,
    },
    Rect {
        params: BasicParams,
        border_color: Color,
        border_width: f32,
        corner_radius: f32,
    },
    Gradient {
        params: BasicParams,
        gradient_type: GradientType,
        /// Gradient angle in degrees, Cartesian convention:
        /// 0° = right, 90° = up (CCW positive). Only used for Line type.
        angle: f32,
        color1: Color,
        color2: Color,
    },
}

impl UIComponent {
    pub fn params(&self) -> &BasicParams {
        match self {
            UIComponent::View { params, .. } => params,
            UIComponent::ScrollView { params, .. } => params,
            UIComponent::Button { params, .. } => params,
            UIComponent::Slider { params, .. } => params,
            UIComponent::Switch { params, .. } => params,
            UIComponent::Label { params, .. } => params,
            UIComponent::TextField { params, .. } => params,
            UIComponent::Texture { params, .. } => params,
            UIComponent::AnimatedTexture { params, .. } => params,
            UIComponent::PhysicsContainer2D { params, .. } => params,
            UIComponent::PhysicsContainer3D { params, .. } => params,
            UIComponent::ProgressBar { params, .. } => params,
            UIComponent::Rect { params, .. } => params,
            UIComponent::Gradient { params, .. } => params,
        }
    }

    pub fn params_mut(&mut self) -> &mut BasicParams {
        match self {
            UIComponent::View { params, .. } => params,
            UIComponent::ScrollView { params, .. } => params,
            UIComponent::Button { params, .. } => params,
            UIComponent::Slider { params, .. } => params,
            UIComponent::Switch { params, .. } => params,
            UIComponent::Label { params, .. } => params,
            UIComponent::TextField { params, .. } => params,
            UIComponent::Texture { params, .. } => params,
            UIComponent::AnimatedTexture { params, .. } => params,
            UIComponent::PhysicsContainer2D { params, .. } => params,
            UIComponent::PhysicsContainer3D { params, .. } => params,
            UIComponent::ProgressBar { params, .. } => params,
            UIComponent::Rect { params, .. } => params,
            UIComponent::Gradient { params, .. } => params,
        }
    }

    pub fn children(&self) -> &[UIComponent] {
        match self {
            UIComponent::View { children, .. } => children,
            UIComponent::ScrollView { children, .. } => children,
            UIComponent::Button { children, .. } => children,
            UIComponent::PhysicsContainer2D { children, .. } => children,
            UIComponent::PhysicsContainer3D { children, .. } => children,
            _ => &[],
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<UIComponent>> {
        match self {
            UIComponent::View { children, .. } => Some(children),
            UIComponent::ScrollView { children, .. } => Some(children),
            UIComponent::Button { children, .. } => Some(children),
            UIComponent::PhysicsContainer2D { children, .. } => Some(children),
            UIComponent::PhysicsContainer3D { children, .. } => Some(children),
            _ => None,
        }
    }

    // * -- Alignment of children -- *
    pub fn align_x(&self) -> AlignX {
        match self {
            UIComponent::View { align_x, .. } => *align_x,
            UIComponent::ScrollView { align_x, .. } => *align_x,
            UIComponent::Button { align_x, .. } => *align_x,
            _ => AlignX::Left,
        }
    }

    pub fn align_y(&self) -> AlignY {
        match self {
            UIComponent::View { align_y, .. } => *align_y,
            UIComponent::ScrollView { align_y, .. } => *align_y,
            UIComponent::Button { align_y, .. } => *align_y,
            _ => AlignY::Top,
        }
    }

    pub fn set_color_with_transition(&mut self, target: Color) {
        match self {
            UIComponent::Button { color, color_tween, params, .. }
            | UIComponent::Label { color, color_tween, params, .. }
            | UIComponent::Slider { color, color_tween, params, .. }
            | UIComponent::Switch { color, color_tween, params, .. }
            | UIComponent::TextField { color, color_tween, params, .. }
            | UIComponent::ProgressBar { color, color_tween, params, .. } => {
                if !params.transition_enabled() {
                    *color = target;
                    *color_tween = None;
                } else {
                    // Avoid restarting the same tween every frame (e.g. hover loop),
                    // which would make perceived duration depend on FPS.
                    if let Some(active) = *color_tween {
                        if color_approx_eq(active.to, target) {
                            return;
                        }
                    }

                    if color_approx_eq(*color, target) {
                        *color = target;
                        *color_tween = None;
                        return;
                    }

                    *color_tween = Some(ColorTween {
                        from: *color,
                        to: target,
                        elapsed: 0.0,
                        duration: params.transition.duration,
                        easing: params.transition.easing,
                    });
                }
            }
            _ => {}
        }
    }

    pub fn update_transitions(&mut self, dt: f32) {
        self.params_mut().update_tweens(dt);

        match self {
            UIComponent::Button { color, color_tween, .. }
            | UIComponent::Label { color, color_tween, .. }
            | UIComponent::Slider { color, color_tween, .. }
            | UIComponent::Switch { color, color_tween, .. }
            | UIComponent::TextField { color, color_tween, .. }
            | UIComponent::ProgressBar { color, color_tween, .. } => {
                if let Some(mut tw) = *color_tween {
                    tw.elapsed += dt;
                    let t = if tw.duration <= 0.0 { 1.0 } else { (tw.elapsed / tw.duration).clamp(0.0, 1.0) };
                    let eased = apply_easing(t, tw.easing);
                    *color = lerp_color(tw.from, tw.to, eased);
                    if t >= 1.0 {
                        *color_tween = None;
                    } else {
                        *color_tween = Some(tw);
                    }
                }
            }
            _ => {}
        }
    }
}

// * ======== VALID TAG NAMES ======== *
pub const VALID_TAGS: &[&str] = &[
    "view", "scroll_view",
    "button", "slider", "switch",
    "label", "text_field",
    "texture", "texture-2d", "animated_texture", "animation",
    "progress_bar", "rect", "gradient",
    "physics-container-2d", "physics-container-3d",
];