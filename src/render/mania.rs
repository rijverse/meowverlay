//! osu! Mania mode (4K / 7K). Port of `drawMania`.

use super::Frame;

pub fn draw(frame: &Frame) {
    let painter = frame.painter;
    let canvas = &frame.canvas;
    let m = &frame.config.mania;
    let is_4k = m.four_k;

    let bg = if is_4k { "mania_bg_4K" } else { "mania_bg_7K" };
    if let Some(tex) = frame.skin.get(bg) {
        canvas.draw_full(painter, tex);
    }

    let keys: &[u32] = if is_4k { &m.key4k } else { &m.key7k };
    let prefix = if is_4k { "key_4K_" } else { "key_7K_" };

    // Per-column "key lit" overlays.
    for (i, key) in keys.iter().enumerate() {
        if frame.pressed.contains(key) {
            if let Some(tex) = frame.skin.get(&format!("{prefix}{i}")) {
                canvas.draw_full(painter, tex);
            }
        }
    }

    // Hand frames.
    let mut left = "mania_leftup";
    let mut right = "mania_rightup";
    let held = |idx: usize| keys.get(idx).map(|k| frame.pressed.contains(k)).unwrap_or(false);

    if is_4k {
        if held(0) {
            left = "mania_left0";
        } else if held(1) {
            left = "mania_left1";
        }
        if held(2) {
            right = "mania_right0";
        } else if held(3) {
            right = "mania_right1";
        }
    } else {
        if held(0) {
            left = "mania_left0";
        } else if held(1) {
            left = "mania_left1";
        } else if held(2) || held(3) {
            left = "mania_left2";
        }
        if held(4) {
            right = "mania_right0";
        } else if held(5) {
            right = "mania_right1";
        } else if held(6) {
            right = "mania_right2";
        }
    }

    if let Some(tex) = frame.skin.get(left) {
        canvas.draw_full(painter, tex);
    }
    if let Some(tex) = frame.skin.get(right) {
        canvas.draw_full(painter, tex);
    }
}
