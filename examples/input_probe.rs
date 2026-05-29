//! Diagnostic: confirm global mouse/keyboard capture works on this machine.
//!
//! Run with `cargo run --example input_probe`, then move the mouse and press keys for a few
//! seconds. If coordinates change and pressed keys are listed, Meowverlay's input path is healthy
//! on your system — no `input` group or elevated permissions required.

use device_query::{DeviceQuery, DeviceState};
use std::{thread, time::Duration};

fn main() {
    let device = DeviceState::new();
    println!("Probing global input for ~3s — move the mouse and press some keys…\n");
    for _ in 0..30 {
        let mouse = device.get_mouse();
        let keys = device.get_keys();
        println!("cursor = {:?}   buttons = {:?}   keys = {:?}", mouse.coords, mouse.button_pressed, keys);
        thread::sleep(Duration::from_millis(100));
    }
}
