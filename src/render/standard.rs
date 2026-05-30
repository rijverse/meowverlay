//! osu! standard mode: keyboard paw frames, a procedural arm following the cursor, the mouse/tablet
//! sprite, and the smoke trail. Port of `drawStandard` from the previous TypeScript implementation.

use super::{any_pressed, AnimState, Frame, SmokeParticle};
use egui::{Color32, Pos2, Stroke};

/// Smoke puffs spawned per second while active. Matches the legacy "one puff per frame at 60 fps".
const SMOKE_SPAWN_PER_SEC: f32 = 60.0;
/// Alpha lost per second by each puff (legacy 0.015/frame × 60 fps), giving a ~1.1 s lifetime.
const SMOKE_FADE_PER_SEC: f32 = 0.9;

pub fn draw(frame: &Frame, anim: &mut AnimState) {
    let painter = frame.painter;
    let canvas = &frame.canvas;
    let osu = &frame.config.osu;
    let is_mouse = osu.mouse;

    // Background.
    let bg_key = if is_mouse { "mousebg" } else { "tabletbg" };
    if let Some(bg) = frame.skin.get(bg_key) {
        canvas.draw_full(painter, bg);
    }

    let k1 = any_pressed(frame.pressed, &osu.key1);
    let k2 = any_pressed(frame.pressed, &osu.key2);
    let wave_active = any_pressed(frame.pressed, &osu.wave);

    // Smoke toggle edge detection (hold vs. toggle behavior).
    let smoke_down = any_pressed(frame.pressed, &osu.smoke);
    if smoke_down && !anim.smoke_key_was_down {
        if osu.toggle_smoke {
            anim.smoke_toggled = !anim.smoke_toggled;
        } else {
            anim.smoke_toggled = true;
        }
    } else if !smoke_down && anim.smoke_key_was_down && !osu.toggle_smoke {
        anim.smoke_toggled = false;
    }
    anim.smoke_key_was_down = smoke_down;

    // Map the cursor into skin space for the arm endpoint / tool position.
    let dec = &frame.config.decoration;
    let mp = &frame.config.mouse_paw;
    let idx = if is_mouse { 0usize } else { 1usize };
    let pe = &mp.paw_ending_point;
    let off_x = dec.offset_x.get(idx).copied().unwrap_or(0.0);
    let off_y = dec.offset_y.get(idx).copied().unwrap_or(0.0);
    let s = dec.scalar.get(idx).copied().unwrap_or(1.0) as f32;
    let cx = pe
        .get(idx * 2)
        .or_else(|| pe.first())
        .copied()
        .unwrap_or(258.0) as f32
        + off_x as f32;
    let cy = pe
        .get(idx * 2 + 1)
        .or_else(|| pe.get(1))
        .copied()
        .unwrap_or(228.0) as f32
        + off_y as f32;
    let (nx, ny) = frame.cursor_norm;
    let rx = if is_mouse { 88.0 } else { 90.0 };
    let ry = if is_mouse { 52.0 } else { 55.0 };
    let mx = cx + (nx - 0.5) * rx * s;
    let my = cy + (ny - 0.5) * ry * s;

    // Spawn smoke puffs at the cursor while active, at a fixed per-second rate (dt-scaled with a
    // fractional carry) so density doesn't change with the frame rate.
    if anim.smoke_toggled {
        anim.smoke_spawn_accum += SMOKE_SPAWN_PER_SEC * frame.dt;
        while anim.smoke_spawn_accum >= 1.0 {
            anim.smoke_spawn_accum -= 1.0;
            anim.smoke.push(SmokeParticle {
                x: mx,
                y: my,
                alpha: 1.0,
                size: 5.0 + fastrand_f32() * 4.0,
            });
        }
    } else {
        anim.smoke_spawn_accum = 0.0;
    }

    // Advance + draw the smoke trail.
    let fade = SMOKE_FADE_PER_SEC * frame.dt;
    anim.smoke.retain_mut(|p| {
        p.alpha -= fade;
        if p.alpha <= 0.0 {
            return false;
        }
        let center = canvas.map(p.x, p.y);
        let radius = p.size * canvas.scale.x;
        let a = (p.alpha * 0.7 * 255.0) as u8;
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(140, 140, 150, a),
        );
        true
    });

    // Left paw frame selection (priority matches the original, plus any-key fallback).
    let fallback = super::unbound_key(
        frame.pressed,
        &[&osu.key1, &osu.key2, &osu.smoke, &osu.wave],
    );
    let left_paw_key = if wave_active {
        "wave"
    } else if anim.smoke_toggled {
        "smoke"
    } else if k1 {
        "osu_left"
    } else if k2 {
        "osu_right"
    } else if let Some(code) = fallback {
        // Any unbound key alternates the paw frame by key-code parity, for visual variety.
        if code % 2 == 0 {
            "osu_left"
        } else {
            "osu_right"
        }
    } else {
        "osu_up"
    };
    if let Some(paw) = frame.skin.get(left_paw_key) {
        canvas.draw_full(painter, paw);
    }

    // Procedural arm: a quadratic curve from the shoulder to the cursor, stroked twice
    // (thick edge color, then thin fill color).
    let ps = &mp.paw_starting_point;
    let x0 = ps
        .get(idx * 2)
        .or_else(|| ps.first())
        .copied()
        .unwrap_or(211.0) as f32;
    let y0 = ps
        .get(idx * 2 + 1)
        .or_else(|| ps.get(1))
        .copied()
        .unwrap_or(159.0) as f32;
    let c1x = x0 + (mx - x0) * 0.1 - (my - y0) * 0.2;
    let c1y = y0 + (my - y0) * 0.8 + (mx - x0) * 0.2;

    let p0 = canvas.map(x0, y0);
    let p1 = canvas.map(c1x, c1y);
    let p2 = canvas.map(mx, my);
    let edge = rgb(&osu.paw_edge);
    let fill = rgb(&osu.paw);
    let w = canvas.scale.x;
    quad(painter, [p0, p1, p2], 14.0 * w, edge);
    quad(painter, [p0, p1, p2], 8.0 * w, fill);

    // Tool sprite (mouse or tablet pen) centered at the cursor point.
    let tool_key = if is_mouse { "mouse" } else { "tablet" };
    if let Some(tool) = frame.skin.get(tool_key) {
        if tool.size()[0] > 1 {
            canvas.draw_sprite(painter, tool, mx, my);
        }
    }
}

fn quad(painter: &egui::Painter, points: [Pos2; 3], width: f32, color: Color32) {
    let shape = egui::epaint::QuadraticBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        Stroke::new(width, color),
    );
    painter.add(shape);
}

fn rgb(v: &[u8]) -> Color32 {
    Color32::from_rgb(
        v.first().copied().unwrap_or(0),
        v.get(1).copied().unwrap_or(0),
        v.get(2).copied().unwrap_or(0),
    )
}

/// Tiny xorshift-based PRNG so we don't pull in the `rand` crate just for smoke jitter.
fn fastrand_f32() -> f32 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u32> = const { Cell::new(0x9E37_79B9) };
    }
    STATE.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        s.set(x);
        (x as f32) / (u32::MAX as f32)
    })
}
