//! osu! Catch the Beat mode. Port of `drawCatch`.

use super::{any_pressed, unbound_key, Frame};

pub fn draw(frame: &Frame) {
    let painter = frame.painter;
    let canvas = &frame.canvas;
    let c = &frame.config.catch_cfg;

    if let Some(bg) = frame.skin.get("catch_bg") {
        canvas.draw_full(painter, bg);
    }

    let l = any_pressed(frame.pressed, &c.left);
    let r = any_pressed(frame.pressed, &c.right);
    let d = any_pressed(frame.pressed, &c.dash);
    let fallback = unbound_key(frame.pressed, &[&c.left, &c.right, &c.dash]);

    let key = if d {
        "catch_dash"
    } else if l && r {
        "catch_mid"
    } else if l {
        "catch_left"
    } else if r {
        "catch_right"
    } else if let Some(code) = fallback {
        // Any unbound key drives left/right movement by key-code parity.
        if code % 2 == 0 { "catch_left" } else { "catch_right" }
    } else {
        "catch_up"
    };
    if let Some(tex) = frame.skin.get(key) {
        canvas.draw_full(painter, tex);
    }
}
