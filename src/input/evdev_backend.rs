//! Linux global input via `evdev` (`/dev/input/event*`).
//!
//! Reads the kernel input devices directly, below the display server, so it works identically on
//! X11 **and** Wayland. This is what makes the overlay react on Wayland at all: Wayland deliberately
//! hides global keyboard state and the pointer from X11 clients, so the `device_query` (Xlib) path
//! saw nothing there. The trade-off is that it needs read access to the device nodes (membership of
//! the `input` group, the usual desktop default).
//!
//! One blocking reader thread per device feeds a shared snapshot that `poll()` copies each frame.
//! Keys are tracked as press/release edges into a held-set, which reproduces the level-based
//! `pressed` semantics the rest of the app expects. The cursor is relative-accumulated for mice and
//! absolute-mapped for tablets and touchpads, clamped/scaled to the configured screen resolution.
//! Wayland does not expose the real pointer position to unprivileged clients, so mouse tracking is
//! an approximation (raw counts, no pointer acceleration); tablet/touchpad absolute tracking maps
//! the device surface onto the screen. (Modern I2C precision touchpads expose a relative "Mouse"
//! node that stays silent — all motion arrives as absolute multitouch — so reading their absolute
//! axes is the only way the cursor moves there at all.)

use super::InputFrame;
use crate::keycodes::{MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT};
use evdev::{AbsoluteAxisCode, Device, EventType, KeyCode};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

// Relative/absolute axis codes from linux/input-event-codes.h. X is axis 0, Y is axis 1 for both
// REL_* and ABS_*; `get_abs_state()` returns an array indexed by these same codes.
const AXIS_X: u16 = 0;
const AXIS_Y: u16 = 1;
/// `BTN_TOOL_PEN` / `BTN_TOOL_FINGER` (linux/input-event-codes.h). A pen marks a graphics
/// tablet/stylus; a finger marks a touchpad. Either tool key flags a device whose `ABS_X/ABS_Y` are
/// a screen-mappable pointer position, as opposed to a joystick/gamepad (which also exposes
/// `ABS_X/ABS_Y` but advertises neither tool key, so it's skipped).
const BTN_TOOL_PEN: u16 = 0x140;
const BTN_TOOL_FINGER: u16 = 0x145;

/// Cursor position and the resolution used to clamp/scale it, all in screen-pixel space.
struct Shared {
    /// VK codes currently held (keys + mouse buttons).
    pressed: HashSet<u32>,
    cursor_x: f64,
    cursor_y: f64,
    res_w: f64,
    res_h: f64,
}

/// Min + span of an absolute axis, used to map tablet coordinates into [0, res].
#[derive(Clone, Copy)]
struct AbsRange {
    min: f64,
    span: f64,
}

pub struct EvdevInput {
    shared: Arc<Mutex<Shared>>,
}

impl EvdevInput {
    pub fn new() -> anyhow::Result<Self> {
        // Default to 1080p centred until the first `set_resolution` arrives, so the paw starts in a
        // sane place rather than the top-left corner.
        let shared = Arc::new(Mutex::new(Shared {
            pressed: HashSet::new(),
            cursor_x: 960.0,
            cursor_y: 540.0,
            res_w: 1920.0,
            res_h: 1080.0,
        }));

        let mut opened = 0usize;
        for (_path, dev) in evdev::enumerate() {
            if !is_input_device(&dev) {
                continue;
            }
            let abs = abs_ranges(&dev);
            let shared = Arc::clone(&shared);
            if thread::Builder::new()
                .name("meowverlay-evdev".into())
                .spawn(move || reader_loop(dev, shared, abs))
                .is_ok()
            {
                opened += 1;
            }
        }

        if opened == 0 {
            anyhow::bail!("no readable /dev/input devices (is the user in the 'input' group?)");
        }
        Ok(Self { shared })
    }

    pub fn set_resolution(&self, width: f64, height: f64) {
        let mut s = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        s.res_w = width.max(1.0);
        s.res_h = height.max(1.0);
    }

    pub fn poll(&self) -> InputFrame {
        let s = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        InputFrame {
            pressed: s.pressed.clone(),
            cursor: (s.cursor_x as i32, s.cursor_y as i32),
        }
    }
}

/// Keyboards, mice and tablets all advertise keys and/or pointer axes; everything else (lid
/// switches, power buttons with no usable keys, etc.) we skip so we don't spawn idle threads.
fn is_input_device(dev: &Device) -> bool {
    dev.supported_keys().is_some()
        || dev.supported_relative_axes().is_some()
        || dev.supported_absolute_axes().is_some()
}

/// Read the absolute X/Y ranges for a pen tablet or touchpad — devices whose `ABS_X/ABS_Y` map onto
/// a screen position. `None` for relative mice and keyboards (no absolute axes) and for
/// joysticks/gamepads (absolute axes but no pointer tool key).
fn abs_ranges(dev: &Device) -> Option<(AbsRange, AbsRange)> {
    let axes = dev.supported_absolute_axes()?;
    if !axes.contains(AbsoluteAxisCode(AXIS_X)) || !axes.contains(AbsoluteAxisCode(AXIS_Y)) {
        return None;
    }
    // A pen (tablet) or finger (touchpad) tool key marks the absolute axes as a screen pointer; skip
    // joysticks/gamepads, which also expose ABS_X/ABS_Y but aren't pointers.
    let is_pointer = dev.supported_keys().is_some_and(|keys| {
        keys.contains(KeyCode(BTN_TOOL_PEN)) || keys.contains(KeyCode(BTN_TOOL_FINGER))
    });
    if !is_pointer {
        return None;
    }
    // `get_abs_state` returns the kernel `input_absinfo` array indexed by axis code; we read the
    // public `minimum`/`maximum` fields directly.
    let state = dev.get_abs_state().ok()?;
    let x = state[AXIS_X as usize];
    let y = state[AXIS_Y as usize];
    Some((
        AbsRange {
            min: x.minimum as f64,
            span: ((x.maximum - x.minimum) as f64).max(1.0),
        },
        AbsRange {
            min: y.minimum as f64,
            span: ((y.maximum - y.minimum) as f64).max(1.0),
        },
    ))
}

fn reader_loop(mut dev: Device, shared: Arc<Mutex<Shared>>, abs: Option<(AbsRange, AbsRange)>) {
    loop {
        // Blocks until the kernel has events for this device, so an idle device costs no CPU.
        let events = match dev.fetch_events() {
            Ok(events) => events,
            // Device unplugged or read error: this thread is done. Other devices keep working.
            Err(_) => return,
        };
        let mut s = shared.lock().unwrap_or_else(|e| e.into_inner());
        for ev in events {
            match ev.event_type() {
                EventType::KEY => {
                    if let Some(vk) = evdev_code_to_vk(ev.code()) {
                        // value 0 = release, 1 = press, 2 = autorepeat.
                        if ev.value() == 0 {
                            s.pressed.remove(&vk);
                        } else {
                            s.pressed.insert(vk);
                        }
                    }
                }
                EventType::RELATIVE => match ev.code() {
                    AXIS_X => s.cursor_x = (s.cursor_x + ev.value() as f64).clamp(0.0, s.res_w),
                    AXIS_Y => s.cursor_y = (s.cursor_y + ev.value() as f64).clamp(0.0, s.res_h),
                    _ => {}
                },
                EventType::ABSOLUTE => {
                    if let Some((rx, ry)) = abs {
                        match ev.code() {
                            AXIS_X => {
                                let f = ((ev.value() as f64 - rx.min) / rx.span).clamp(0.0, 1.0);
                                s.cursor_x = f * s.res_w;
                            }
                            AXIS_Y => {
                                let f = ((ev.value() as f64 - ry.min) / ry.span).clamp(0.0, 1.0);
                                s.cursor_y = f * s.res_h;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Map a Linux `KEY_*` / `BTN_*` code (linux/input-event-codes.h) to the numeric VK/JS code used in
/// config files — the same numbers `keycodes::keycode_to_vk` produces, so evdev and the X11 fallback
/// bind identically. Returns `None` for codes we don't bind.
fn evdev_code_to_vk(code: u16) -> Option<u32> {
    Some(match code {
        // Letters
        30 => 65, // A
        48 => 66, // B
        46 => 67, // C
        32 => 68, // D
        18 => 69, // E
        33 => 70, // F
        34 => 71, // G
        35 => 72, // H
        23 => 73, // I
        36 => 74, // J
        37 => 75, // K
        38 => 76, // L
        50 => 77, // M
        49 => 78, // N
        24 => 79, // O
        25 => 80, // P
        16 => 81, // Q
        19 => 82, // R
        31 => 83, // S
        20 => 84, // T
        22 => 85, // U
        47 => 86, // V
        17 => 87, // W
        45 => 88, // X
        21 => 89, // Y
        44 => 90, // Z

        // Top-row digits (KEY_1..KEY_9 = 2..10, KEY_0 = 11)
        2 => 49,
        3 => 50,
        4 => 51,
        5 => 52,
        6 => 53,
        7 => 54,
        8 => 55,
        9 => 56,
        10 => 57,
        11 => 48,

        // Whitespace / editing
        14 => 8,   // Backspace
        15 => 9,   // Tab
        28 => 13,  // Enter
        1 => 27,   // Esc
        57 => 32,  // Space
        58 => 20,  // CapsLock
        111 => 46, // Delete
        110 => 45, // Insert
        102 => 36, // Home
        107 => 35, // End
        104 => 33, // PageUp
        109 => 34, // PageDown

        // Arrows
        105 => 37, // Left
        103 => 38, // Up
        106 => 39, // Right
        108 => 40, // Down

        // Modifiers (left/right collapse to one VK, matching browser keyCodes)
        42 | 54 => 16,  // Shift
        29 | 97 => 17,  // Ctrl
        56 | 100 => 18, // Alt
        125 => 91,      // Left Meta/Super -> Win
        126 => 92,      // Right Meta/Super -> Win

        // Function keys (F1..F10 = 59..68, F11/F12 = 87/88, F13..F20 = 183..190)
        59 => 112,
        60 => 113,
        61 => 114,
        62 => 115,
        63 => 116,
        64 => 117,
        65 => 118,
        66 => 119,
        67 => 120,
        68 => 121,
        87 => 122,
        88 => 123,
        183 => 124,
        184 => 125,
        185 => 126,
        186 => 127,
        187 => 128,
        188 => 129,
        189 => 130,
        190 => 131,

        // Punctuation (US layout VK codes)
        39 => 186, // ;
        13 => 187, // =
        51 => 188, // ,
        12 => 189, // -
        52 => 190, // .
        53 => 191, // /
        41 => 192, // `
        26 => 219, // [
        43 => 220, // \
        27 => 221, // ]
        40 => 222, // '

        // Numpad
        82 => 96,   // KP0
        79 => 97,   // KP1
        80 => 98,   // KP2
        81 => 99,   // KP3
        75 => 100,  // KP4
        76 => 101,  // KP5
        77 => 102,  // KP6
        71 => 103,  // KP7
        72 => 104,  // KP8
        73 => 105,  // KP9
        55 => 106,  // KP*
        78 => 107,  // KP+
        74 => 109,  // KP-
        83 => 110,  // KP.
        98 => 111,  // KP/
        117 => 146, // KP=
        96 => 13,   // KP Enter -> Enter

        // Mouse buttons (BTN_LEFT/RIGHT/MIDDLE = 0x110/0x111/0x112)
        0x110 => MOUSE_LEFT,
        0x111 => MOUSE_RIGHT,
        0x112 => MOUSE_MIDDLE,

        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keycodes::vk_to_label;

    /// Every code we translate must yield a labelled VK, so the settings UI never shows a bare
    /// number for a bindable key. Spot-checks representative codes from each group.
    #[test]
    fn mapped_codes_have_labels() {
        let codes = [
            30, 11, 2, 28, 1, 57, 42, 29, 56, 125, 59, 88, 183, 190, 39, 96, 117, 0x110, 0x111,
            0x112,
        ];
        for c in codes {
            let vk = evdev_code_to_vk(c).unwrap_or_else(|| panic!("code {c} should map"));
            let label = vk_to_label(vk);
            assert!(
                !label.starts_with("Code "),
                "code {c} (vk {vk}) lacks a label"
            );
            assert_ne!(label, "None", "code {c} (vk {vk}) labelled None");
        }
    }

    /// Unmapped codes return None rather than a bogus binding.
    #[test]
    fn unmapped_code_is_none() {
        assert_eq!(evdev_code_to_vk(0xFFFF), None);
    }
}
