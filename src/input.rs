//! Global keyboard + mouse capture via `device_query`.
//!
//! Cross-platform (Windows / macOS / Linux). On Linux it queries X11 (`XQueryKeymap` /
//! `XQueryPointer`) and needs no special permissions or `input`-group membership. On macOS the
//! user must grant Accessibility permission for global key reads to work.
//!
//! We poll once per rendered frame instead of using an event hook, which is simple and robust, and egui
//! repaints continuously for the animation anyway.

use crate::keycodes::{keycode_to_vk, MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT};
use device_query::{DeviceQuery, DeviceState};
use std::collections::HashSet;

/// One sample of global input state.
#[derive(Debug, Clone, Default)]
pub struct InputFrame {
    /// Currently held keys + mouse buttons, as numeric VK codes (see `keycodes`).
    pub pressed: HashSet<u32>,
    /// Absolute cursor position in screen pixels.
    pub cursor: (i32, i32),
}

pub struct GlobalInput {
    device: DeviceState,
}

impl GlobalInput {
    pub fn new() -> Self {
        Self {
            device: DeviceState::new(),
        }
    }

    pub fn poll(&self) -> InputFrame {
        let mut pressed = HashSet::new();

        for key in self.device.get_keys() {
            let code = keycode_to_vk(&key);
            if code != 0 {
                pressed.insert(code);
            }
        }

        let mouse = self.device.get_mouse();
        // `button_pressed` is 1-indexed in device_query: index 1 = left, 2 = right, 3 = middle.
        if mouse.button_pressed.get(1).copied().unwrap_or(false) {
            pressed.insert(MOUSE_LEFT);
        }
        if mouse.button_pressed.get(2).copied().unwrap_or(false) {
            pressed.insert(MOUSE_RIGHT);
        }
        if mouse.button_pressed.get(3).copied().unwrap_or(false) {
            pressed.insert(MOUSE_MIDDLE);
        }

        InputFrame {
            pressed,
            cursor: mouse.coords,
        }
    }
}

impl Default for GlobalInput {
    fn default() -> Self {
        Self::new()
    }
}
