//! Canvas rendering. Ports the four `draw*` routines from the previous TypeScript Canvas code to
//! the egui `Painter`. All sprite coordinates are in "skin pixels" (the native size of the skin's
//! background image), while [`Canvas`] maps those into on-screen points and handles the left-handed mirror.

mod catch;
mod mania;
mod standard;
mod taiko;

use crate::config::Config;
use crate::skin::Skin;
use egui::{Color32, Pos2, Rect, Vec2};
use std::collections::HashSet;

/// Maps skin-pixel coordinates onto the on-screen panel, applying the horizontal mirror for
/// left-handed mode.
pub struct Canvas {
    /// Panel rectangle on screen (points).
    pub rect: Rect,
    /// Screen points per skin pixel.
    pub scale: Vec2,
    /// Skin canvas size in pixels.
    pub size: Vec2,
    /// Mirror horizontally (left-handed layout).
    pub mirror: bool,
}

impl Canvas {
    pub fn new(rect: Rect, skin_size: Vec2, mirror: bool) -> Self {
        let scale = vec2_div(rect.size(), skin_size);
        Self {
            rect,
            scale,
            size: skin_size,
            mirror,
        }
    }

    /// Map a skin-space point to an on-screen point.
    fn map(&self, x: f32, y: f32) -> Pos2 {
        let sx = if self.mirror { self.size.x - x } else { x };
        Pos2::new(
            self.rect.min.x + sx * self.scale.x,
            self.rect.min.y + y * self.scale.y,
        )
    }

    fn uv_full(&self) -> Rect {
        if self.mirror {
            Rect::from_min_max(Pos2::new(1.0, 0.0), Pos2::new(0.0, 1.0))
        } else {
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0))
        }
    }

    /// Draw a full-canvas sprite (background / paw frame), stretched to fill the panel.
    fn draw_full(&self, painter: &egui::Painter, tex: &egui::TextureHandle) {
        painter.image(tex.id(), self.rect, self.uv_full(), Color32::WHITE);
    }

    /// Draw a sprite of native pixel size `(w, h)` centered at skin-space `(cx, cy)`.
    fn draw_sprite(&self, painter: &egui::Painter, tex: &egui::TextureHandle, cx: f32, cy: f32) {
        let [w, h] = tex.size();
        let size = Vec2::new(w as f32 * self.scale.x, h as f32 * self.scale.y);
        let rect = Rect::from_center_size(self.map(cx, cy), size);
        painter.image(tex.id(), rect, self.uv_full(), Color32::WHITE);
    }
}

fn vec2_div(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x / b.x.max(1.0), a.y / b.y.max(1.0))
}

/// A single smoke puff particle.
#[derive(Clone, Copy)]
pub struct SmokeParticle {
    pub x: f32,
    pub y: f32,
    pub alpha: f32,
    pub size: f32,
}

/// Mutable animation state that persists across frames (smoke trail + smoke toggle edge tracking).
#[derive(Default)]
pub struct AnimState {
    pub smoke: Vec<SmokeParticle>,
    pub smoke_toggled: bool,
    pub smoke_key_was_down: bool,
    /// Fractional carry for the dt-scaled smoke spawner, so puff density is frame-rate independent.
    pub smoke_spawn_accum: f32,
}

/// Everything a per-mode draw routine needs for one frame.
pub struct Frame<'a> {
    pub painter: &'a egui::Painter,
    pub canvas: Canvas,
    pub config: &'a Config,
    pub skin: &'a Skin,
    pub pressed: &'a HashSet<u32>,
    /// Cursor position normalized to [0, 1] across the screen.
    pub cursor_norm: (f32, f32),
    /// Spike-clamped frame time in seconds, for frame-rate-independent animation.
    pub dt: f32,
}

/// Returns true if any of `keys` is currently held.
fn any_pressed(pressed: &HashSet<u32>, keys: &[u32]) -> bool {
    keys.iter().any(|k| pressed.contains(k))
}

/// Draw the current game mode.
pub fn draw(frame: &Frame, anim: &mut AnimState) {
    match frame.config.mode {
        2 => taiko::draw(frame),
        3 => catch::draw(frame),
        4 => mania::draw(frame),
        _ => standard::draw(frame, anim),
    }
}
