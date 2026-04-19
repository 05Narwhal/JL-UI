// * =========== MODULE: UI -> PARSER =========== *
// * =========== IMPORTS =========== *
use std::fs;
use quick_xml::events::Event;
use quick_xml::Reader;
use macroquad::color::Color;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::modules::global::traits::HexSupport;
use crate::modules::ui::types::ui::{
    AlignX, AlignY, BasicParams, GradientType, ObjectFit, PosValue, SizeExpr, SizeOp, SizeValue,
    Physics2DEngine, TransitionConfig, TransitionEasing, UIComponent, VALID_TAGS,
};

#[cfg(feature = "debug")]
macro_rules! debug {
    ($($arg:tt)*) => { log::trace!($($arg)*) };
}
#[cfg(feature = "debug")]
macro_rules! info {
    ($($arg:tt)*) => { log::trace!($($arg)*) };
}
#[cfg(feature = "debug")]
macro_rules! warn {
    ($($arg:tt)*) => { log::trace!($($arg)*) };
}
#[cfg(feature = "debug")]
macro_rules! error {
    ($($arg:tt)*) => { log::error!($($arg)*) };
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($($arg:tt)*) => { () };
}
#[cfg(not(feature = "debug"))]
macro_rules! info {
    ($($arg:tt)*) => { () };
}
#[cfg(not(feature = "debug"))]
macro_rules! warn {
    ($($arg:tt)*) => { () };
}
#[cfg(not(feature = "debug"))]
macro_rules! error {
    ($($arg:tt)*) => { () };
}

#[derive(Debug, Clone, PartialEq)]
pub enum XmlVariableValue {
    NumberF32(f32),
    NumberI32(i32),
    NumberUsize(usize),
    NumberU32(u32),
    Color(Color),
    String(String),
    Bool(bool),
}

// * =========== VARIABLE STORE =========== *
// Tracks <var> (mutable) and <const> (immutable) entries from the <def> block.
// Numeric values are stored as resolved f32 (px at parse time).
// Color values are stored as macroquad Color alongside a separate mutability map.
#[derive(Debug, Clone)]
pub struct VarStore {
    f32_values:         HashMap<String, f32>,
    i32_values:         HashMap<String, i32>,
    usize_values:       HashMap<String, usize>,
    u32_values:         HashMap<String, u32>,
    mutable:            HashMap<String, bool>,
    color_values:       HashMap<String, Color>,
    color_mutable:      HashMap<String, bool>,
    string_values:      HashMap<String, String>,
    string_mutable:     HashMap<String, bool>,
    bool_values:        HashMap<String, bool>,
    bool_mutable:   HashMap<String, bool>,
}

impl VarStore {
    pub fn new() -> Self {
        Self {
            f32_values:         HashMap::new(),
            i32_values:         HashMap::new(),
            usize_values:       HashMap::new(),
            u32_values:         HashMap::new(),
            mutable:            HashMap::new(),
            color_values:       HashMap::new(),
            color_mutable:      HashMap::new(),
            string_values:  HashMap::new(),
            string_mutable: HashMap::new(),
            bool_values:    HashMap::new(),
            bool_mutable:   HashMap::new(),
        }
    }

    // ---- numeric vars ----
    pub fn define_f32_var(&mut self, name: &str, value: f32) {
        self.f32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), true);
    }

    pub fn define_f32_const(&mut self, name: &str, value: f32) {
        self.f32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), false);
    }

    pub fn get_f32_var(&self, name: &str) -> Option<f32> {
        self.f32_values.get(name).copied()
    }

    pub fn define_i32_var(&mut self, name: &str, value: i32) {
        self.i32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), true);
    }

    pub fn define_i32_const(&mut self, name: &str, value: i32) {
        self.i32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), false);
    }

    pub fn get_i32_var(&self, name: &str) -> Option<i32> {
        self.i32_values.get(name).copied()
    }

    pub fn define_usize_var(&mut self, name: &str, value: usize) {
        self.usize_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), true);
    }

    pub fn define_usize_const(&mut self, name: &str, value: usize) {
        self.usize_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), false);
    }

    pub fn get_usize_var(&self, name: &str) -> Option<usize> {
        self.usize_values.get(name).copied()
    }

    pub fn define_u32_var(&mut self, name: &str, value: u32) {
        self.u32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), true);
    }

    pub fn define_u32_const(&mut self, name: &str, value: u32) {
        self.u32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), false);
    }

    pub fn get_u32_var(&self, name: &str) -> Option<u32> {
        self.u32_values.get(name).copied()
    }

    pub fn has_value(&self, name: &str) -> bool {
        self.f32_values.contains_key(name)
            || self.i32_values.contains_key(name)
            || self.usize_values.contains_key(name)
            || self.u32_values.contains_key(name)
            || self.color_values.contains_key(name)
            || self.string_values.contains_key(name)
            || self.bool_values.contains_key(name)
    }

    pub fn get_value(&self, name: &str) -> Option<XmlVariableValue> {
        if let Some(value) = self.f32_values.get(name).copied() {
            return Some(XmlVariableValue::NumberF32(value));
        }
        if let Some(value) = self.i32_values.get(name).copied() {
            return Some(XmlVariableValue::NumberI32(value));
        }
        if let Some(value) = self.usize_values.get(name).copied() {
            return Some(XmlVariableValue::NumberUsize(value));
        }
        if let Some(value) = self.u32_values.get(name).copied() {
            return Some(XmlVariableValue::NumberU32(value));
        }
        if let Some(color) = self.color_values.get(name).copied() {
            return Some(XmlVariableValue::Color(color));
        }
        if let Some(value) = self.string_values.get(name).cloned() {
            return Some(XmlVariableValue::String(value));
        }
        if let Some(value) = self.bool_values.get(name).copied() {
            return Some(XmlVariableValue::Bool(value));
        }
        None
    }

    pub fn set_value(&mut self, name: &str, value: XmlVariableValue) -> bool {
        match value {
            XmlVariableValue::NumberF32(v) => {
                if !self.is_mutable(name) || !self.f32_values.contains_key(name) {
                    return false;
                }
                self.f32_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::NumberI32(v) => {
                if !self.is_mutable(name) || !self.i32_values.contains_key(name) {
                    return false;
                }
                self.i32_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::NumberUsize(v) => {
                if !self.is_mutable(name) || !self.usize_values.contains_key(name) {
                    return false;
                }
                self.usize_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::NumberU32(v) => {
                if !self.is_mutable(name) || !self.u32_values.contains_key(name) {
                    return false;
                }
                self.u32_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::Color(v) => {
                if !self.is_color_mutable(name) || !self.color_values.contains_key(name) {
                    return false;
                }
                self.color_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::String(v) => {
                if !self.is_string_mutable(name) || !self.string_values.contains_key(name) {
                    return false;
                }
                self.string_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::Bool(v) => {
                if !self.is_bool_mutable(name) || !self.bool_values.contains_key(name) {
                    return false;
                }
                self.bool_values.insert(name.to_string(), v);
                true
            }
        }
    }

    pub fn is_mutable(&self, name: &str) -> bool {
        self.mutable.get(name).copied().unwrap_or(false)
    }

    // ---- color vars ----
    pub fn define_color_var(&mut self, name: &str, color: Color) {
        self.color_values.insert(name.to_string(), color);
        self.color_mutable.insert(name.to_string(), true);
    }

    pub fn define_color_const(&mut self, name: &str, color: Color) {
        self.color_values.insert(name.to_string(), color);
        self.color_mutable.insert(name.to_string(), false);
    }

    pub fn get_color_var(&self, name: &str) -> Option<Color> {
        self.color_values.get(name).copied()
    }

    pub fn is_color_mutable(&self, name: &str) -> bool {
        self.color_mutable.get(name).copied().unwrap_or(false)
    }

    // ---- force-set (bypasses const protection; used by the "==" operator) ----
    pub fn force_set_f32(&mut self, name: &str, value: f32) {
        self.f32_values.insert(name.to_string(), value);
        self.mutable.entry(name.to_string()).or_insert(true);
    }

    pub fn force_set_i32(&mut self, name: &str, value: i32) {
        self.i32_values.insert(name.to_string(), value);
        self.mutable.entry(name.to_string()).or_insert(true);
    }

    pub fn force_set_usize(&mut self, name: &str, value: usize) {
        self.usize_values.insert(name.to_string(), value);
        self.mutable.entry(name.to_string()).or_insert(true);
    }

    pub fn force_set_u32(&mut self, name: &str, value: u32) {
        self.u32_values.insert(name.to_string(), value);
        self.mutable.entry(name.to_string()).or_insert(true);
    }

    pub fn force_set_color(&mut self, name: &str, color: Color) {
        self.color_values.insert(name.to_string(), color);
        self.color_mutable.entry(name.to_string()).or_insert(true);
    }

    // ---- string/path vars ----
    pub fn define_string_var(&mut self, name: &str, value: &str) {
        self.string_values.insert(name.to_string(), value.to_string());
        self.string_mutable.insert(name.to_string(), true);
    }

    pub fn define_string_const(&mut self, name: &str, value: &str) {
        self.string_values.insert(name.to_string(), value.to_string());
        self.string_mutable.insert(name.to_string(), false);
    }

    pub fn get_string_var(&self, name: &str) -> Option<&str> {
        self.string_values.get(name).map(|s| s.as_str())
    }

    pub fn is_string_mutable(&self, name: &str) -> bool {
        self.string_mutable.get(name).copied().unwrap_or(false)
    }

    pub fn force_set_string(&mut self, name: &str, value: &str) {
        self.string_values.insert(name.to_string(), value.to_string());
        self.string_mutable.entry(name.to_string()).or_insert(true);
    }

    // ---- bool vars ----
    pub fn define_bool_var(&mut self, name: &str, value: bool) {
        self.bool_values.insert(name.to_string(), value);
        self.bool_mutable.insert(name.to_string(), true);
    }

    pub fn define_bool_const(&mut self, name: &str, value: bool) {
        self.bool_values.insert(name.to_string(), value);
        self.bool_mutable.insert(name.to_string(), false);
    }

    pub fn get_bool_var(&self, name: &str) -> Option<bool> {
        self.bool_values.get(name).copied()
    }

    pub fn is_bool_mutable(&self, name: &str) -> bool {
        self.bool_mutable.get(name).copied().unwrap_or(false)
    }

    pub fn force_set_bool(&mut self, name: &str, value: bool) {
        self.bool_values.insert(name.to_string(), value);
        self.bool_mutable.entry(name.to_string()).or_insert(true);
    }

    // ---- augmented numeric assignment ----
    // Returns true on success, logs a warning on failure.
    pub fn augmented_assign(&mut self, name: &str, op: &str, rhs: f32) -> bool {
        if !self.is_mutable(name) {
            warn!("[UI::Parser] Cannot mutate const '{}'", name);
            return false;
        }
        let cur = self.f32_values.entry(name.to_string()).or_insert(0.0);
        *cur = match op {
            "+=" => *cur + rhs,
            "-=" => *cur - rhs,
            "*=" => *cur * rhs,
            "/=" => if rhs.abs() > f32::EPSILON { *cur / rhs } else { 0.0 },
            "%=" => *cur % rhs,
            _ => { warn!("[UI::Parser] Unknown augmented op '{}'", op); return false; }
        };
        true
    }

    // Merge variables from an included schema into this scope.
    fn absorb_from(&mut self, other: &VarStore) {
        self.f32_values.extend(other.f32_values.clone());
        self.mutable.extend(other.mutable.clone());
        self.color_values.extend(other.color_values.clone());
        self.color_mutable.extend(other.color_mutable.clone());
        self.string_values.extend(other.string_values.clone());
        self.string_mutable.extend(other.string_mutable.clone());
        self.bool_values.extend(other.bool_values.clone());
        self.bool_mutable.extend(other.bool_mutable.clone());
    }
}

// * =========== GLOBAL VARIABLE STORE =========== *
// Persists across multiple XML file loads.  Any <var>/<const> in a <def> block that
// carries `global="true"` is registered here and becomes visible in all subsequently-
// parsed files without re-declaration.
//
// Redefinition rules (enforced inside parse_def_block):
//   • global const re-declared in any file  →  Err (parse error)
//   • global var  re-declared in any file   →  Ok  (var is mutable; value is updated)
#[derive(Debug, Clone)]
pub struct GlobalVarStore {
    f32_values:         HashMap<String, f32>,
    i32_values:         HashMap<String, i32>,
    usize_values:       HashMap<String, usize>,
    u32_values:         HashMap<String, u32>,
    mutable:        HashMap<String, bool>,   // false = const, true = var
    color_values:   HashMap<String, Color>,
    color_mutable:  HashMap<String, bool>,
    string_values:  HashMap<String, String>,
    string_mutable: HashMap<String, bool>,
    bool_values:    HashMap<String, bool>,
    bool_mutable:   HashMap<String, bool>,
}

impl GlobalVarStore {
    pub fn new() -> Self {
        Self {
            f32_values:         HashMap::new(),
            i32_values:         HashMap::new(),
            usize_values:       HashMap::new(),
            u32_values:         HashMap::new(),
            mutable:        HashMap::new(),
            color_values:   HashMap::new(),
            color_mutable:  HashMap::new(),
            string_values:  HashMap::new(),
            string_mutable: HashMap::new(),
            bool_values:    HashMap::new(),
            bool_mutable:   HashMap::new(),
        }
    }

    // True if `name` has been registered under any type category.
    fn is_defined(&self, name: &str) -> bool {
        self.mutable.contains_key(name)
            || self.color_mutable.contains_key(name)
            || self.string_mutable.contains_key(name)
            || self.bool_mutable.contains_key(name)
    }

    // ---- numeric ----
    fn define_f32(&mut self, name: &str, value: f32, is_mutable: bool) {
        self.f32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), is_mutable);
    }
    fn get_f32(&self, name: &str) -> Option<f32> { self.f32_values.get(name).copied() }

    fn define_i32(&mut self, name: &str, value: i32, is_mutable: bool) {
        self.i32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), is_mutable);
    }
    fn get_i32(&self, name: &str) -> Option<i32> { self.i32_values.get(name).copied() }

    fn define_usize(&mut self, name: &str, value: usize, is_mutable: bool) {
        self.usize_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), is_mutable);
    }
    fn get_usize(&self, name: &str) -> Option<usize> { self.usize_values.get(name).copied() }

    fn define_u32(&mut self, name: &str, value: u32, is_mutable: bool) {
        self.u32_values.insert(name.to_string(), value);
        self.mutable.insert(name.to_string(), is_mutable);
    }
    fn get_u32(&self, name: &str) -> Option<u32> { self.u32_values.get(name).copied() }

    pub fn has_value(&self, name: &str) -> bool {
        self.f32_values.contains_key(name)
            || self.i32_values.contains_key(name)
            || self.usize_values.contains_key(name)
            || self.u32_values.contains_key(name)
            || self.color_values.contains_key(name)
            || self.string_values.contains_key(name)
            || self.bool_values.contains_key(name)
    }

    pub fn get_value(&self, name: &str) -> Option<XmlVariableValue> {
        if let Some(value) = self.f32_values.get(name).copied() {
            return Some(XmlVariableValue::NumberF32(value));
        }
        if let Some(value) = self.i32_values.get(name).copied() {
            return Some(XmlVariableValue::NumberI32(value));
        }
        if let Some(value) = self.usize_values.get(name).copied() {
            return Some(XmlVariableValue::NumberUsize(value));
        }
        if let Some(value) = self.u32_values.get(name).copied() {
            return Some(XmlVariableValue::NumberU32(value));
        }
        if let Some(color) = self.color_values.get(name).copied() {
            return Some(XmlVariableValue::Color(color));
        }
        if let Some(value) = self.string_values.get(name).cloned() {
            return Some(XmlVariableValue::String(value));
        }
        if let Some(value) = self.bool_values.get(name).copied() {
            return Some(XmlVariableValue::Bool(value));
        }
        None
    }

    pub fn set_value(&mut self, name: &str, value: XmlVariableValue) -> bool {
        match value {
            XmlVariableValue::NumberF32(v) => {
                if !self.mutable.get(name).copied().unwrap_or(false) || !self.f32_values.contains_key(name) {
                    return false;
                }
                self.f32_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::NumberI32(v) => {
                if !self.mutable.get(name).copied().unwrap_or(false) || !self.i32_values.contains_key(name) {
                    return false;
                }
                self.i32_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::NumberUsize(v) => {
                if !self.mutable.get(name).copied().unwrap_or(false) || !self.usize_values.contains_key(name) {
                    return false;
                }
                self.usize_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::NumberU32(v) => {
                if !self.mutable.get(name).copied().unwrap_or(false) || !self.u32_values.contains_key(name) {
                    return false;
                }
                self.u32_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::Color(v) => {
                if !self.color_mutable.get(name).copied().unwrap_or(false) || !self.color_values.contains_key(name) {
                    return false;
                }
                self.color_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::String(v) => {
                if !self.string_mutable.get(name).copied().unwrap_or(false) || !self.string_values.contains_key(name) {
                    return false;
                }
                self.string_values.insert(name.to_string(), v);
                true
            }
            XmlVariableValue::Bool(v) => {
                if !self.bool_mutable.get(name).copied().unwrap_or(false) || !self.bool_values.contains_key(name) {
                    return false;
                }
                self.bool_values.insert(name.to_string(), v);
                true
            }
        }
    }

    // ---- color ----
    fn define_color(&mut self, name: &str, color: Color, is_mutable: bool) {
        self.color_values.insert(name.to_string(), color);
        self.color_mutable.insert(name.to_string(), is_mutable);
    }
    fn get_color(&self, name: &str) -> Option<Color> { self.color_values.get(name).copied() }

    // ---- string / path ----
    fn define_string(&mut self, name: &str, value: &str, is_mutable: bool) {
        self.string_values.insert(name.to_string(), value.to_string());
        self.string_mutable.insert(name.to_string(), is_mutable);
    }
    fn get_string(&self, name: &str) -> Option<&str> {
        self.string_values.get(name).map(|s| s.as_str())
    }

    // ---- bool ----
    fn define_bool(&mut self, name: &str, value: bool, is_mutable: bool) {
        self.bool_values.insert(name.to_string(), value);
        self.bool_mutable.insert(name.to_string(), is_mutable);
    }
    fn get_bool(&self, name: &str) -> Option<bool> {
        self.bool_values.get(name).copied()
    }
}

fn parse_bool_literal(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

// Convert a macroquad Color back to an #RRGGBBAA hex string.
fn color_to_hex(c: Color) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}",
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
        (c.a * 255.0).round() as u8)
}

// * =========== MATH EXPRESSION PARSER =========== *
// Tokeniser
#[derive(Debug)]
enum Token {
    Num(f32, SizeUnit),
    NegNum(f32, SizeUnit), // produced only when '-' immediately precedes a number (negative px)
    Plus, Minus, Star, Slash, LParen, RParen,
}

#[derive(Debug, Clone, Copy)]
enum SizeUnit { Px, Pct, Vw, Vh, Deg }

// Expand "$var_name" references in a raw string using the VarStore (file-local) and,
// as a fallback, the GlobalVarStore (cross-file).
// Each occurrence of "$identifier" is replaced with the resolved value.
// The plain "$" (not followed by a letter) is NOT touched — it stays for the pos parser.
fn expand_vars(s: &str, vars: &VarStore, globals: &GlobalVarStore) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() && (chars[i + 1].is_alphabetic() || chars[i + 1] == '_') {
            // consume identifier
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_' || chars[end] == '-') {
                end += 1;
            }
            let name: String = chars[start..end].iter().collect();
            match vars.get_f32_var(&name) {
                Some(v) => {
                    // Use integer formatting for whole numbers, 4dp otherwise
                    if v.fract() == 0.0 {
                        out.push_str(&format!("{}px", v as i64));
                    } else {
                        out.push_str(&format!("{:.4}px", v));
                    }
                }
                None => match vars.get_color_var(&name) {
                    Some(c) => { out.push_str(&color_to_hex(c)); }
                    None => match vars.get_string_var(&name) {
                        Some(s) => { out.push_str(s); }
                        None => match vars.get_bool_var(&name) {
                            Some(v) => {
                                out.push_str(if v { "true" } else { "false" });
                            }
                            None => {
                                // Fall back to GlobalVarStore
                                if let Some(v) = globals.get_f32(&name) {
                                    if v.fract() == 0.0 {
                                        out.push_str(&format!("{}px", v as i64));
                                    } else {
                                        out.push_str(&format!("{:.4}px", v));
                                    }
                                } else if let Some(c) = globals.get_color(&name) {
                                    out.push_str(&color_to_hex(c));
                                } else if let Some(s) = globals.get_string(&name) {
                                    out.push_str(s);
                                } else if let Some(v) = globals.get_bool(&name) {
                                    out.push_str(if v { "true" } else { "false" });
                                } else {
                                    warn!("[UI::Parser] Undefined variable '{}'", name);
                                    out.push_str("0px");
                                }
                            }
                        },
                    },
                },
            }
            i = end;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn tokenize(s: &str) -> Vec<Token> {
    let chars: Vec<char> = s.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => { i += 1; }
            '+' => { tokens.push(Token::Plus);   i += 1; }
            '*' => { tokens.push(Token::Star);   i += 1; }
            '/' => { tokens.push(Token::Slash);  i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            '-' => {
                // Peek: if previous token is an operator/lparen or there is no previous token,
                // this is a unary/negative sign and the next chars form a number → NegNum.
                let is_unary = matches!(
                    tokens.last(),
                    None | Some(Token::Plus) | Some(Token::Minus)
                        | Some(Token::Star) | Some(Token::Slash) | Some(Token::LParen)
                );
                if is_unary && i + 1 < chars.len() && (chars[i + 1].is_ascii_digit() || chars[i + 1] == '.') {
                    i += 1; // skip '-'
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let num: f32 = chars[start..i].iter().collect::<String>().parse().unwrap_or(0.0);
                    let unit = parse_unit(&chars, &mut i);
                    tokens.push(Token::NegNum(num, unit));
                } else {
                    tokens.push(Token::Minus);
                    i += 1;
                }
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num: f32 = chars[start..i].iter().collect::<String>().parse().unwrap_or(0.0);
                let unit = parse_unit(&chars, &mut i);
                tokens.push(Token::Num(num, unit));
            }
            _ => { i += 1; } // skip unknown characters
        }
    }
    tokens
}

fn parse_unit(chars: &[char], i: &mut usize) -> SizeUnit {
    if *i + 2 <= chars.len() && chars[*i] == 'v' && chars[*i + 1] == 'w' {
        *i += 2; SizeUnit::Vw
    } else if *i + 2 <= chars.len() && chars[*i] == 'v' && chars[*i + 1] == 'h' {
        *i += 2; SizeUnit::Vh
    } else if *i + 2 <= chars.len() && chars[*i] == 'p' && chars[*i + 1] == 'x' {
        *i += 2; SizeUnit::Px
    } else if *i + 3 <= chars.len() && chars[*i] == 'd' && chars[*i + 1] == 'e' && chars[*i + 2] == 'g' {
        *i += 3; SizeUnit::Deg
    } else if *i < chars.len() && chars[*i] == '%' {
        *i += 1; SizeUnit::Pct
    } else {
        SizeUnit::Px  // bare integer → px
    }
}

// Recursive-descent parser: expr > term > factor
// Precedence (low → high): + - >> * / >> unary/neg/parens
fn parse_expr(tokens: &[Token], pos: usize) -> (SizeExpr, usize) {
    let (mut left, mut pos) = parse_term(tokens, pos);
    loop {
        match tokens.get(pos) {
            Some(Token::Plus)  => { let (r, p) = parse_term(tokens, pos + 1); left = SizeExpr::BinOp(Box::new(left), SizeOp::Add, Box::new(r)); pos = p; }
            Some(Token::Minus) => { let (r, p) = parse_term(tokens, pos + 1); left = SizeExpr::BinOp(Box::new(left), SizeOp::Sub, Box::new(r)); pos = p; }
            _ => break,
        }
    }
    (left, pos)
}

fn parse_term(tokens: &[Token], pos: usize) -> (SizeExpr, usize) {
    let (mut left, mut pos) = parse_factor(tokens, pos);
    loop {
        match tokens.get(pos) {
            Some(Token::Star)  => { let (r, p) = parse_factor(tokens, pos + 1); left = SizeExpr::BinOp(Box::new(left), SizeOp::Mul, Box::new(r)); pos = p; }
            Some(Token::Slash) => { let (r, p) = parse_factor(tokens, pos + 1); left = SizeExpr::BinOp(Box::new(left), SizeOp::Div, Box::new(r)); pos = p; }
            _ => break,
        }
    }
    (left, pos)
}

fn parse_factor(tokens: &[Token], pos: usize) -> (SizeExpr, usize) {
    match tokens.get(pos) {
        Some(Token::LParen) => {
            let (expr, p) = parse_expr(tokens, pos + 1);
            let p = if matches!(tokens.get(p), Some(Token::RParen)) { p + 1 } else { p };
            (expr, p)
        }
        Some(Token::Minus) => {
            // unary minus on a sub-expression
            let (inner, p) = parse_factor(tokens, pos + 1);
            (SizeExpr::BinOp(Box::new(SizeExpr::Px(0.0)), SizeOp::Sub, Box::new(inner)), p)
        }
        Some(Token::Plus) => parse_factor(tokens, pos + 1),
        Some(Token::NegNum(v, unit)) => {
            // -Npx means "parent_dim - N"
            let expr = match unit {
                SizeUnit::Px  => SizeExpr::NegPx(*v),
                // For non-px negative literals fall back to 0 - value
                SizeUnit::Pct => SizeExpr::BinOp(Box::new(SizeExpr::Px(0.0)), SizeOp::Sub, Box::new(SizeExpr::Pct(*v))),
                SizeUnit::Vw  => SizeExpr::BinOp(Box::new(SizeExpr::Px(0.0)), SizeOp::Sub, Box::new(SizeExpr::Vw(*v))),
                SizeUnit::Vh  => SizeExpr::BinOp(Box::new(SizeExpr::Px(0.0)), SizeOp::Sub, Box::new(SizeExpr::Vh(*v))),
                SizeUnit::Deg => SizeExpr::BinOp(Box::new(SizeExpr::Px(0.0)), SizeOp::Sub, Box::new(SizeExpr::Deg(*v))),
            };
            (expr, pos + 1)
        }
        Some(Token::Num(v, unit)) => {
            let expr = match unit {
                SizeUnit::Px  => SizeExpr::Px(*v),
                SizeUnit::Pct => SizeExpr::Pct(*v),
                SizeUnit::Vw  => SizeExpr::Vw(*v),
                SizeUnit::Vh  => SizeExpr::Vh(*v),
                SizeUnit::Deg => SizeExpr::Deg(*v),
            };
            (expr, pos + 1)
        }
        _ => (SizeExpr::Px(0.0), pos),
    }
}

// Resolve a simple expression string to a concrete f32 for storing in VarStore.
// Uses a dummy resolve with parent_dim=0/vw=1920/vh=1080 so "px" values are exact.
// This is only used for def-block values which should be simple px/vw/vh constants.
fn resolve_def_value(s: &str) -> f32 {
    let tokens = tokenize(s);
    let (expr, _) = parse_expr(&tokens, 0);
    expr.resolve(0.0, 1920.0, 1080.0)
}

// * =========== PARSER =========== *
pub struct UIParser;

impl UIParser {
    // * -- Load and parse an XML file into a list of root components -- *
    pub fn load(path: &str) -> Result<(Vec<UIComponent>, VarStore), String> {
        let mut globals = GlobalVarStore::new();
        Self::load_with_globals(path, &mut globals)
    }

    // * -- Pre-scan pass: read a file and collect only global="true" def entries -- *
    // Used by load_schemas() so all globals are known before any file is fully parsed.
    pub fn collect_globals_from_file(path: &str, globals: &mut GlobalVarStore) {
        let mut include_stack = Vec::new();
        Self::collect_globals_from_file_recursive(&PathBuf::from(path), globals, &mut include_stack);
    }

    fn collect_globals_from_file_recursive(path: &Path, globals: &mut GlobalVarStore, include_stack: &mut Vec<PathBuf>) {
        let canonical = match fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                warn!("[UI::Parser] Could not resolve '{}' while pre-scanning globals: {}", path.to_string_lossy(), e);
                return;
            }
        };

        if let Some(pos) = include_stack.iter().position(|p| p == &canonical) {
            let mut chain = include_stack[pos..]
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>();
            chain.push(canonical.to_string_lossy().to_string());
            warn!("[UI::Parser] Circular schema copy detected while pre-scanning globals: {}", chain.join(" -> "));
            return;
        }

        include_stack.push(canonical.clone());

        let xml = match fs::read_to_string(&canonical) {
            Ok(xml) => xml,
            Err(e) => {
                warn!("[UI::Parser] Could not pre-scan '{}' for globals: {}", canonical.to_string_lossy(), e);
                include_stack.pop();
                return;
            }
        };

        Self::collect_globals(&xml, globals);

        let base_dir = canonical.parent().unwrap_or(Path::new("."));
        for src in Self::extract_schema_sources(&xml) {
            match Self::resolve_include_path(base_dir, &src) {
                Ok(include_path) => Self::collect_globals_from_file_recursive(&include_path, globals, include_stack),
                Err(msg) => warn!("[UI::Parser] {}", msg),
            }
        }

        include_stack.pop();
    }

    // * -- Pre-scan pass: collect global="true" entries from raw XML -- *
    pub fn collect_globals(xml: &str, globals: &mut GlobalVarStore) {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "def" {
                        Self::scan_def_for_globals(&mut reader, globals);
                        return; // only the first def block matters
                    }
                }
                Ok(Event::Eof) | Err(_) => return,
                _ => {}
            }
        }
    }

    fn scan_def_for_globals(reader: &mut Reader<&[u8]>, globals: &mut GlobalVarStore) {
        let temp_vars = VarStore::new(); // no file-local vars yet during pre-scan
        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let attrs = match Self::parse_attributes(e) { Ok(a) => a, Err(_) => continue };

                    let is_global = Self::raw_attr(&attrs, "global")
                        .map(|v| v.trim().eq_ignore_ascii_case("true") || v.trim() == "1")
                        .unwrap_or(false);
                    if !is_global { continue; }

                    let name = Self::raw_attr(&attrs, "name").unwrap_or_default();
                    if globals.is_defined(&name) { continue; } // first-writer wins

                    let raw_val  = Self::raw_attr(&attrs, "value").unwrap_or_default();
                    let raw_trim = raw_val.trim();
                    let is_const = tag.as_str() == "const";

                    if raw_trim.starts_with('#') || raw_trim.eq_ignore_ascii_case("transparent") {
                        let color = <Color as HexSupport>::from_hex(raw_trim);
                        globals.define_color(&name, color, !is_const);
                    } else if let Some(flag) = parse_bool_literal(raw_trim) {
                        globals.define_bool(&name, flag, !is_const);
                    } else if raw_trim.starts_with("@/") || raw_trim.starts_with("./") || raw_trim.starts_with("../") || raw_trim.starts_with('/') {
                        let expanded = expand_vars(raw_trim, &temp_vars, globals);
                        globals.define_string(&name, &expanded, !is_const);
                    } else {
                        let expanded = expand_vars(raw_trim, &temp_vars, globals);
                        let value = resolve_def_value(&expanded);
                        globals.define_f32(&name, value, !is_const);
                    }
                    debug!("[UI::Parser] Pre-scan: registered global '{}'", name);
                }
                Ok(Event::End(ref e)) => {
                    if String::from_utf8_lossy(e.name().as_ref()) == "def" { return; }
                }
                Ok(Event::Eof) | Err(_) => return,
                _ => {}
            }
        }
    }

    // * -- Load with a shared GlobalVarStore (globals persist across calls) -- *
    pub fn load_with_globals(path: &str, globals: &mut GlobalVarStore) -> Result<(Vec<UIComponent>, VarStore), String> {
        let mut include_stack = Vec::new();
        Self::load_with_globals_recursive(path, globals, &mut include_stack)
    }

    fn load_with_globals_recursive(path: &str, globals: &mut GlobalVarStore, include_stack: &mut Vec<PathBuf>) -> Result<(Vec<UIComponent>, VarStore), String> {
        info!("[UI::Parser] Loading schema from: '{}'", path);

        let canonical = fs::canonicalize(path).map_err(|e| {
            let msg = format!("Failed to resolve schema path '{}': {}", path, e);
            error!("[UI::Parser] {}", msg);
            msg
        })?;

        if let Some(pos) = include_stack.iter().position(|p| p == &canonical) {
            let mut chain = include_stack[pos..]
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>();
            chain.push(canonical.to_string_lossy().to_string());
            let msg = format!("Circular schema copy detected: {}", chain.join(" -> "));
            error!("[UI::Parser] {}", msg);
            return Err(msg);
        }

        include_stack.push(canonical.clone());

        let xml = fs::read_to_string(&canonical).map_err(|e| {
            let msg = format!("Failed to read file '{}': {}", canonical.to_string_lossy(), e);
            error!("[UI::Parser] {}", msg);
            msg
        })?;

        info!("[UI::Parser] File read successfully ({} bytes)", xml.len());

        let base_dir = canonical.parent().unwrap_or(Path::new(".")).to_path_buf();
        let parsed = Self::parse_with_globals_in_context(&xml, globals, Some(base_dir), include_stack);
        include_stack.pop();
        parsed
    }

    // * -- Parse raw XML string -- *
    pub fn parse(xml: &str) -> Result<(Vec<UIComponent>, VarStore), String> {
        let mut globals = GlobalVarStore::new();
        Self::parse_with_globals(xml, &mut globals)
    }

    // * -- Parse raw XML string with a shared GlobalVarStore -- *
    pub fn parse_with_globals(xml: &str, globals: &mut GlobalVarStore) -> Result<(Vec<UIComponent>, VarStore), String> {
        let mut include_stack = Vec::new();
        Self::parse_with_globals_in_context(xml, globals, None, &mut include_stack)
    }

    fn parse_with_globals_in_context(
        xml: &str,
        globals: &mut GlobalVarStore,
        base_dir: Option<PathBuf>,
        include_stack: &mut Vec<PathBuf>,
    ) -> Result<(Vec<UIComponent>, VarStore), String> {
        let mut reader = Reader::from_str(xml);
        let mut vars = VarStore::new();

        debug!("[UI::Parser] Starting XML parse");

        // First pass: scan for optional <xml> wrapper, then <def>/<ui> roots.
        let components = loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag.as_str() {
                        // Transparent document wrapper — just continue iterating its children
                        "xml" => {}
                        "def" => {
                            Self::parse_def_block(&mut reader, &mut vars, globals)?;
                        }
                        "ui" | "schema" => {
                            break Self::parse_children(&mut reader, &tag, &mut vars, globals, base_dir.as_deref(), include_stack)?;
                        }
                        _ => {
                            let msg = format!("Expected <def> or <ui> at top level, found <{}>", tag);
                            error!("[UI::Parser] {}", msg);
                            return Err(msg);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    // Closing </xml> wrapper with no <ui> found yet — treat as EOF
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "xml" {
                        let msg = "Missing <ui> element inside <xml>".to_string();
                        error!("[UI::Parser] {}", msg);
                        return Err(msg);
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag != "def" {
                        let msg = format!("Unexpected self-closing <{}/> at top level", tag);
                        error!("[UI::Parser] {}", msg);
                        return Err(msg);
                    }
                }
                Ok(Event::Eof) => {
                    let msg = "Empty or missing <ui> root element".to_string();
                    error!("[UI::Parser] {}", msg);
                    return Err(msg);
                }
                Ok(_) => continue,
                Err(e) => {
                    let msg = format!("XML syntax error: {}", e);
                    error!("[UI::Parser] {}", msg);
                    return Err(msg);
                }
            }
        };

        info!("[UI::Parser] Parsed {} root component(s) successfully", components.len());
        Ok((components, vars))
    }

    // * -- Parse the <def> block: collect <var> and <const> declarations -- *
    fn parse_def_block(reader: &mut Reader<&[u8]>, vars: &mut VarStore, globals: &mut GlobalVarStore) -> Result<(), String> {
        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let attrs = Self::parse_attributes(e)?;
                    let name     = Self::raw_attr(&attrs, "name").unwrap_or_default();
                    let raw_val  = Self::raw_attr(&attrs, "value").unwrap_or_default();
                    let raw_trim = raw_val.trim();

                    let is_global = Self::raw_attr(&attrs, "global")
                        .map(|v| v.trim().eq_ignore_ascii_case("true") || v.trim() == "1")
                        .unwrap_or(false);
                    let is_const = tag.as_str() == "const";

                    // Globals were pre-populated by scan_def_for_globals() before this full
                    // parse. Write to globals only if not already there (first-writer wins);
                    // always write to local vars so $refs resolve within this file.
                    let register_in_globals = is_global && !globals.is_defined(&name);

                    // Color value (#RRGGBBAA, #RRGGBB, or "transparent")
                    if raw_trim.starts_with('#') || raw_trim.eq_ignore_ascii_case("transparent") {
                        let color = <Color as HexSupport>::from_hex(raw_trim);
                        if register_in_globals {
                            globals.define_color(&name, color, !is_const);
                        }
                        match tag.as_str() {
                            "var"   => { debug!("[UI::Parser] def color var '{}' = {}", name, raw_trim); vars.define_color_var(&name, color); }
                            "const" => { debug!("[UI::Parser] def color const '{}' = {}", name, raw_trim); vars.define_color_const(&name, color); }
                            other   => warn!("[UI::Parser] Unknown def entry: <{}>", other),
                        }
                    } else if let Some(flag) = parse_bool_literal(raw_trim) {
                        if register_in_globals {
                            globals.define_bool(&name, flag, !is_const);
                        }
                        match tag.as_str() {
                            "var"   => { debug!("[UI::Parser] def bool var '{}' = {}", name, flag); vars.define_bool_var(&name, flag); }
                            "const" => { debug!("[UI::Parser] def bool const '{}' = {}", name, flag); vars.define_bool_const(&name, flag); }
                            other   => warn!("[UI::Parser] Unknown def entry: <{}>", other),
                        }
                    } else if raw_trim.starts_with("@/") || raw_trim.starts_with("./") || raw_trim.starts_with("../") || raw_trim.starts_with('/') {
                        // Path/string value
                        let expanded = expand_vars(raw_trim, vars, globals);
                        if register_in_globals {
                            globals.define_string(&name, &expanded, !is_const);
                        }
                        match tag.as_str() {
                            "var"   => { debug!("[UI::Parser] def string var '{}' = {}", name, expanded); vars.define_string_var(&name, &expanded); }
                            "const" => { debug!("[UI::Parser] def string const '{}' = {}", name, expanded); vars.define_string_const(&name, &expanded); }
                            other   => warn!("[UI::Parser] Unknown def entry: <{}>", other),
                        }
                    } else {
                        // Numeric value — may reference earlier vars
                        let expanded = expand_vars(raw_trim, vars, globals);
                        let value = resolve_def_value(&expanded);
                        if register_in_globals {
                            globals.define_f32(&name, value, !is_const);
                        }
                        match tag.as_str() {
                            "var"   => { debug!("[UI::Parser] def var '{}' = {}", name, value); vars.define_f32_var(&name, value); }
                            "const" => { debug!("[UI::Parser] def const '{}' = {}", name, value); vars.define_f32_const(&name, value); }
                            other   => warn!("[UI::Parser] Unknown def entry: <{}>", other),
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if String::from_utf8_lossy(e.name().as_ref()) == "def" {
                        return Ok(());
                    }
                }
                Ok(Event::Eof) => return Err("Unexpected EOF inside <def>".to_string()),
                Ok(_) => {}
                Err(e) => return Err(format!("XML error in <def>: {}", e)),
            }
        }
    }

    // * -- Parse children until the closing tag is found -- *
    fn parse_children(
        reader: &mut Reader<&[u8]>,
        parent_tag: &str,
        vars: &mut VarStore,
        globals: &mut GlobalVarStore,
        base_dir: Option<&Path>,
        include_stack: &mut Vec<PathBuf>,
    ) -> Result<Vec<UIComponent>, String> {
        let mut children = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if tag == "schema" {
                        let attrs = Self::parse_attributes(e)?;
                        let included = Self::load_included_schema(&attrs, vars, globals, base_dir, include_stack)?;
                        children.extend(included);
                        Self::skip_tag(reader, "schema")?;
                        continue;
                    }

                    Self::validate_tag(&tag)?;
                    let raw_attrs = Self::parse_attributes(e)?;
                    let attrs = Self::process_attrs(raw_attrs, vars, globals);
                    debug!("[UI::Parser] Parsing <{}> (open) inside <{}>", tag, parent_tag);
                    let component = Self::build_component(&tag, &attrs, reader, vars, globals, base_dir, include_stack)?;
                    children.push(component);
                }
                Ok(Event::Empty(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if tag == "schema" {
                        let attrs = Self::parse_attributes(e)?;
                        let included = Self::load_included_schema(&attrs, vars, globals, base_dir, include_stack)?;
                        children.extend(included);
                        continue;
                    }

                    // <var name="foo" value="..." /> inside the UI tree → variable mutation
                    if tag == "var" {
                        let attrs = Self::parse_attributes(e)?;
                        let name      = Self::raw_attr(&attrs, "name").unwrap_or_default();
                        let raw_val   = Self::raw_attr(&attrs, "value").unwrap_or_default();
                        let val_trimmed = raw_val.trim();

                        // "==" prefix: force-assign regardless of const/var status
                        if let Some(rhs) = val_trimmed.strip_prefix("==") {
                            let rhs = rhs.trim();
                            if rhs.starts_with('#') {
                                let color = <Color as HexSupport>::from_hex(rhs);
                                vars.force_set_color(&name, color);
                                debug!("[UI::Parser] <var> '{}' == color {}", name, rhs);
                            } else if let Some(flag) = parse_bool_literal(rhs) {
                                vars.force_set_bool(&name, flag);
                                debug!("[UI::Parser] <var> '{}' == bool {}", name, flag);
                            } else if rhs.starts_with("@/") || rhs.starts_with("./") || rhs.starts_with("../") {
                                let expanded = expand_vars(rhs, vars, globals);
                                vars.force_set_string(&name, &expanded);
                                debug!("[UI::Parser] <var> '{}' == string {}", name, expanded);
                            } else {
                                let expanded = expand_vars(rhs, vars, globals);
                                let value = resolve_def_value(&expanded);
                                vars.force_set_f32(&name, value);
                                debug!("[UI::Parser] <var> '{}' == {}", name, value);
                            }
                            continue;
                        }

                        // Direct color assignment
                        if val_trimmed.starts_with('#') {
                            if vars.is_color_mutable(&name) {
                                let color = <Color as HexSupport>::from_hex(val_trimmed);
                                vars.define_color_var(&name, color);
                                debug!("[UI::Parser] <var> color '{}' = {}", name, val_trimmed);
                            } else {
                                warn!("[UI::Parser] <var> cannot reassign const color '{}'", name);
                            }
                            continue;
                        }

                        // Direct bool assignment
                        if let Some(flag) = parse_bool_literal(val_trimmed) {
                            if vars.is_bool_mutable(&name) {
                                vars.define_bool_var(&name, flag);
                                debug!("[UI::Parser] <var> bool '{}' = {}", name, flag);
                            } else {
                                warn!("[UI::Parser] <var> cannot reassign const bool '{}'", name);
                            }
                            continue;
                        }

                        // Direct string/path assignment (@/ ./ ../)
                        if val_trimmed.starts_with("@/") || val_trimmed.starts_with("./") || val_trimmed.starts_with("../") {
                            if vars.is_string_mutable(&name) {
                                let expanded = expand_vars(val_trimmed, vars, globals);
                                vars.define_string_var(&name, &expanded);
                                debug!("[UI::Parser] <var> string '{}' = {}", name, expanded);
                            } else {
                                warn!("[UI::Parser] <var> cannot reassign const string '{}'", name);
                            }
                            continue;
                        }

                        // Augmented numeric assignment: +=, -=, *=, /=, %=
                        let aug_op = ["*=", "/=", "%=", "+=", "-="]
                            .iter()
                            .find(|&&op| val_trimmed.starts_with(op));
                        if let Some(&op) = aug_op {
                            let rhs_str = val_trimmed[op.len()..].trim();
                            let expanded = expand_vars(rhs_str, vars, globals);
                            let rhs = resolve_def_value(&expanded);
                            if !vars.augmented_assign(&name, op, rhs) {
                                warn!("[UI::Parser] <var> mutation failed for '{}'", name);
                            } else {
                                debug!("[UI::Parser] <var> '{}' {} {}", name, op, rhs);
                            }
                        } else {
                            // Direct numeric reassignment
                            if vars.is_mutable(&name) {
                                let expanded = expand_vars(val_trimmed, vars, globals);
                                let value = resolve_def_value(&expanded);
                                vars.define_f32_var(&name, value);
                                debug!("[UI::Parser] <var> '{}' = {}", name, value);
                            } else {
                                warn!("[UI::Parser] <var> cannot reassign const '{}'", name);
                            }
                        }
                        continue; // not a UI component — do not push to children
                    }

                    Self::validate_tag(&tag)?;
                    let raw_attrs = Self::parse_attributes(e)?;
                    let attrs = Self::process_attrs(raw_attrs, vars, globals);
                    debug!("[UI::Parser] Parsing <{} /> (self-closing) inside <{}>", tag, parent_tag);
                    let component = Self::build_component_empty(&tag, &attrs)?;
                    children.push(component);
                }
                Ok(Event::End(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == parent_tag {
                        return Ok(children);
                    } else {
                        let msg = format!("Unexpected closing tag </{}>, expected </{}>", tag, parent_tag);
                        error!("[UI::Parser] {}", msg);
                        return Err(msg);
                    }
                }
                Ok(Event::Eof) => {
                    let msg = format!("Unexpected end of file while parsing <{}>", parent_tag);
                    error!("[UI::Parser] {}", msg);
                    return Err(msg);
                }
                Ok(_) => continue,
                Err(e) => {
                    let msg = format!("XML syntax error inside <{}>: {}", parent_tag, e);
                    error!("[UI::Parser] {}", msg);
                    return Err(msg);
                }
            }
        }
    }

    // * -- Process raw attrs: expand $var_name references, return the resolved attr list -- *
    fn process_attrs(raw: Vec<(String, String)>, vars: &VarStore, globals: &GlobalVarStore) -> Vec<(String, String)> {
        raw.into_iter()
            .map(|(key, val)| {
                let expanded = expand_vars(val.trim(), vars, globals);
                (key, expanded)
            })
            .collect()
    }

    // * -- Validate that a tag name is recognized -- *
    fn validate_tag(tag: &str) -> Result<(), String> {
        if VALID_TAGS.contains(&tag) {
            Ok(())
        } else {
            let msg = format!("Unknown UI component tag: <{}>", tag);
            error!("[UI::Parser] {}", msg);
            Err(msg)
        }
    }

    // * -- Parse XML attributes into key-value pairs -- *
    fn parse_attributes(e: &quick_xml::events::BytesStart) -> Result<Vec<(String, String)>, String> {
        let mut attrs = Vec::new();
        for attr in e.attributes() {
            let attr = attr.map_err(|err| format!("Attribute parse error: {}", err))?;
            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
            let val = String::from_utf8_lossy(&attr.value).to_string();
            attrs.push((key, val));
        }
        Ok(attrs)
    }

    // * -- Get a raw (unexpanded) attribute string -- *
    fn raw_attr(attrs: &[(String, String)], key: &str) -> Option<String> {
        attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    // * -- Build a component that has an opening+closing tag -- *
    fn build_component(
        tag: &str,
        attrs: &[(String, String)],
        reader: &mut Reader<&[u8]>,
        vars: &mut VarStore,
        globals: &mut GlobalVarStore,
        base_dir: Option<&Path>,
        include_stack: &mut Vec<PathBuf>,
    ) -> Result<UIComponent, String> {
        let params = Self::extract_basic_params(attrs);
        let children = Self::parse_children(reader, tag, vars, globals, base_dir, include_stack)?;

        match tag {
            "view" => Ok(UIComponent::View {
                align_x: Self::get_align_x(attrs, "alignX"),
                align_y: Self::get_align_y(attrs, "alignY"),
                params,
                children,
            }),
            "scroll_view" => Ok(UIComponent::ScrollView {
                align_x: Self::get_align_x(attrs, "alignX"),
                align_y: Self::get_align_y(attrs, "alignY"),
                scroll_x: Self::get_bool(attrs, "scrollX", true),
                scroll_y: Self::get_bool(attrs, "scrollY", true),
                params,
                children,
            }),
            "button" => Ok(UIComponent::Button {
                text: Self::get_str(attrs, "text", ""),
                color: Self::get_color(attrs, "color", Color::new(1.0, 1.0, 1.0, 1.0)),
                align_x: Self::get_align_x(attrs, "alignX"),
                align_y: Self::get_align_y(attrs, "alignY"),
                hovered: false,
                pressed: false,
                color_tween: None,
                params,
                children,
            }),
            "physics-container-2d" => {
                if !cfg!(feature = "ui-2d") {
                    return Err("Tag <physics-container-2d> requires Cargo feature `ui-2d`".to_string());
                }
                Ok(UIComponent::PhysicsContainer2D {
                    params,
                    engine: Self::get_physics_2d_engine(attrs),
                    children,
                })
            }
            "physics-container-3d" => {
                if !cfg!(feature = "ui-3d") {
                    return Err("Tag <physics-container-3d> requires Cargo feature `ui-3d`".to_string());
                }
                Ok(UIComponent::PhysicsContainer3D {
                    params,
                    children,
                })
            }
            _ => Self::build_leaf(tag, attrs, params),
        }
    }

    // * -- Build a self-closing tag component -- *
    fn build_component_empty(tag: &str, attrs: &[(String, String)]) -> Result<UIComponent, String> {
        let params = Self::extract_basic_params(attrs);

        match tag {
            "view" => Ok(UIComponent::View {
                align_x: Self::get_align_x(attrs, "alignX"),
                align_y: Self::get_align_y(attrs, "alignY"),
                params,
                children: Vec::new(),
            }),
            "scroll_view" => Ok(UIComponent::ScrollView {
                align_x: Self::get_align_x(attrs, "alignX"),
                align_y: Self::get_align_y(attrs, "alignY"),
                scroll_x: Self::get_bool(attrs, "scrollX", true),
                scroll_y: Self::get_bool(attrs, "scrollY", true),
                params,
                children: Vec::new(),
            }),
            "button" => Ok(UIComponent::Button {
                text: Self::get_str(attrs, "text", ""),
                color: Self::get_color(attrs, "color", Color::new(1.0, 1.0, 1.0, 1.0)),
                align_x: Self::get_align_x(attrs, "alignX"),
                align_y: Self::get_align_y(attrs, "alignY"),
                hovered: false,
                pressed: false,
                color_tween: None,
                params,
                children: Vec::new(),
            }),
            "physics-container-2d" => {
                if !cfg!(feature = "ui-2d") {
                    return Err("Tag <physics-container-2d> requires Cargo feature `ui-2d`".to_string());
                }
                Ok(UIComponent::PhysicsContainer2D {
                    params,
                    engine: Self::get_physics_2d_engine(attrs),
                    children: Vec::new(),
                })
            }
            "physics-container-3d" => {
                if !cfg!(feature = "ui-3d") {
                    return Err("Tag <physics-container-3d> requires Cargo feature `ui-3d`".to_string());
                }
                Ok(UIComponent::PhysicsContainer3D {
                    params,
                    children: Vec::new(),
                })
            }
            _ => Self::build_leaf(tag, attrs, params),
        }
    }

    // * -- Build a leaf component -- *
    fn build_leaf(tag: &str, attrs: &[(String, String)], params: BasicParams) -> Result<UIComponent, String> {
        match tag {
            "slider" => Ok(UIComponent::Slider {
                min: Self::get_f32(attrs, "min", 0.0),
                max: Self::get_f32(attrs, "max", 100.0),
                value: Self::get_f32(attrs, "value", 0.0),
                color: Self::get_color(attrs, "color", Color::new(0.5, 0.5, 0.5, 1.0)),
                track_color: Self::get_color(attrs, "trackColor", Color::new(0.2, 0.2, 0.2, 1.0)),
                dragging: false,
                changed: false,
                color_tween: None,
                params,
            }),
            "switch" => Ok(UIComponent::Switch {
                checked: Self::get_bool(attrs, "checked", false),
                color: Self::get_color(attrs, "color", Color::new(0.3, 0.8, 0.3, 1.0)),
                color_tween: None,
                params,
            }),
            "label" => {
                let mut lparams = Self::extract_basic_params(attrs);
                if !attrs.iter().any(|(k, _)| k == "width") {
                    lparams.width = SizeValue::Auto;
                }
                if !attrs.iter().any(|(k, _)| k == "height") {
                    lparams.height = SizeValue::Auto;
                }
                Ok(UIComponent::Label {
                    text: Self::get_str(attrs, "text", ""),
                    color: Self::get_color(attrs, "color", Color::new(1.0, 1.0, 1.0, 1.0)),
                    color_tween: None,
                    params: lparams,
                })
            }
            "text_field" => Ok(UIComponent::TextField {
                placeholder: Self::get_str(attrs, "placeholder", ""),
                text: Self::get_str(attrs, "text", ""),
                color: Self::get_color(attrs, "color", Color::new(1.0, 1.0, 1.0, 1.0)),
                focused: false,
                color_tween: None,
                params,
            }),
            "texture" | "texture-2d" => Ok(UIComponent::Texture {
                src: Self::get_str(attrs, "src", ""),
                object_fit: Self::get_object_fit(attrs),
                params,
            }),
            "animated_texture" | "animation" => {
                let autoplay = Self::get_bool(attrs, "autoplay", false);
                Ok(UIComponent::AnimatedTexture {
                    src:           Self::get_str(attrs, "src", ""),
                    fps:           Self::get_f32(attrs, "fps", 12.0),
                    loop_anim:     Self::get_bool(attrs, "loop", false),
                    autoplay,
                    opacity:       Self::get_opacity(attrs, "opacity", 1.0),
                    object_fit:    Self::get_object_fit(attrs),
                    // Populated by load_textures() from conf.jsonc
                    frame_count:   0,
                    timing:        [0.42, 0.0, 0.58, 1.0], // ease-in-out until conf loaded
                    playing:       autoplay,
                    paused:        false,
                    current_frame: 0,
                    elapsed:       0.0,
                    params,
                })
            }
            "progress_bar" => Ok(UIComponent::ProgressBar {
                value: Self::get_f32(attrs, "value", 0.0),
                max: Self::get_f32(attrs, "max", 100.0),
                color: Self::get_color(attrs, "color", Color::new(0.3, 0.7, 1.0, 1.0)),
                color_tween: None,
                params,
            }),
            "rect" => Ok(UIComponent::Rect {
                border_color: Self::get_color(attrs, "borderColor", Color::new(0.0, 0.0, 0.0, 0.0)),
                border_width: Self::get_f32(attrs, "borderWidth", 0.0),
                corner_radius: Self::get_f32(attrs, "cornerRadius", 0.0),
                params,
            }),
            "gradient" => Ok(UIComponent::Gradient {
                gradient_type: Self::get_gradient_type(attrs),
                angle: Self::get_angle_f32(attrs, "angle", 0.0),
                color1: Self::get_color(attrs, "color1", Color::new(0.0, 0.0, 0.0, 1.0)),
                color2: Self::get_color(attrs, "color2", Color::new(1.0, 1.0, 1.0, 1.0)),
                params,
            }),
            _ => Err(format!("Unknown leaf tag: <{}>", tag)),
        }
    }

    fn load_included_schema(
        attrs: &[(String, String)],
        vars: &mut VarStore,
        globals: &mut GlobalVarStore,
        base_dir: Option<&Path>,
        include_stack: &mut Vec<PathBuf>,
    ) -> Result<Vec<UIComponent>, String> {
        let src_raw = Self::raw_attr(attrs, "src").unwrap_or_default();
        let src = expand_vars(src_raw.trim(), vars, globals);

        if src.trim().is_empty() {
            return Err("<schema> include is missing required 'src' attribute".to_string());
        }

        let base_dir = base_dir.ok_or_else(|| {
            format!("Cannot resolve <schema src=\"{}\"> without a source file base directory", src)
        })?;

        let include_path = Self::resolve_include_path(base_dir, &src)?;
        let include_path_str = include_path.to_string_lossy().to_string();
        let (included_components, included_vars) = Self::load_with_globals_recursive(&include_path_str, globals, include_stack)?;
        vars.absorb_from(&included_vars);
        Ok(included_components)
    }

    fn resolve_include_path(base_dir: &Path, src: &str) -> Result<PathBuf, String> {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return Err("Schema include src cannot be empty".to_string());
        }

        let src_path = if let Some(rooted) = trimmed.strip_prefix("@/") {
            PathBuf::from(rooted)
        } else {
            PathBuf::from(trimmed)
        };

        let joined = if src_path.is_absolute() {
            src_path
        } else {
            base_dir.join(src_path)
        };

        fs::canonicalize(&joined).map_err(|e| {
            format!(
                "Failed to resolve included schema '{}' from '{}': {}",
                src,
                base_dir.to_string_lossy(),
                e
            )
        })
    }

    fn extract_schema_sources(xml: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut reader = Reader::from_str(xml);

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    if String::from_utf8_lossy(e.name().as_ref()) != "schema" {
                        continue;
                    }
                    if let Ok(attrs) = Self::parse_attributes(e) {
                        if let Some(src) = Self::raw_attr(&attrs, "src") {
                            let src = src.trim().to_string();
                            if !src.is_empty() {
                                out.push(src);
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(_) => break,
            }
        }

        out
    }

    fn skip_tag(reader: &mut Reader<&[u8]>, tag: &str) -> Result<(), String> {
        let mut depth = 1;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    if String::from_utf8_lossy(e.name().as_ref()) == tag {
                        depth += 1;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if String::from_utf8_lossy(e.name().as_ref()) == tag {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(());
                        }
                    }
                }
                Ok(Event::Eof) => {
                    return Err(format!("Unexpected EOF while skipping <{}> tag", tag));
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(format!("XML syntax error while skipping <{}>: {}", tag, e));
                }
            }
        }
    }

    // * ======== ATTRIBUTE HELPERS ======== *
    fn extract_basic_params(attrs: &[(String, String)]) -> BasicParams {
        BasicParams {
            id: Self::get_str(attrs, "id", ""),
            class_name: Self::get_str(attrs, "className", ""),
            width: Self::get_size(attrs, "width", SizeValue::Expr(SizeExpr::Px(50.0))),
            height: Self::get_size(attrs, "height", SizeValue::Expr(SizeExpr::Px(50.0))),
            x: Self::get_pos(attrs, "x"),
            y: Self::get_pos(attrs, "y"),
            rotation: Self::get_f32(attrs, "rotation", 0.0),
            scale: Self::get_f32(attrs, "scale", 1.0),
            font_size: Self::get_u32(attrs, "fontSize", 16),
            z_order: Self::get_i32(attrs, "zOrder", 0),
            visible: Self::get_bool(attrs, "visible", true),
            usr_interact: Self::get_bool(attrs, "usrInteract", true),
            bg_color: Self::get_color(attrs, "bgColor", Color::new(0.0, 0.0, 0.0, 0.0)),
            transition: TransitionConfig {
                duration: Self::get_transition_duration(attrs, "transition", 0.0),
                easing: Self::get_transition_easing(attrs, &["transitionTiming", "transitionTimingFunction", "transition-timing-function"], TransitionEasing::Ease),
            },
            bg_color_tween: None,
            scale_tween: None,
        }
    }

    fn get_transition_duration(attrs: &[(String, String)], key: &str, default_seconds: f32) -> f32 {
        match attrs.iter().find(|(k, _)| k == key) {
            Some((_, v)) => {
                let s = v.trim().to_lowercase();
                if let Some(ms) = s.strip_suffix("ms") {
                    ms.trim().parse::<f32>().unwrap_or(default_seconds * 1000.0) / 1000.0
                } else if let Some(sec) = s.strip_suffix('s') {
                    sec.trim().parse::<f32>().unwrap_or(default_seconds)
                } else {
                    s.parse::<f32>().unwrap_or(default_seconds)
                }
            }
            None => default_seconds,
        }
        .max(0.0)
    }

    fn get_transition_easing(attrs: &[(String, String)], keys: &[&str], default: TransitionEasing) -> TransitionEasing {
        let Some((_, raw)) = attrs.iter().find(|(k, _)| keys.iter().any(|key| key == k)) else {
            return default;
        };

        let eased = raw.trim().to_lowercase();
        match eased.as_str() {
            "ease" => TransitionEasing::Ease,
            "ease-in" => TransitionEasing::EaseIn,
            "ease-out" => TransitionEasing::EaseOut,
            "ease-in-out" => TransitionEasing::EaseInOut,
            "linear" => TransitionEasing::Linear,
            _ => {
                if let Some(points) = Self::parse_cubic_bezier(&eased) {
                    TransitionEasing::CubicBezier(points)
                } else {
                    default
                }
            }
        }
    }

    fn parse_cubic_bezier(raw: &str) -> Option<[f32; 4]> {
        let start = raw.find("cubic-bezier(")?;
        if start != 0 || !raw.ends_with(')') {
            return None;
        }
        let inside = &raw[13..raw.len() - 1];
        let parts: Vec<&str> = inside.split(',').map(|s| s.trim()).collect();
        if parts.len() != 4 {
            return None;
        }
        let x1 = parts[0].parse::<f32>().ok()?;
        let y1 = parts[1].parse::<f32>().ok()?;
        let x2 = parts[2].parse::<f32>().ok()?;
        let y2 = parts[3].parse::<f32>().ok()?;
        Some([x1, y1, x2, y2])
    }

    // * -- Parse a SizeValue from an attribute string -- *
    // Formats: "40px"  "-40px"  "50%"  "30vw"  "20vh"  "(40px + 2%) - 2vh"  "auto"
    // "-20px" means parent_dim - 20 (from the opposite side).
    // Bare integers (e.g. "40") are treated as pixels: "40px".
    fn get_size(attrs: &[(String, String)], key: &str, default: SizeValue) -> SizeValue {
        let raw = match attrs.iter().find(|(k, _)| k == key) {
            Some((_, v)) => v.trim().to_string(),
            None => return default,
        };

        if raw == "auto" {
            return SizeValue::Auto;
        }
        SizeValue::Expr(Self::parse_size_expr(&raw))
    }

    // * -- Parse a math size expression into a SizeExpr AST -- *
    fn parse_size_expr(s: &str) -> SizeExpr {
        let tokens = tokenize(s);
        let (expr, _) = parse_expr(&tokens, 0);
        expr
    }

    // * -- Parse a PosValue from an attribute string -- *
    // Formats: "40px"  "-40px"  "50%"  "(40px+2%)"  "auto"  "$"  "$+20px"  "$-20px"  "$(expr)"
    // "-20px" in a position means parent_dim - 20 (from the opposite side).
    // "$" alone (not followed by a letter) → parent-aligned Auto.
    // "$name" (followed by a letter) → variable reference (already expanded by expand_vars).
    fn get_pos(attrs: &[(String, String)], key: &str) -> PosValue {
        let raw = match attrs.iter().find(|(k, _)| k == key) {
            Some((_, v)) => v.trim().to_string(),
            None => return PosValue::Px(SizeExpr::Px(0.0)),
        };

        // "auto" or plain "$" (not followed by identifier)
        if raw == "auto" || raw == "$" {
            return PosValue::Auto(SizeExpr::Px(0.0));
        }
        // "$+20px", "$-20px", "$(expr)" — auto with expression offset
        // (var refs have already been expanded to numeric strings by this point)
        if raw.starts_with('$') && !raw[1..].starts_with(|c: char| c.is_alphabetic() || c == '_') {
            return PosValue::Auto(Self::parse_size_expr(raw[1..].trim()));
        }
        PosValue::Px(Self::parse_size_expr(&raw))
    }

    fn get_align_x(attrs: &[(String, String)], key: &str) -> AlignX {
        match attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()) {
            Some("center") => AlignX::Center,
            Some("right")  => AlignX::Right,
            _              => AlignX::Left,
        }
    }

    fn get_align_y(attrs: &[(String, String)], key: &str) -> AlignY {
        match attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()) {
            Some("center") => AlignY::Center,
            Some("bottom") => AlignY::Bottom,
            _              => AlignY::Top,
        }
    }

    fn get_str(attrs: &[(String, String)], key: &str, default: &str) -> String {
        attrs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| default.to_string())
    }

    fn get_f32(attrs: &[(String, String)], key: &str, default: f32) -> f32 {
        attrs.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| {
                // Accept both plain floats and expressions like "50px", "90deg" → strip unit suffix
                let s = v.trim();
                let stripped = s.trim_end_matches("deg")
                                .trim_end_matches("px")
                                .trim_end_matches('%')
                                .trim_end_matches("vw")
                                .trim_end_matches("vh");
                stripped.parse().ok()
            })
            .unwrap_or(default)
    }

    fn get_i32(attrs: &[(String, String)], key: &str, default: i32) -> i32 {
        attrs.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| v.trim().trim_end_matches("px").parse().ok())
            .unwrap_or(default)
    }

    fn get_u32(attrs: &[(String, String)], key: &str, default: u32) -> u32 {
        attrs.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| v.trim().trim_end_matches("px").parse().ok())
            .unwrap_or(default)
    }

    fn get_bool(attrs: &[(String, String)], key: &str, default: bool) -> bool {
        attrs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v == "true" || v == "1")
            .unwrap_or(default)
    }

    fn get_color(attrs: &[(String, String)], key: &str, default: Color) -> Color {
        attrs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| <Color as HexSupport>::from_hex(v.trim()))
            .unwrap_or(default)
    }

    /// Parse opacity/percent attribute: "100%" → 1.0, "50%" → 0.5, "0.75" → 0.75
    fn get_opacity(attrs: &[(String, String)], key: &str, default: f32) -> f32 {
        attrs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| {
                let s = v.trim();
                if let Some(pct) = s.strip_suffix('%') {
                    pct.parse::<f32>().unwrap_or(default * 100.0) / 100.0
                } else {
                    s.trim_end_matches("px").parse::<f32>().unwrap_or(default)
                }
            })
            .unwrap_or(default)
            .clamp(0.0, 1.0)
    }

    fn get_object_fit(attrs: &[(String, String)]) -> ObjectFit {
        match attrs.iter().find(|(k, _)| k == "object-fit").map(|(_, v)| v.as_str()) {
            Some("cover")   => ObjectFit::Cover,
            Some("contain") => ObjectFit::Contain,
            _               => ObjectFit::Warp,  // default: stretch to fill
        }
    }

    /// Parse a size/angle expression string (already `$var`-expanded by `process_attrs`).
    /// Resolves with parent_dim=0 so `px` and `deg` values pass through as-is.
    fn get_angle_f32(attrs: &[(String, String)], key: &str, default: f32) -> f32 {
        attrs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| {
                let tokens = tokenize(v.trim());
                let (expr, _) = parse_expr(&tokens, 0);
                expr.resolve(0.0, 0.0, 0.0)
            })
            .unwrap_or(default)
    }

    fn get_gradient_type(attrs: &[(String, String)]) -> GradientType {
        match attrs.iter().find(|(k, _)| k == "type").map(|(_, v)| v.as_str()) {
            Some("radial") => GradientType::Radial,
            _              => GradientType::Linear,
        }
    }

    /// Parse the optional 2D physics backend selector.
    ///
    /// Supported values:
    /// - `macroquad` (default)
    /// - `bevy`
    fn get_physics_2d_engine(attrs: &[(String, String)]) -> Physics2DEngine {
        match attrs.iter().find(|(k, _)| k == "engine").map(|(_, v)| v.trim().to_ascii_lowercase()) {
            Some(value) if value == "bevy" => Physics2DEngine::Bevy,
            _ => Physics2DEngine::Macroquad,
        }
    }
}
