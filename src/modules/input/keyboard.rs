// * =========== IMPORTS =========== * //
use std::collections::{HashMap, HashSet};
use macroquad::prelude::{is_key_down, KeyCode};

// ── Type aliases ──────────────────────────────────────────────────────────────

/// One or more physical keys treated as equivalent (e.g. LeftControl + RightControl for "Ctrl").
/// Any key in the group being held satisfies the group.
type KeyGroup = Vec<KeyCode>;

/// An ordered list of key groups that must ALL be held simultaneously.
/// Example: `[{LeftControl, RightControl}, {S}]` for "Ctrl+S".
type Combo = Vec<KeyGroup>;

/// Multiple binding variants for a single action, separated by `|` in the config string.
/// Any one variant being active triggers the action.
type Alternatives = Vec<Combo>;

// * =========== KEYBOARD MANAGER =========== * //

/// Manages keyboard input with support for:
/// - Multiple key alternatives per action  (`"W|ArrowUp"` → Jump)
/// - Simultaneous combo bindings           (`"Ctrl+S"`     → Save)
/// - Single-key bindings                   (`"Ctrl"`        → Sprint)
/// - Combo-priority suppression: a single-key action is suppressed while a
///   longer registered combo sharing that key is fully held.
///   e.g. holding Ctrl+S fires "Save" but NOT "Sprint".
pub struct KeyboardManager {
    /// Parsed keybinds: action name → alternatives.
    keybinds: HashMap<String, Alternatives>,
    /// Raw keybind strings kept for external access / serialisation.
    raw_keybinds: HashMap<String, String>,
    /// Actions held during the current frame (updated by `update()`).
    held_cache: HashSet<String>,
    /// Actions that transitioned not-held → held this frame.
    pressed_cache: HashSet<String>,
    /// Actions that transitioned held → not-held this frame.
    released_cache: HashSet<String>,
}

impl KeyboardManager {
    // ── Lifecycle ─────────────────────────────────────────────────────────────

    /// Creates the manager. Call once before the game loop.
    pub fn init(keybinds: HashMap<String, String>) -> Self {
        let parsed = Self::parse_keybinds(&keybinds);
        Self {
            keybinds: parsed,
            raw_keybinds: keybinds,
            held_cache:     HashSet::new(),
            pressed_cache:  HashSet::new(),
            released_cache: HashSet::new(),
        }
    }

    /// Samples the keyboard state for this frame.
    /// Call once at the **start** of every game-loop iteration, before querying actions.
    pub fn update(&mut self) {
        // Collect action names first to avoid a borrow conflict when calling eval_held.
        let actions: Vec<String> = self.keybinds.keys().cloned().collect();

        let currently_held: HashSet<String> = actions
            .iter()
            .filter(|a| self.eval_held(a))
            .cloned()
            .collect();

        self.pressed_cache  = currently_held.difference(&self.held_cache).cloned().collect();
        self.released_cache = self.held_cache.difference(&currently_held).cloned().collect();
        self.held_cache     = currently_held;
    }

    // ── Public query API (call after `update()` each frame) ───────────────────

    /// Returns `true` while the action's binding is held down.
    pub fn action_held(&self, action: &str) -> bool {
        self.held_cache.contains(action)
    }

    /// Returns `true` on the first frame the action's binding is pressed.
    pub fn action_pressed(&self, action: &str) -> bool {
        self.pressed_cache.contains(action)
    }

    /// Returns `true` on the frame the action's binding is released.
    pub fn action_released(&self, action: &str) -> bool {
        self.released_cache.contains(action)
    }

    // ── Runtime configuration ─────────────────────────────────────────────────

    /// Replaces all keybinds at runtime (e.g. from a settings menu).
    /// Resets all cached state to avoid stale readings.
    pub fn update_keybinds(&mut self, keybinds: HashMap<String, String>) {
        self.keybinds     = Self::parse_keybinds(&keybinds);
        self.raw_keybinds = keybinds;
        self.held_cache.clear();
        self.pressed_cache.clear();
        self.released_cache.clear();
    }

    /// Returns the raw keybind strings (action → binding string).
    pub fn get_keybinds(&self) -> &HashMap<String, String> {
        &self.raw_keybinds
    }

    /// Returns a cloned snapshot of keybinds for persistence.
    pub fn clone_keybinds(&self) -> HashMap<String, String> {
        self.raw_keybinds.clone()
    }

    /// Updates a single action binding and refreshes parsed bindings immediately.
    pub fn set_keybind(&mut self, action: &str, binding: &str) {
        self.raw_keybinds
            .insert(action.to_string(), binding.to_string());
        self.keybinds = Self::parse_keybinds(&self.raw_keybinds);
        self.held_cache.clear();
        self.pressed_cache.clear();
        self.released_cache.clear();
    }

    /// Returns `true` if `action` exists in the keybinds map.
    pub fn is_keybind_valid(&self, action: &str) -> bool {
        self.keybinds.contains_key(action)
    }

    // ── Internal evaluation ───────────────────────────────────────────────────

    fn eval_held(&self, action: &str) -> bool {
        let Some(alternatives) = self.keybinds.get(action) else {
            return false;
        };
        for combo in alternatives {
            if !Self::combo_held(combo) {
                continue;
            }
            // Suppress single-key combos when a longer registered combo that
            // shares the same key is fully held anywhere in the key map.
            if combo.len() == 1 && self.blocked_by_longer_combo(&combo[0]) {
                continue;
            }
            return true;
        }
        false
    }

    /// All key groups in the combo must have at least one key held.
    fn combo_held(combo: &Combo) -> bool {
        combo.iter().all(|group| group.iter().any(|k| is_key_down(*k)))
    }

    /// Returns `true` if any multi-key combo registered for **any** action —
    /// whose keys overlap with `single_group` — is currently fully held.
    fn blocked_by_longer_combo(&self, single_group: &KeyGroup) -> bool {
        for alternatives in self.keybinds.values() {
            for combo in alternatives {
                if combo.len() <= 1 {
                    continue;
                }
                let shares_key = combo
                    .iter()
                    .any(|g| g.iter().any(|k| single_group.contains(k)));
                if shares_key && Self::combo_held(combo) {
                    return true;
                }
            }
        }
        false
    }

    // ── Parsing ───────────────────────────────────────────────────────────────

    /// Parses `HashMap<action, bind_string>` into the internal representation.
    ///
    /// Bind string syntax:
    /// - `|` separates alternatives:  `"W|ArrowUp"`
    /// - `+` separates combo keys:    `"Ctrl+S"`
    /// - Both combined:               `"Ctrl+S|Ctrl+Z"`
    fn parse_keybinds(raw: &HashMap<String, String>) -> HashMap<String, Alternatives> {
        raw.iter()
            .map(|(action, bind_str)| {
                let alternatives = bind_str
                    .split('|')
                    .filter_map(|alt| {
                        let combo: Combo = alt
                            .trim()
                            .split('+')
                            .filter_map(|key_str| {
                                let group = Self::parse_key_group(key_str.trim());
                                if group.is_empty() {
                                    log::warn!(
                                        "[KeyboardManager] Unknown key '{}' in binding for '{}'",
                                        key_str.trim(), action
                                    );
                                    None
                                } else {
                                    Some(group)
                                }
                            })
                            .collect();
                        if combo.is_empty() { None } else { Some(combo) }
                    })
                    .collect::<Alternatives>();
                (action.clone(), alternatives)
            })
            .collect()
    }

    /// Maps a key-name string to a `KeyGroup`.
    /// Modifier aliases expand to both left and right variants.
    fn parse_key_group(key_str: &str) -> KeyGroup {
        match key_str.to_lowercase().as_str() {
            // ── Modifiers (both sides) ─────────────────────────────────────
            "ctrl" | "control"               => vec![KeyCode::LeftControl, KeyCode::RightControl],
            "shift"                          => vec![KeyCode::LeftShift,   KeyCode::RightShift],
            "alt"                            => vec![KeyCode::LeftAlt,     KeyCode::RightAlt],
            "super" | "win" | "cmd" | "meta" => vec![KeyCode::LeftSuper,   KeyCode::RightSuper],

            // ── Explicit left/right modifiers ──────────────────────────────
            "leftctrl"  | "leftcontrol"  => vec![KeyCode::LeftControl],
            "rightctrl" | "rightcontrol" => vec![KeyCode::RightControl],
            "leftshift"                  => vec![KeyCode::LeftShift],
            "rightshift"                 => vec![KeyCode::RightShift],
            "leftalt"                    => vec![KeyCode::LeftAlt],
            "rightalt"                   => vec![KeyCode::RightAlt],

            // ── Arrow keys ─────────────────────────────────────────────────
            "arrowup"    | "up"    => vec![KeyCode::Up],
            "arrowdown"  | "down"  => vec![KeyCode::Down],
            "arrowleft"  | "left"  => vec![KeyCode::Left],
            "arrowright" | "right" => vec![KeyCode::Right],

            // ── Named keys ─────────────────────────────────────────────────
            "enter"     | "return" => vec![KeyCode::Enter],
            "space"                => vec![KeyCode::Space],
            "escape"    | "esc"    => vec![KeyCode::Escape],
            "tab"                  => vec![KeyCode::Tab],
            "backspace"            => vec![KeyCode::Backspace],
            "delete"    | "del"    => vec![KeyCode::Delete],
            "insert"    | "ins"    => vec![KeyCode::Insert],
            "home"                 => vec![KeyCode::Home],
            "end"                  => vec![KeyCode::End],
            "pageup"               => vec![KeyCode::PageUp],
            "pagedown"             => vec![KeyCode::PageDown],
            "capslock"             => vec![KeyCode::CapsLock],
            "pause"                => vec![KeyCode::Pause],
            "printscreen"          => vec![KeyCode::PrintScreen],

            // ── Function keys ──────────────────────────────────────────────
            "f1"  => vec![KeyCode::F1],  "f2"  => vec![KeyCode::F2],
            "f3"  => vec![KeyCode::F3],  "f4"  => vec![KeyCode::F4],
            "f5"  => vec![KeyCode::F5],  "f6"  => vec![KeyCode::F6],
            "f7"  => vec![KeyCode::F7],  "f8"  => vec![KeyCode::F8],
            "f9"  => vec![KeyCode::F9],  "f10" => vec![KeyCode::F10],
            "f11" => vec![KeyCode::F11], "f12" => vec![KeyCode::F12],

            // ── Single character (letters + digits) ────────────────────────
            s => Self::parse_char_key(s),
        }
    }

    fn parse_char_key(s: &str) -> KeyGroup {
        let mut chars = s.chars();
        let Some(c) = chars.next() else { return vec![] };
        if chars.next().is_some() { return vec![] } // multi-char unknown token

        let key = match c.to_ascii_lowercase() {
            'a' => KeyCode::A, 'b' => KeyCode::B, 'c' => KeyCode::C,
            'd' => KeyCode::D, 'e' => KeyCode::E, 'f' => KeyCode::F,
            'g' => KeyCode::G, 'h' => KeyCode::H, 'i' => KeyCode::I,
            'j' => KeyCode::J, 'k' => KeyCode::K, 'l' => KeyCode::L,
            'm' => KeyCode::M, 'n' => KeyCode::N, 'o' => KeyCode::O,
            'p' => KeyCode::P, 'q' => KeyCode::Q, 'r' => KeyCode::R,
            's' => KeyCode::S, 't' => KeyCode::T, 'u' => KeyCode::U,
            'v' => KeyCode::V, 'w' => KeyCode::W, 'x' => KeyCode::X,
            'y' => KeyCode::Y, 'z' => KeyCode::Z,
            '0' => KeyCode::Key0, '1' => KeyCode::Key1, '2' => KeyCode::Key2,
            '3' => KeyCode::Key3, '4' => KeyCode::Key4, '5' => KeyCode::Key5,
            '6' => KeyCode::Key6, '7' => KeyCode::Key7, '8' => KeyCode::Key8,
            '9' => KeyCode::Key9,
            _ => return vec![],
        };
        vec![key]
    }
}