//! osu! Taiko mode. Port of `drawTaiko`.

use super::{any_pressed, Frame};

pub fn draw(frame: &Frame) {
    let painter = frame.painter;
    let canvas = &frame.canvas;
    let t = &frame.config.taiko;

    if let Some(bg) = frame.skin.get("taiko_bg") {
        canvas.draw_full(painter, bg);
    }

    let lc = any_pressed(frame.pressed, &t.left_centre);
    let lr = any_pressed(frame.pressed, &t.left_rim);
    let rc = any_pressed(frame.pressed, &t.right_centre);
    let rr = any_pressed(frame.pressed, &t.right_rim);

    let left = if lr {
        "taiko_leftrim"
    } else if lc {
        "taiko_leftcentre"
    } else {
        "taiko_leftup"
    };
    if let Some(tex) = frame.skin.get(left) {
        canvas.draw_full(painter, tex);
    }

    let right = if rr {
        "taiko_rightrim"
    } else if rc {
        "taiko_rightcentre"
    } else {
        "taiko_rightup"
    };
    if let Some(tex) = frame.skin.get(right) {
        canvas.draw_full(painter, tex);
    }
}
