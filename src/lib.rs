//! `jl_UI` is an XML-driven UI runtime for game-oriented Rust projects.
//!
//! This crate intentionally exposes a compact public surface for library consumers,
//! while keeping most parser/renderer internals private behind stable wrapper APIs.
/*
Copyright (C) 2026 Julian Loy

This file is part of jl_ui.

jl_ui is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License v3.
*/

// * ======== MODULES ======== *
pub mod prelude;
pub mod utils;
mod modules;

// * ======== FEATURE-GATED UI LOGGING ======== *
// These macros keep all UI runtime logging controllable through the `debug` feature.
// When `debug` is disabled they compile to no-ops and do not affect host logging.
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! ui_trace {
	($($arg:tt)*) => {
		log::trace!($($arg)*)
	};
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! ui_trace {
	($($arg:tt)*) => {
		()
	};
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! ui_error {
	($($arg:tt)*) => {
		log::error!($($arg)*)
	};
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! ui_error {
	($($arg:tt)*) => {
		()
	};
}

// #[macroquad::main(window_conf)]

// * ======== CURATED PUBLIC EXPORTS ======== *
pub use modules::ui::lib::{ElementParams, LoadedSchema, UI, UIHandle, UIHandleMut};
pub use modules::ui::parser::XmlVariableValue;
pub use modules::ui::types::ui::{
	AlignX, AlignY, BasicParams, GradientType, ObjectFit, Physics2DEngine,
	PhysicsRenderContext2D, PhysicsRenderContext3D, PosValue, ResolvedLayout, SizeExpr,
	SizeOp, SizeValue, TransitionConfig, TransitionEasing, UIComponent,
};
pub use modules::audio::loader::*;
pub use modules::input::keyboard::*;
pub use modules::global::types::WindowConfig;
pub use modules::global::traits::*;
pub use jl_ui_procmacro::jlui_init;

