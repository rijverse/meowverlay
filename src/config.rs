//! Typed model of the `bongocat-osu` `config.json` format.
//!
//! Every field has a `serde(default)` so arbitrary / partial skin configs load without error,
//! and unknown top-level keys (e.g. `"custom"`) are preserved on save via `extra`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    #[serde(default)]
    pub letterboxing: bool,
    #[serde(default = "default_width")]
    pub width: f64,
    #[serde(default = "default_height")]
    pub height: f64,
    #[serde(default)]
    pub horizontal_position: f64,
    #[serde(default)]
    pub vertical_position: f64,
}
fn default_width() -> f64 { 1920.0 }
fn default_height() -> f64 { 1080.0 }
impl Default for Resolution {
    fn default() -> Self {
        Self { letterboxing: false, width: 1920.0, height: 1080.0, horizontal_position: 0.0, vertical_position: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Decoration {
    #[serde(default)]
    pub left_handed: bool,
    #[serde(default = "white")]
    pub rgb: Vec<u8>,
    #[serde(default = "two_zero")]
    pub offset_x: Vec<f64>,
    #[serde(default = "two_zero")]
    pub offset_y: Vec<f64>,
    #[serde(default = "two_one")]
    pub scalar: Vec<f64>,
}
fn white() -> Vec<u8> { vec![255, 255, 255] }
fn two_zero() -> Vec<f64> { vec![0.0, 0.0] }
fn two_one() -> Vec<f64> { vec![1.0, 1.0] }
impl Default for Decoration {
    fn default() -> Self {
        Self { left_handed: false, rgb: white(), offset_x: vec![0.0, 11.0], offset_y: vec![0.0, -65.0], scalar: vec![1.0, 1.0] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsuCfg {
    #[serde(default = "yes")]
    pub mouse: bool,
    #[serde(default)]
    pub toggle_smoke: bool,
    #[serde(default = "white")]
    pub paw: Vec<u8>,
    #[serde(default = "black")]
    pub paw_edge: Vec<u8>,
    #[serde(default)]
    pub key1: Vec<u32>,
    #[serde(default)]
    pub key2: Vec<u32>,
    #[serde(default)]
    pub smoke: Vec<u32>,
    #[serde(default)]
    pub wave: Vec<u32>,
}
fn yes() -> bool { true }
fn black() -> Vec<u8> { vec![0, 0, 0] }
impl Default for OsuCfg {
    fn default() -> Self {
        Self { mouse: true, toggle_smoke: false, paw: white(), paw_edge: black(),
            key1: vec![90], key2: vec![88], smoke: vec![67], wave: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaikoCfg {
    #[serde(default = "k88")] pub left_centre: Vec<u32>,
    #[serde(default = "k67")] pub right_centre: Vec<u32>,
    #[serde(default = "k90")] pub left_rim: Vec<u32>,
    #[serde(default = "k86")] pub right_rim: Vec<u32>,
}
fn k88() -> Vec<u32> { vec![88] }
fn k67() -> Vec<u32> { vec![67] }
fn k90() -> Vec<u32> { vec![90] }
fn k86() -> Vec<u32> { vec![86] }
impl Default for TaikoCfg {
    fn default() -> Self { Self { left_centre: k88(), right_centre: k67(), left_rim: k90(), right_rim: k86() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchCfg {
    #[serde(default = "k37")] pub left: Vec<u32>,
    #[serde(default = "k39")] pub right: Vec<u32>,
    #[serde(default = "k16")] pub dash: Vec<u32>,
}
fn k37() -> Vec<u32> { vec![37] }
fn k39() -> Vec<u32> { vec![39] }
fn k16() -> Vec<u32> { vec![16] }
impl Default for CatchCfg {
    fn default() -> Self { Self { left: k37(), right: k39(), dash: k16() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManiaCfg {
    #[serde(rename = "4K", default = "yes")]
    pub four_k: bool,
    #[serde(rename = "key4K", default = "default_4k")]
    pub key4k: Vec<u32>,
    #[serde(rename = "key7K", default = "default_7k")]
    pub key7k: Vec<u32>,
}
fn default_4k() -> Vec<u32> { vec![68, 70, 74, 75] }
fn default_7k() -> Vec<u32> { vec![83, 68, 70, 32, 74, 75, 76] }
impl Default for ManiaCfg {
    fn default() -> Self { Self { four_k: true, key4k: default_4k(), key7k: default_7k() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MousePaw {
    #[serde(default = "paw_start")]
    pub paw_starting_point: Vec<f64>,
    #[serde(default = "paw_end")]
    pub paw_ending_point: Vec<f64>,
}
fn paw_start() -> Vec<f64> { vec![211.0, 159.0] }
fn paw_end() -> Vec<f64> { vec![258.0, 228.0] }
impl Default for MousePaw {
    fn default() -> Self { Self { paw_starting_point: paw_start(), paw_ending_point: paw_end() } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_mode")]
    pub mode: u32,
    #[serde(default)]
    pub resolution: Resolution,
    #[serde(default)]
    pub decoration: Decoration,
    #[serde(default)]
    pub osu: OsuCfg,
    #[serde(default)]
    pub taiko: TaikoCfg,
    #[serde(rename = "catch", default)]
    pub catch_cfg: CatchCfg,
    #[serde(default)]
    pub mania: ManiaCfg,
    #[serde(rename = "mousePaw", default)]
    pub mouse_paw: MousePaw,

    /// Any extra top-level keys we don't model (e.g. `"custom"`) — preserved across save.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn default_mode() -> u32 { 1 }

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        let cfg = serde_json::from_str(&text)
            .with_context(|| format!("parsing config {}", path.display()))?;
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text).with_context(|| format!("writing config {}", path.display()))?;
        Ok(())
    }
}
