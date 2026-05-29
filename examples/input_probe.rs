//! Diagnostic: confirm global mouse/keyboard capture works on this machine.
//!
//! Run with `cargo run --example input_probe`, then **move the mouse and press keys** for ~12s.
//! It prints only when the state *changes*, so you can see whether real input is detected. If
//! coordinates and pressed keys update as you move/type, Meowverlay's input path is healthy here —
//! no `input` group or elevated permissions required.

use device_query::{DeviceQuery, DeviceState};
use std::{thread, time::{Duration, Instant}};

fn main() {
    let device = DeviceState::new();
    println!("Probing global input for ~12s — MOVE THE MOUSE and PRESS KEYS now.\n");
    let start = Instant::now();
    let mut last = String::new();
    let mut changes = 0u32;
    while start.elapsed() < Duration::from_secs(12) {
        let mouse = device.get_mouse();
        let keys = device.get_keys();
        let now = format!("cursor={:?} buttons={:?} keys={:?}", mouse.coords, mouse.button_pressed, keys);
        if now != last {
            println!("[{:>5.1}s] {now}", start.elapsed().as_secs_f32());
            last = now;
            changes += 1;
        }
        thread::sleep(Duration::from_millis(30));
    }
    println!("\n{changes} state changes detected. If this is ~1 (only the first print) while you were\nmoving/typing, global polling is NOT seeing input on this session.");
}
