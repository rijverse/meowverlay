//! Global keyboard + mouse capture.
//!
//! Two backends:
//! * **Linux** uses [`evdev_backend`] (`/dev/input/event*`), which works identically under X11 and
//!   Wayland. It is required because Wayland deliberately hides global keyboard state and the
//!   pointer from X11 clients, so the `device_query` (Xlib) path below sees nothing under a Wayland
//!   session. Reading evdev needs membership of the `input` group (the usual desktop default). If no
//!   device can be opened we fall back to `device_query` and print a hint.
//! * **Windows / macOS** keep `device_query`. On macOS the user must grant Accessibility permission.
//!
//! We poll once per rendered frame instead of using an event hook: simple and robust, and egui
//! repaints continuously for the animation anyway. (The evdev backend *is* event-driven internally,
//! but still exposes a level-based snapshot here so the call site is identical across platforms.)

use crate::keycodes::{keycode_to_vk, MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT};
use device_query::{DeviceQuery, DeviceState};
use std::collections::HashSet;

#[cfg(target_os = "linux")]
mod evdev_backend;

/// One sample of global input state.
#[derive(Debug, Clone, Default)]
pub struct InputFrame {
    /// Currently held keys + mouse buttons, as numeric VK codes (see `keycodes`).
    pub pressed: HashSet<u32>,
    /// Absolute cursor position in screen pixels. On the evdev backend this is already clamped to
    /// the configured resolution (see [`GlobalInput::set_resolution`]).
    pub cursor: (i32, i32),
}

pub struct GlobalInput {
    backend: Backend,
}

enum Backend {
    #[cfg(target_os = "linux")]
    Evdev(evdev_backend::EvdevInput),
    DeviceQuery(DeviceState),
}

impl GlobalInput {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            match evdev_backend::EvdevInput::new() {
                Ok(ev) => {
                    return Self {
                        backend: Backend::Evdev(ev),
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[meowverlay] evdev input unavailable ({e}); falling back to X11 polling, \
                         which does not work under Wayland. To enable global input add yourself to \
                         the 'input' group and re-log: sudo usermod -aG input $USER"
                    );
                }
            }
        }
        Self {
            backend: Backend::DeviceQuery(DeviceState::new()),
        }
    }

    /// Set the screen resolution used to clamp/scale the cursor. Only the evdev backend needs this
    /// (it has no absolute pointer to read). For `device_query` it is a no-op.
    pub fn set_resolution(&self, width: f64, height: f64) {
        match &self.backend {
            #[cfg(target_os = "linux")]
            Backend::Evdev(ev) => ev.set_resolution(width, height),
            Backend::DeviceQuery(_) => {
                let _ = (width, height);
            }
        }
    }

    pub fn poll(&self) -> InputFrame {
        match &self.backend {
            #[cfg(target_os = "linux")]
            Backend::Evdev(ev) => ev.poll(),
            Backend::DeviceQuery(device) => poll_device_query(device),
        }
    }
}

/// Poll the cross-platform `device_query` backend (X11 on Linux, native on Windows/macOS).
fn poll_device_query(device: &DeviceState) -> InputFrame {
    let mut pressed = HashSet::new();

    for key in device.get_keys() {
        let code = keycode_to_vk(&key);
        if code != 0 {
            pressed.insert(code);
        }
    }

    let mouse = device.get_mouse();
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

impl Default for GlobalInput {
    fn default() -> Self {
        Self::new()
    }
}
