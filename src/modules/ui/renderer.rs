// * =========== MODULE: UI -> RENDERER =========== *
// * =========== IMPORTS =========== *
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::modules::ui::components;
use crate::modules::ui::types::ui::{
    apply_easing, AlignX, AlignY, ObjectFit, Physics2DRenderCallback, Physics3DRenderCallback,
    PhysicsRenderContext2D, PhysicsRenderContext3D, PosValue, ResolvedLayout, SizeValue,
    TransitionEasing, UIComponent,
};

pub struct UIRenderer;

impl UIRenderer {
    // * -- Entry point: resolve layout + handle input + draw -- *
    // layout_cache is filled per-frame with resolved bounds keyed by component ID
    pub fn render(
        components: &mut Vec<UIComponent>,
        parent: ResolvedLayout,
        vw: f32,
        vh: f32,
        layout_cache: &mut HashMap<String, ResolvedLayout>,
        click_cache:  &mut HashSet<String>,
        release_cache: &mut HashSet<String>,
        texture_cache: &HashMap<String, Texture2D>,
        physics_2d_by_id: &mut HashMap<String, Physics2DRenderCallback>,
        physics_2d_by_class: &mut HashMap<String, Physics2DRenderCallback>,
        physics_3d_by_id: &mut HashMap<String, Physics3DRenderCallback>,
        physics_3d_by_class: &mut HashMap<String, Physics3DRenderCallback>,
    ) {
        let frame_dt = get_frame_time();
        Self::render_list(
            components,
            parent,
            vw,
            vh,
            layout_cache,
            click_cache,
            release_cache,
            texture_cache,
            physics_2d_by_id,
            physics_2d_by_class,
            physics_3d_by_id,
            physics_3d_by_class,
            1.0,
            frame_dt,
        );
    }

    fn render_list(
        components: &mut Vec<UIComponent>,
        parent: ResolvedLayout,
        vw: f32,
        vh: f32,
        layout_cache: &mut HashMap<String, ResolvedLayout>,
        click_cache:  &mut HashSet<String>,
        release_cache: &mut HashSet<String>,
        texture_cache: &HashMap<String, Texture2D>,
        physics_2d_by_id: &mut HashMap<String, Physics2DRenderCallback>,
        physics_2d_by_class: &mut HashMap<String, Physics2DRenderCallback>,
        physics_3d_by_id: &mut HashMap<String, Physics3DRenderCallback>,
        physics_3d_by_class: &mut HashMap<String, Physics3DRenderCallback>,
        parent_scale: f32,
        frame_dt: f32,
    ) {
        let (mx, my) = mouse_position();
        let lmb_down = is_mouse_button_down(MouseButton::Left);
        let lmb_pressed = is_mouse_button_pressed(MouseButton::Left);

        for component in components.iter_mut() {
            if !component.params().visible {
                continue;
            }

            component.update_transitions(frame_dt);

            let effective_scale = parent_scale * component.params().scale;
            let layout = Self::resolve_layout(component, &parent, vw, vh, effective_scale);

            // Cache the resolved layout by id (non-empty ids only)
            let id = component.params().id.clone();
            if !id.is_empty() {
                layout_cache.insert(id.clone(), layout);
            }

            let hovered = mx >= layout.x && mx <= layout.x + layout.w
                && my >= layout.y && my <= layout.y + layout.h;

            // Record click for any hovered+LMB-pressed component with an id
            if hovered && lmb_pressed && !id.is_empty() {
                click_cache.insert(id.clone());
            }

            // Record release for any hovered+LMB-released component with an id
            if hovered && is_mouse_button_released(MouseButton::Left) && !id.is_empty() {
                release_cache.insert(id.clone());
            }

            match component {
                UIComponent::View { params, children, align_x, align_y } => {
                    components::view::draw(layout, params.bg_color);
                    let cp = ResolvedLayout { x: layout.x, y: layout.y, w: layout.w, h: layout.h, align_x: *align_x, align_y: *align_y };
                    Self::render_list(children, cp, vw, vh, layout_cache, click_cache, release_cache, texture_cache, physics_2d_by_id, physics_2d_by_class, physics_3d_by_id, physics_3d_by_class, effective_scale, frame_dt);
                }

                UIComponent::ScrollView { params, children, align_x, align_y, .. } => {
                    components::scroll_view::draw(layout, params.bg_color);
                    let cp = ResolvedLayout { x: layout.x, y: layout.y, w: layout.w, h: layout.h, align_x: *align_x, align_y: *align_y };
                    Self::render_list(children, cp, vw, vh, layout_cache, click_cache, release_cache, texture_cache, physics_2d_by_id, physics_2d_by_class, physics_3d_by_id, physics_3d_by_class, effective_scale, frame_dt);
                }

                UIComponent::Button { params, text, color, children, hovered: hov, pressed: prs, align_x, align_y, .. } => {
                    components::button::draw(layout, params.bg_color, text, *color, params.font_size as f32 * effective_scale, hov, prs, hovered, lmb_down);
                    let cp = ResolvedLayout { x: layout.x, y: layout.y, w: layout.w, h: layout.h, align_x: *align_x, align_y: *align_y };
                    Self::render_list(children, cp, vw, vh, layout_cache, click_cache, release_cache, texture_cache, physics_2d_by_id, physics_2d_by_class, physics_3d_by_id, physics_3d_by_class, effective_scale, frame_dt);
                }

                UIComponent::PhysicsContainer2D { params, engine, children } => {
                    components::view::draw(layout, params.bg_color);

                    let context = PhysicsRenderContext2D {
                        id: params.id.clone(),
                        class_name: params.class_name.clone(),
                        layout,
                        engine: *engine,
                        visible: params.visible,
                        z_order: params.z_order,
                    };

                    if !params.id.is_empty() {
                        if let Some(handler) = physics_2d_by_id.get_mut(&params.id) {
                            handler(context.clone());
                        }
                    }

                    if !params.class_name.is_empty() {
                        for token in params
                            .class_name
                            .split(|ch: char| ch.is_whitespace() || ch == ',')
                            .filter(|token| !token.is_empty())
                        {
                            if let Some(handler) = physics_2d_by_class.get_mut(token) {
                                handler(context.clone());
                                break;
                            }
                        }
                    }

                    let cp = ResolvedLayout {
                        x: layout.x,
                        y: layout.y,
                        w: layout.w,
                        h: layout.h,
                        ..Default::default()
                    };
                    Self::render_list(children, cp, vw, vh, layout_cache, click_cache, release_cache, texture_cache, physics_2d_by_id, physics_2d_by_class, physics_3d_by_id, physics_3d_by_class, effective_scale, frame_dt);
                }

                UIComponent::PhysicsContainer3D { params, children } => {
                    components::view::draw(layout, params.bg_color);

                    let context = PhysicsRenderContext3D {
                        id: params.id.clone(),
                        class_name: params.class_name.clone(),
                        layout,
                        visible: params.visible,
                        z_order: params.z_order,
                    };

                    if !params.id.is_empty() {
                        if let Some(handler) = physics_3d_by_id.get_mut(&params.id) {
                            handler(context.clone());
                        }
                    }

                    if !params.class_name.is_empty() {
                        for token in params
                            .class_name
                            .split(|ch: char| ch.is_whitespace() || ch == ',')
                            .filter(|token| !token.is_empty())
                        {
                            if let Some(handler) = physics_3d_by_class.get_mut(token) {
                                handler(context.clone());
                                break;
                            }
                        }
                    }

                    let cp = ResolvedLayout {
                        x: layout.x,
                        y: layout.y,
                        w: layout.w,
                        h: layout.h,
                        ..Default::default()
                    };
                    Self::render_list(children, cp, vw, vh, layout_cache, click_cache, release_cache, texture_cache, physics_2d_by_id, physics_2d_by_class, physics_3d_by_id, physics_3d_by_class, effective_scale, frame_dt);
                }

                UIComponent::Label { params, text, color, .. } => {
                    components::label::draw(layout, text, *color, params.font_size as f32, effective_scale);
                }

                UIComponent::Slider { params, value, min, max, color, track_color, dragging: drag, changed, .. } => {
                    components::slider::draw(layout, value, *min, *max, *color, *track_color, drag, changed, hovered, lmb_pressed, lmb_down, mx);
                    let _ = params;
                }

                UIComponent::Switch { params, checked, color, .. } => {
                    components::switch::draw(layout, checked, *color, hovered, lmb_pressed);
                    let _ = params;
                }

                UIComponent::TextField { params, text, placeholder, color, focused: foc, .. } => {
                    components::text_field::draw(layout, params.font_size as f32 * effective_scale, text, placeholder.as_str(), *color, foc, hovered, lmb_pressed);
                }

                UIComponent::ProgressBar { params: _, value, max, color, .. } => {
                    components::progress_bar::draw(layout, *value, *max, *color);
                }

                UIComponent::Texture { params, src, object_fit } => {
                    components::texture::draw(layout, src.as_str(), *object_fit, texture_cache, params.bg_color);
                }

                UIComponent::AnimatedTexture { params, src, fps, loop_anim, opacity, object_fit,
                                               frame_count, timing, playing, elapsed, current_frame, .. } => {
                    if *playing && *frame_count > 0 {
                        *elapsed += frame_dt;
                        let total_dur = *frame_count as f32 / fps.max(0.001);
                        let raw_t = if *loop_anim {
                            (*elapsed % total_dur) / total_dur
                        } else {
                            (*elapsed / total_dur).min(1.0)
                        };
                        let eased = apply_easing(raw_t, TransitionEasing::CubicBezier(*timing)).clamp(0.0, 1.0);
                        *current_frame = ((eased * *frame_count as f32) as u32)
                            .min(frame_count.saturating_sub(1));
                    }
                    let frame_key = format!("{}/frame_{}", src, *current_frame);
                    let bg        = params.bg_color;
                    let fit       = *object_fit;
                    let op        = *opacity;
                    components::animated_texture::draw(layout, &frame_key, fit, op, texture_cache, bg);
                }

                UIComponent::Rect { params, border_color, border_width, corner_radius } => {
                    components::rect::draw(layout, params.bg_color, *border_color, *border_width, *corner_radius);
                }

                UIComponent::Gradient { params: _, gradient_type, angle, color1, color2 } => {
                    components::gradient::draw(layout, *gradient_type, *angle, *color1, *color2);
                }
            }
        }
    }

    // * -- Resolve a component's absolute pixel bounds from parent context -- *
    fn resolve_layout(component: &UIComponent, parent: &ResolvedLayout, vw: f32, vh: f32, effective_scale: f32) -> ResolvedLayout {
        let p = component.params();
        // Labels measure at the accumulated (parent * self) scale so auto-sizing matches the drawn size.
        let w_raw = match component {
            UIComponent::Label { text, params, .. } if matches!(params.width, SizeValue::Auto) => {
                let sf = params.font_size as f32 * effective_scale;
                measure_text(text, None, sf as u16, 1.0).width.max(1.0)
            }
            _ => p.width.resolve(parent.w, vw, vh),
        };
        let h_raw = match component {
            UIComponent::Label { text, params, .. } if matches!(params.height, SizeValue::Auto) => {
                let sf = params.font_size as f32 * effective_scale;
                measure_text(text, None, sf as u16, 1.0).height.max(1.0)
            }
            _ => p.height.resolve(parent.h, vw, vh),
        };
        let w = w_raw * p.scale;
        let h = h_raw * p.scale;

        let x = match &p.x {
            PosValue::Px(expr) => parent.x + expr.resolve(parent.w, vw, vh),
            PosValue::Auto(expr) => {
                let offset = expr.resolve(parent.w, vw, vh);
                let aligned = match parent.align_x {
                    AlignX::Left   => parent.x,
                    AlignX::Center => parent.x + (parent.w - w) / 2.0,
                    AlignX::Right  => parent.x + parent.w - w,
                };
                aligned + offset
            }
        };

        let y = match &p.y {
            PosValue::Px(expr) => parent.y + expr.resolve(parent.h, vw, vh),
            PosValue::Auto(expr) => {
                let offset = expr.resolve(parent.h, vw, vh);
                let aligned = match parent.align_y {
                    AlignY::Top    => parent.y,
                    AlignY::Center => parent.y + (parent.h - h) / 2.0,
                    AlignY::Bottom => parent.y + parent.h - h,
                };
                aligned + offset
            }
        };

        ResolvedLayout { x, y, w, h, ..Default::default() }
    }

    // * -- Compute the draw rect for a texture given its natural size and object-fit mode -- *
    pub fn object_fit_rect(fit: ObjectFit, layout: ResolvedLayout, tex_w: f32, tex_h: f32) -> (f32, f32, f32, f32) {
        match fit {
            ObjectFit::Warp => (layout.x, layout.y, layout.w, layout.h),
            ObjectFit::Cover => {
                let scale = (layout.w / tex_w).max(layout.h / tex_h);
                let dw = tex_w * scale;
                let dh = tex_h * scale;
                let dx = layout.x + (layout.w - dw) / 2.0;
                let dy = layout.y + (layout.h - dh) / 2.0;
                (dx, dy, dw, dh)
            }
            ObjectFit::Contain => {
                let scale = (layout.w / tex_w).min(layout.h / tex_h);
                let dw = tex_w * scale;
                let dh = tex_h * scale;
                let dx = layout.x + (layout.w - dw) / 2.0;
                let dy = layout.y + (layout.h - dh) / 2.0;
                (dx, dy, dw, dh)
            }
        }
    }

    #[allow(dead_code)]
    fn draw_bg(layout: ResolvedLayout, color: Color) {
        if color.a > 0.0 {
            draw_rectangle(layout.x, layout.y, layout.w, layout.h, color);
        }
    }
}
