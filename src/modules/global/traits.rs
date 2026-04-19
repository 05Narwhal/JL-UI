use std::path::{Path, PathBuf};
use macroquad::{color::Color, miniquad::conf::Platform, prelude::Conf};

use crate::{WindowConfig, prelude::XmlVariableValue};

// ^ ==== TRAITS ==== ^
// -- Extra Path Methods --
pub trait PathTrait {
    fn display_relative(&self) -> String;
}
// -- Extra String Traits --
pub trait StringExt {
    fn to_path_buf(&self) -> PathBuf;
}

// -- Hex support for Colors --
pub trait HexSupport {
    fn to_hex(&self) -> String;
    fn from_hex(hex: &str) -> Self;
}

// -- Macroquad Config Conversion --
pub trait ToMacroquadConfig {
    fn to_macroquad_config(&self) -> Conf;
}

// ^ ==== Implementations ==== ^
// -- PathTrait for PathBuf & Path --
impl PathTrait for PathBuf {
    fn display_relative(&self) -> String {
        let child_path = &self.file_name().unwrap_or_default().to_string_lossy();
        let parent_path = &self.parent().unwrap_or_else(|| Path::new("")).file_name().unwrap_or_default().to_string_lossy();

        format!("{}{}{}", parent_path, std::path::MAIN_SEPARATOR, child_path)
    }
}

impl PathTrait for Path {
    fn display_relative(&self) -> String {
        let child_path = &self.file_name().unwrap_or_default().to_string_lossy();
        let parent_path = &self.parent().unwrap_or_else(|| Path::new("")).file_name().unwrap_or_default().to_string_lossy();

        format!("{}{}{}", parent_path, std::path::MAIN_SEPARATOR, child_path)
    }
}

// -- Hex support for Color --
impl HexSupport for Color {
    fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8
        )
    }

    fn from_hex(hex: &str) -> Self {
        if hex.eq_ignore_ascii_case("transparent") {
            return Color::new(0.0, 0.0, 0.0, 0.0);
        }
        let hex = hex.trim_start_matches('#');
        // Guard: require at least 6 valid hex digits to avoid panics on bad input.
        if hex.len() < 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Color::new(0.0, 0.0, 0.0, 0.0);
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        let a = if hex.len() >= 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
        } else {
            255
        };
        Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0
        )
    }
}

// -- StringExt for String --
impl StringExt for String {
    fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.as_str())
    }
}

// * ==== XML VARIABLE SUPPORT ==== *
// f32
impl From<f32> for XmlVariableValue {
    fn from(value: f32) -> Self {
        XmlVariableValue::NumberF32(value)
    }
}

impl TryFrom<XmlVariableValue> for f32 {
    type Error = ();

    fn try_from(value: XmlVariableValue) -> Result<Self, Self::Error> {
        match value {
            XmlVariableValue::NumberF32(n) => Ok(n),
            XmlVariableValue::NumberI32(n) => Ok(n as f32),
            XmlVariableValue::NumberUsize(n) => Ok(n as f32),
            XmlVariableValue::NumberU32(n) => Ok(n as f32),
            _ => Err(()),
        }
    }
}

// i32
impl From<i32> for XmlVariableValue {
    fn from(value: i32) -> Self {
        XmlVariableValue::NumberI32(value)
    }
}

impl TryFrom<XmlVariableValue> for i32 {
    type Error = ();

    fn try_from(value: XmlVariableValue) -> Result<Self, Self::Error> {
        match value {
            XmlVariableValue::NumberF32(n) => Ok(n as i32),
            XmlVariableValue::NumberI32(n) => Ok(n),
            XmlVariableValue::NumberUsize(n) => Ok(n as i32),
            XmlVariableValue::NumberU32(n) => Ok(n as i32),
            _ => Err(()),
        }
    }
}

// usize
impl From<usize> for XmlVariableValue {
    fn from(value: usize) -> Self {
        XmlVariableValue::NumberUsize(value)
    }
}

impl TryFrom<XmlVariableValue> for usize {
    type Error = ();

    fn try_from(value: XmlVariableValue) -> Result<Self, Self::Error> {
        match value {
            XmlVariableValue::NumberF32(n) => Ok(n as usize),
            XmlVariableValue::NumberI32(n) => Ok(n as usize),
            XmlVariableValue::NumberUsize(n) => Ok(n),
            XmlVariableValue::NumberU32(n) => Ok(n as usize),
            _ => Err(()),
        }
    }
}

// u32
impl From<u32> for XmlVariableValue {
    fn from(value: u32) -> Self {
        XmlVariableValue::NumberU32(value)
    }
}

impl TryFrom<XmlVariableValue> for u32 {
    type Error = ();

    fn try_from(value: XmlVariableValue) -> Result<Self, Self::Error> {
        match value {
            XmlVariableValue::NumberF32(n) => Ok(n as u32),
            XmlVariableValue::NumberI32(n) => Ok(n as u32),
            XmlVariableValue::NumberUsize(n) => Ok(n as u32),
            XmlVariableValue::NumberU32(n) => Ok(n),
            _ => Err(()),
        }
    }
}

// Color
impl From<Color> for XmlVariableValue {
    fn from(value: Color) -> Self {
        XmlVariableValue::Color(value)
    }
}

impl TryFrom<XmlVariableValue> for Color {
    type Error = ();

    fn try_from(value: XmlVariableValue) -> Result<Self, Self::Error> {
        match value {
            XmlVariableValue::Color(c) => Ok(c),
            _ => Err(()),
        }
    }
}

// String
impl From<String> for XmlVariableValue {
    fn from(value: String) -> Self {
        XmlVariableValue::String(value)
    }
}

impl TryFrom<XmlVariableValue> for String {
    type Error = ();

    fn try_from(value: XmlVariableValue) -> Result<Self, Self::Error> {
        match value {
            XmlVariableValue::String(s) => Ok(s),
            _ => Err(()),
        }
    }
}

// &str
impl From<&str> for XmlVariableValue {
    fn from(value: &str) -> Self {
        XmlVariableValue::String(value.to_string())
    }
}

impl TryFrom<XmlVariableValue> for &str {
    type Error = ();

    fn try_from(value: XmlVariableValue) -> Result<Self, Self::Error> {
        match value {
            XmlVariableValue::String(s) => Ok(Box::leak(s.into_boxed_str())),
            _ => Err(()),
        }
    }
}

// bool
impl From<bool> for XmlVariableValue {
    fn from(value: bool) -> Self {
        XmlVariableValue::Bool(value)
    }
}

impl TryFrom<XmlVariableValue> for bool {
    type Error = ();

    fn try_from(value: XmlVariableValue) -> Result<Self, Self::Error> {
        match value {
            XmlVariableValue::Bool(b) => Ok(b),
            _ => Err(()),
        }
    }
}

// * ==== Macroquad Config Conversion ==== *
impl ToMacroquadConfig for WindowConfig {
    fn to_macroquad_config(&self) -> Conf {
        Conf {
            window_title: self.window_title.clone(),
            window_width: self.window_width,
            window_height: self.window_height,
            high_dpi: self.high_dpi,
            fullscreen: self.fullscreen,
            sample_count: self.sample_count,
            window_resizable: self.window_resizable,
            icon: self.icon.clone(),
            platform: self.platform.clone(),
        }
    }
}

impl From<WindowConfig> for Conf {
    fn from(config: WindowConfig) -> Self {
        config.to_macroquad_config()
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            window_title: String::new(),
            window_width: 800,
            window_height: 600,
            high_dpi: false,
            fullscreen: false,
            sample_count: 1,
            window_resizable: false,
            icon: None,
            platform: Platform::default(),
        }
    }
}