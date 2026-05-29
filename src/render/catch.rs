//! osu! Catch the Beat mode. Port of `drawCatch`.

use super::{any_pressed, Frame};

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

    let key = if d {
        "catch_dash"
    } else if l && r {
        "catch_mid"
    } else if l {
        "catch_left"
    } else if r {
        "catch_right"
    } else {
        "catch_up"
    };
    if let Some(tex) = frame.skin.get(key) {
        canvas.draw_full(painter, tex);
    }
}
