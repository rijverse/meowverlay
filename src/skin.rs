//! Skin discovery and asset loading.
//!
//! A skin is a directory under `skins/` containing a `config.json` and an `img/` tree laid out
//! exactly like `bongocat-osu` skins, so existing skins are drop-in compatible.

use crate::config::Config;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Logical key -> path (relative to the skin dir) for every sprite we may draw.
/// Mirrors the `imagePaths` table from the previous TypeScript implementation.
const IMAGE_PATHS: &[(&str, &str)] = &[
    ("mousebg", "img/osu/mousebg.png"),
    ("tabletbg", "img/osu/tabletbg.png"),
    ("osu_left", "img/osu/left.png"),
    ("osu_right", "img/osu/right.png"),
    ("osu_up", "img/osu/up.png"),
    ("mouse", "img/osu/mouse.png"),
    ("tablet", "img/osu/tablet.png"),
    ("smoke", "img/osu/smoke.png"),
    ("wave", "img/osu/wave.png"),
    ("taiko_bg", "img/taiko/bg.png"),
    ("taiko_leftrim", "img/taiko/leftrim.png"),
    ("taiko_leftcentre", "img/taiko/leftcentre.png"),
    ("taiko_leftup", "img/taiko/leftup.png"),
    ("taiko_rightrim", "img/taiko/rightrim.png"),
    ("taiko_rightcentre", "img/taiko/rightcentre.png"),
    ("taiko_rightup", "img/taiko/rightup.png"),
    ("catch_bg", "img/catch/bg.png"),
    ("catch_left", "img/catch/left.png"),
    ("catch_right", "img/catch/right.png"),
    ("catch_up", "img/catch/up.png"),
    ("catch_dash", "img/catch/dash.png"),
    ("catch_mid", "img/catch/mid.png"),
    ("mania_bg_4K", "img/mania/4K/bg.png"),
    ("mania_bg_7K", "img/mania/7K/bg.png"),
    ("mania_leftup", "img/mania/leftup.png"),
    ("mania_left0", "img/mania/left0.png"),
    ("mania_left1", "img/mania/left1.png"),
    ("mania_left2", "img/mania/left2.png"),
    ("mania_rightup", "img/mania/rightup.png"),
    ("mania_right0", "img/mania/right0.png"),
    ("mania_right1", "img/mania/right1.png"),
    ("mania_right2", "img/mania/right2.png"),
    ("key_4K_0", "img/mania/4K/0.png"),
    ("key_4K_1", "img/mania/4K/1.png"),
    ("key_4K_2", "img/mania/4K/2.png"),
    ("key_4K_3", "img/mania/4K/3.png"),
    ("key_7K_0", "img/mania/7K/0.png"),
    ("key_7K_1", "img/mania/7K/1.png"),
    ("key_7K_2", "img/mania/7K/2.png"),
    ("key_7K_3", "img/mania/7K/3.png"),
    ("key_7K_4", "img/mania/7K/4.png"),
    ("key_7K_5", "img/mania/7K/5.png"),
    ("key_7K_6", "img/mania/7K/6.png"),
];

pub struct Skin {
    pub config: Config,
    /// `Some(msg)` when the skin's `config.json` existed but failed to parse. The app loads defaults
    /// in that case but must *not* silently overwrite the user's file (see `MeowApp::save_config`).
    pub config_error: Option<String>,
    textures: HashMap<&'static str, egui::TextureHandle>,
}

/// The background texture key for the given mode, honouring the osu! mouse/tablet and mania 4K/7K
/// sub-selections. Mirrors the per-mode draw routines so the window sizes to what actually renders.
fn mode_bg_key(config: &Config) -> &'static str {
    match config.mode {
        2 => "taiko_bg",
        3 => "catch_bg",
        4 => {
            if config.mania.four_k {
                "mania_bg_4K"
            } else {
                "mania_bg_7K"
            }
        }
        _ => {
            if config.osu.mouse {
                "mousebg"
            } else {
                "tabletbg"
            }
        }
    }
}

impl Skin {
    pub fn get(&self, key: &str) -> Option<&egui::TextureHandle> {
        self.textures.get(key)
    }

    /// The overlay canvas size, derived from the *current mode's* background image so swapping modes
    /// (or skins whose Taiko/Catch/Mania backgrounds differ in size) resizes the window correctly.
    /// Falls back through the other mode backgrounds, then to the classic 612x354.
    pub fn canvas_size(&self) -> egui::Vec2 {
        let primary = mode_bg_key(&self.config);
        for key in [
            primary,
            "mousebg",
            "tabletbg",
            "taiko_bg",
            "catch_bg",
            "mania_bg_4K",
            "mania_bg_7K",
        ] {
            if let Some(tex) = self.textures.get(key) {
                let s = tex.size();
                if s[0] > 1 && s[1] > 1 {
                    return egui::vec2(s[0] as f32, s[1] as f32);
                }
            }
        }
        egui::vec2(612.0, 354.0)
    }

    /// Load a skin's config and all available sprites into GPU textures.
    ///
    /// Returns `Err` only when the skin directory itself is missing, so the caller can fall back to
    /// the default skin. A missing `config.json` loads defaults silently, while a *malformed* one loads
    /// defaults but records `config_error` so we never silently overwrite the user's file.
    pub fn load(ctx: &egui::Context, skins_dir: &Path, name: &str) -> Result<Self> {
        let skin_dir = skins_dir.join(name);
        if !skin_dir.is_dir() {
            anyhow::bail!("skin directory not found: {}", skin_dir.display());
        }

        let config_path = skin_dir.join("config.json");
        let (config, config_error) = if config_path.exists() {
            match Config::load(&config_path) {
                Ok(c) => (c, None),
                Err(e) => (Config::default(), Some(format!("{e:#}"))),
            }
        } else {
            (Config::default(), None)
        };

        let mut textures = HashMap::new();
        for (key, rel) in IMAGE_PATHS {
            let path = skin_dir.join(rel);
            match load_texture(ctx, key, &path) {
                Ok(tex) => {
                    textures.insert(*key, tex);
                }
                Err(_) => { /* missing sprite is fine; mode just won't draw it */ }
            }
        }

        Ok(Self {
            config,
            config_error,
            textures,
        })
    }
}

fn load_texture(ctx: &egui::Context, key: &str, path: &Path) -> Result<egui::TextureHandle> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let img = image::load_from_memory(&bytes)
        .with_context(|| format!("decoding {}", path.display()))?
        .to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Ok(ctx.load_texture(key, color, egui::TextureOptions::LINEAR))
}

/// Read just the background image dimensions (without decoding pixels) so the window can open at
/// the correct size before the egui context exists.
pub fn probe_canvas_size(skins_dir: &Path, name: &str) -> egui::Vec2 {
    let dir = skins_dir.join(name);
    // Consult the config so the window opens at the size of the *configured* mode's background,
    // not always the standard one. A missing/malformed config just falls back to defaults here,
    // the in-app load surfaces parse errors.
    let config = Config::load(&dir.join("config.json")).unwrap_or_default();
    let primary = match config.mode {
        2 => "img/taiko/bg.png",
        3 => "img/catch/bg.png",
        4 => {
            if config.mania.four_k {
                "img/mania/4K/bg.png"
            } else {
                "img/mania/7K/bg.png"
            }
        }
        _ => {
            if config.osu.mouse {
                "img/osu/mousebg.png"
            } else {
                "img/osu/tabletbg.png"
            }
        }
    };
    for rel in [
        primary,
        "img/osu/mousebg.png",
        "img/osu/tabletbg.png",
        "img/taiko/bg.png",
        "img/catch/bg.png",
        "img/mania/4K/bg.png",
        "img/mania/7K/bg.png",
    ] {
        if let Ok((w, h)) = image::image_dimensions(dir.join(rel)) {
            return egui::vec2(w as f32, h as f32);
        }
    }
    egui::vec2(612.0, 354.0)
}

/// List skin directories under `skins/`. Always returns at least `["default"]`.
pub fn discover_skins(skins_dir: &Path) -> Vec<String> {
    let mut skins = Vec::new();
    if let Ok(entries) = std::fs::read_dir(skins_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    skins.push(name.to_string());
                }
            }
        }
    }
    skins.sort();
    if skins.is_empty() {
        skins.push("default".to_string());
    }
    skins
}

/// Locate the `skins/` directory by walking up from the cwd and the executable path.
/// Port of the previous `find_skins_dir`, works on all platforms.
pub fn find_skins_dir() -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("skins");
        if p.is_dir() {
            return p;
        }
        if let Some(parent) = cwd.parent() {
            let p = parent.join("skins");
            if p.is_dir() {
                return p;
            }
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent();
        while let Some(d) = dir {
            let p = d.join("skins");
            if p.is_dir() {
                return p;
            }
            dir = d.parent();
        }
    }
    PathBuf::from("skins")
}
