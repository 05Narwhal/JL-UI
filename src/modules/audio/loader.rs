// * =========== IMPORTS =========== * //
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use macroquad::audio::{
    load_sound, play_sound, play_sound_once, set_sound_volume, stop_sound,
    PlaySoundParams, Sound,
};

// * =========== AUDIO MANAGER =========== * //

/// Volume calculation: `effective = master × category × individual`
///
/// # Categories
/// Sounds can be grouped into named categories (e.g. `"sfx"`, `"music"`, `"ambient"`).
/// Each category has its own volume multiplier applied on top of the master volume.
/// A sound can belong to at most one category at a time.
pub struct AudioManager {
    master_volume:      f32,
    /// Per-category volume multiplier (0.0–1.0).
    category_volumes:   HashMap<String, f32>,
    /// Sound ID → category name (reverse lookup).
    sound_categories:   HashMap<String, String>,
    /// Category name → set of sound IDs.
    categories:         HashMap<String, HashSet<String>>,
    /// Per-sound base volume (0.0–1.0), before category and master multipliers.
    individual_volumes: HashMap<String, f32>,
    /// The loaded Sound objects.
    loaded_sounds:      HashMap<String, Sound>,
}

impl AudioManager {
    // ── Lifecycle ─────────────────────────────────────────────────────────────

    /// Create the manager. Call once before the game loop.
    pub fn init() -> Self {
        Self {
            master_volume:      1.0,
            category_volumes:   HashMap::new(),
            sound_categories:   HashMap::new(),
            categories:         HashMap::new(),
            individual_volumes: HashMap::new(),
            loaded_sounds:      HashMap::new(),
        }
    }

    // ── Loading ────────────────────────────────────────────────────────────────

    /// Load a single sound file with an explicit ID.
    pub async fn load_audio(&mut self, sound_id: &str, path: &Path, volume: f32) {
        self.load_internal(sound_id, path, volume, None).await;
    }

    /// Load a single sound file with an explicit ID and assign it to `category`.
    pub async fn load_audio_into(&mut self, sound_id: &str, path: &Path, volume: f32, category: &str) {
        self.load_internal(sound_id, path, volume, Some(category)).await;
    }

    /// Load all supported audio files in `lib_path` and assign them to categories based on subfolder names.
    /// E.g. `lib_path/sfx/explosion.ogg` becomes sound ID `explosion` in category `sfx`.
    /// Supported extensions: `ogg`, `wav`, `mp3`, `flac`.
    pub async fn load_audio_library(&mut self, lib_path: &Path, volume: Option<f32>) {
        let volume = volume.unwrap_or(1.0);

        // go over the folder, each sub-folder is the category nae with the audios inside it
        let entries = match std::fs::read_dir(lib_path) {
            Ok(e) => e,
            Err(err) => {
                log::error!("[AudioManager] Cannot read directory {:?}: {}", lib_path, err);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let category = path.file_name().and_then(|n| n.to_str()).unwrap_or("uncategorized");
            self.load_folder_into(&path, volume, category).await;
        }
    }

    /// Load every supported audio file in `folder`.
    /// The filename stem (`"explosion"` from `"explosion.ogg"`) becomes the sound ID.
    /// Supported extensions: `ogg`, `wav`, `mp3`, `flac`.
    pub async fn load_folder(&mut self, folder: &Path, volume: f32) {
        self.load_folder_internal(folder, volume, None).await;
    }

    /// Load every supported audio file in `folder` and assign them all to `category`.
    pub async fn load_folder_into(&mut self, folder: &Path, volume: f32, category: &str) {
        self.load_folder_internal(folder, volume, Some(category)).await;
    }

    async fn load_folder_internal(&mut self, folder: &Path, volume: f32, category: Option<&str>) {
        let entries = match std::fs::read_dir(folder) {
            Ok(e) => e,
            Err(err) => {
                log::error!("[AudioManager] Cannot read directory {:?}: {}", folder, err);
                return;
            }
        };

        const SUPPORTED: &[&str] = &["ogg", "wav", "mp3", "flac"];

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() { continue; }

            let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase());
            let Some(ext) = ext else { continue };
            if !SUPPORTED.contains(&ext.as_str()) { continue; }

            let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()) else { continue };
            self.load_internal(&stem, &path, volume, category).await;
        }
    }

    async fn load_internal(&mut self, sound_id: &str, path: &Path, volume: f32, category: Option<&str>) {
        let Some(path_str) = path.to_str() else {
            log::error!("[AudioManager] Invalid path for sound '{}'", sound_id);
            return;
        };
        match load_sound(path_str).await {
            Ok(sound) => {
                let volume = volume.clamp(0.0, 1.0);
                // Compute effective volume incorporating category if supplied.
                let effective = self.effective_on_load(volume, category);
                set_sound_volume(&sound, effective);
                self.loaded_sounds.insert(sound_id.to_string(), sound);
                self.individual_volumes.insert(sound_id.to_string(), volume);
                if let Some(cat) = category {
                    self.attach_to_category(sound_id, cat);
                }
            }
            Err(err) => {
                log::error!("[AudioManager] Failed to load '{}' from {:?}: {:?}", sound_id, path, err);
            }
        }
    }

    // ── Category management ────────────────────────────────────────────────────

    /// Explicitly create a named category (also happens automatically on first use).
    pub fn create_category(&mut self, category: &str) {
        self.categories.entry(category.to_string()).or_default();
        self.category_volumes.entry(category.to_string()).or_insert(1.0);
    }

    /// Assign an already-loaded sound to `category`, replacing any previous assignment.
    pub fn assign_to_category(&mut self, sound_id: &str, category: &str) {
        if !self.loaded_sounds.contains_key(sound_id) {
            log::warn!("[AudioManager] assign_to_category: '{}' is not loaded", sound_id);
            return;
        }
        // Detach from old category.
        if let Some(old) = self.sound_categories.get(sound_id).cloned() {
            if let Some(set) = self.categories.get_mut(&old) {
                set.remove(sound_id);
            }
        }
        self.attach_to_category(sound_id, category);
        self.refresh_volume(sound_id);
    }

    /// Remove a sound from its current category.
    pub fn remove_from_category(&mut self, sound_id: &str) {
        if let Some(old) = self.sound_categories.remove(sound_id) {
            if let Some(set) = self.categories.get_mut(&old) {
                set.remove(sound_id);
            }
        }
        self.refresh_volume(sound_id);
    }

    fn attach_to_category(&mut self, sound_id: &str, category: &str) {
        self.sound_categories.insert(sound_id.to_string(), category.to_string());
        self.categories.entry(category.to_string()).or_default().insert(sound_id.to_string());
        self.category_volumes.entry(category.to_string()).or_insert(1.0);
    }

    // ── Volume ─────────────────────────────────────────────────────────────────

    /// Set the master volume (0.0–1.0). Immediately re-applies to all loaded sounds.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        let ids: Vec<String> = self.loaded_sounds.keys().cloned().collect();
        for id in ids { self.refresh_volume(&id); }
    }

    pub fn get_master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Set a category's volume multiplier (0.0–1.0). Immediately re-applies to all sounds in it.
    pub fn set_category_volume(&mut self, category: &str, volume: f32) {
        self.category_volumes.insert(category.to_string(), volume.clamp(0.0, 1.0));
        let ids: Vec<String> = self.categories
            .get(category)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        for id in ids { self.refresh_volume(&id); }
    }

    pub fn get_category_volume(&self, category: &str) -> f32 {
        self.category_volumes.get(category).copied().unwrap_or(1.0)
    }

    /// Set an individual sound's base volume (0.0–1.0).
    pub fn set_sound_volume(&mut self, sound_id: &str, volume: f32) {
        self.individual_volumes.insert(sound_id.to_string(), volume.clamp(0.0, 1.0));
        self.refresh_volume(sound_id);
    }

    pub fn get_sound_volume(&self, sound_id: &str) -> f32 {
        self.individual_volumes.get(sound_id).copied().unwrap_or(1.0)
    }

    // ── Playback ───────────────────────────────────────────────────────────────

    /// Play the sound once (not looped). Stops any instance already playing.
    pub fn play(&self, sound_id: &str) {
        self.play_internal(sound_id, false);
    }

    /// Play the sound looped. Stops any instance already playing.
    pub fn play_looped(&self, sound_id: &str) {
        self.play_internal(sound_id, true);
    }

    /// Spawn a one-shot sound instance without stopping what is already playing.
    /// Uses the effective volume set on the `Sound` object.
    pub fn play_once(&self, sound_id: &str) {
        if let Some(sound) = self.loaded_sounds.get(sound_id) {
            play_sound_once(sound);
        } else {
            log::warn!("[AudioManager] play_once: '{}' not loaded", sound_id);
        }
    }

    /// Stop the sound (stops all currently playing instances).
    pub fn stop(&self, sound_id: &str) {
        if let Some(sound) = self.loaded_sounds.get(sound_id) {
            stop_sound(sound);
        }
    }

    /// Stop all sounds belonging to `category`.
    pub fn stop_category(&self, category: &str) {
        if let Some(ids) = self.categories.get(category) {
            for id in ids {
                if let Some(sound) = self.loaded_sounds.get(id) {
                    stop_sound(sound);
                }
            }
        }
    }

    /// Stop every loaded sound.
    pub fn stop_all(&self) {
        for sound in self.loaded_sounds.values() {
            stop_sound(sound);
        }
    }

    // ── Queries ────────────────────────────────────────────────────────────────

    /// Returns `true` if a sound with `sound_id` has been loaded.
    pub fn is_loaded(&self, sound_id: &str) -> bool {
        self.loaded_sounds.contains_key(sound_id)
    }

    /// Returns the IDs of all sounds in `category`.
    pub fn get_sounds_in_category(&self, category: &str) -> Vec<&str> {
        self.categories
            .get(category)
            .map(|s| s.iter().map(|id| id.as_str()).collect())
            .unwrap_or_default()
    }

    /// Returns the category that `sound_id` belongs to, if any.
    pub fn get_category_of(&self, sound_id: &str) -> Option<&str> {
        self.sound_categories.get(sound_id).map(|s| s.as_str())
    }

    /// Returns all registered category names.
    pub fn list_categories(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }

    // ── Internals ──────────────────────────────────────────────────────────────

    fn play_internal(&self, sound_id: &str, looped: bool) {
        if let Some(sound) = self.loaded_sounds.get(sound_id) {
            let volume = self.compute_effective(sound_id);
            play_sound(sound, PlaySoundParams { volume, looped });
        } else {
            log::warn!("[AudioManager] play: '{}' not loaded", sound_id);
        }
    }

    /// Compute and push the effective volume onto the Sound object (for play_once compat).
    fn refresh_volume(&self, sound_id: &str) {
        if let Some(sound) = self.loaded_sounds.get(sound_id) {
            set_sound_volume(sound, self.compute_effective(sound_id));
        }
    }

    fn compute_effective(&self, sound_id: &str) -> f32 {
        let individual = self.individual_volumes.get(sound_id).copied().unwrap_or(1.0);
        let cat_vol = self.sound_categories
            .get(sound_id)
            .and_then(|cat| self.category_volumes.get(cat))
            .copied()
            .unwrap_or(1.0);
        (individual * cat_vol * self.master_volume).clamp(0.0, 1.0)
    }

    /// Effective volume at load time, before the sound is inserted into the maps.
    fn effective_on_load(&self, individual: f32, category: Option<&str>) -> f32 {
        let cat_vol = category
            .and_then(|cat| self.category_volumes.get(cat))
            .copied()
            .unwrap_or(1.0);
        (individual * cat_vol * self.master_volume).clamp(0.0, 1.0)
    }
}