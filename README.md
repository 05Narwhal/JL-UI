# jl_UI

jl_UI is an XML-driven UI runtime for game-oriented Rust projects.

It is built around macroquad for rendering/input and provides a schema-based UI pipeline with optional audio, input helpers, and 3D integrations.

## Highlights

- XML-first UI schemas for fast iteration
- Runtime schema switching for menus and scenes
- Built-in integration with macroquad main loop patterns
- Optional feature flags for audio, input, debug logging, and 3D support via Bevy
- Custom app bootstrap attribute via jlui_init

## Installation

Add this to your Cargo.toml:

	[dependencies]
	jl_UI = "0.1.2026-dev"

Or use local path development:

	[dependencies]
	jl_UI = { path = "./jl-ui" }

## Quick Start

	use jl_UI::prelude::*;
	use macroquad::prelude::*;

	fn window_conf() -> WindowConfig {
		WindowConfig {
			window_title: "jl_UI App".to_string(),
			window_width: 1280,
			window_height: 720,
			high_dpi: true,
			..Default::default()
		}
	}

	#[jlui_init(window_conf)]
	async fn main() {
		let mut ui = UI::init();
		ui.load_schemas("ui");
		ui.use_schema("main_menu.xml");

		loop {
			clear_background(BLACK);
			ui.render();
			next_frame().await;
		}
	}

## Minimal Schema Example

	<?xml version="1.0" encoding="UTF-8"?>
	<xml>
		<ui>
			<gradient id="bg" width="100vw" height="100vh" type="line" angle="135deg" color1="#0B1E2EFF" color2="#132E39FF" />

			<view id="menu_panel" x="50% - 220px" y="50% - 180px" width="440px" height="360px" bgColor="#0E141FCC" alignX="center" alignY="top">
				<label id="title" text="jl_UI Example" color="#F3F9FFFF" x="$" y="24px" fontSize="44" />
				<button id="btn_start" text="Start" x="$" y="156px" width="290px" height="56px" bgColor="#1F6AA5FF" color="#F4FBFFFF" fontSize="24" />
			</view>
		</ui>
	</xml>

## Features

- default: debug
- debug: chrono, log, env_logger, colored
- audio: audio loader helpers
- input: keyboard input helpers
- ui-2d: 2D UI pathway
- ui-3d: enables Bevy-backed 3D support

Enable feature flags as needed:

	[dependencies]
	jl_UI = { version = "0.1.2026-dev", features = ["audio", "input", "ui-3d"] }

## Running The Included Example

The workspace includes a complete demo under tests/ui_bevy_menu_example.

From repository root:

	cargo run --manifest-path tests/ui_bevy_menu_example/Cargo.toml

## License

GPL-3.0
