//! Translation between `device_query::Keycode` and the numeric key codes used in the
//! `bongocat-osu` `config.json` format (JavaScript / Windows virtual-key codes, e.g. A = 65).
//!
//! Keeping this convention is what makes existing bongocat-osu skins drop-in compatible.

use device_query::Keycode;

/// Mouse button codes, matching the convention used by the previous implementation.
pub const MOUSE_LEFT: u32 = 1;
pub const MOUSE_RIGHT: u32 = 2;
pub const MOUSE_MIDDLE: u32 = 4;

/// Map a `device_query` keycode to the numeric VK/JS code used in config files.
/// Returns 0 for keys we don't track.
pub fn keycode_to_vk(key: &Keycode) -> u32 {
    use Keycode::*;
    match key {
        // Letters A..Z -> 65..90
        A => 65, B => 66, C => 67, D => 68, E => 69, F => 70, G => 71, H => 72,
        I => 73, J => 74, K => 75, L => 76, M => 77, N => 78, O => 79, P => 80,
        Q => 81, R => 82, S => 83, T => 84, U => 85, V => 86, W => 87, X => 88,
        Y => 89, Z => 90,

        // Top-row digits 0..9 -> 48..57
        Key0 => 48, Key1 => 49, Key2 => 50, Key3 => 51, Key4 => 52,
        Key5 => 53, Key6 => 54, Key7 => 55, Key8 => 56, Key9 => 57,

        // Whitespace / editing
        Backspace => 8, Tab => 9, Enter => 13, Escape => 27, Space => 32,
        CapsLock => 20, Delete => 46, Insert => 45, Home => 36, End => 35,
        PageUp => 33, PageDown => 34,

        // Arrows
        Left => 37, Up => 38, Right => 39, Down => 40,

        // Modifiers (left/right collapse to the same VK, matching browser keyCodes)
        LShift | RShift => 16,
        LControl | RControl => 17,
        LAlt | RAlt => 18,

        // Function keys
        F1 => 112, F2 => 113, F3 => 114, F4 => 115, F5 => 116, F6 => 117,
        F7 => 118, F8 => 119, F9 => 120, F10 => 121, F11 => 122, F12 => 123,

        // Punctuation (US layout VK codes)
        Semicolon => 186, Equal => 187, Comma => 188, Minus => 189, Dot => 190,
        Slash => 191, Grave => 192, LeftBracket => 219, BackSlash => 220,
        RightBracket => 221, Apostrophe => 222,

        // Numpad digits -> 96..105
        Numpad0 => 96, Numpad1 => 97, Numpad2 => 98, Numpad3 => 99, Numpad4 => 100,
        Numpad5 => 101, Numpad6 => 102, Numpad7 => 103, Numpad8 => 104, Numpad9 => 105,
        NumpadMultiply => 106, NumpadAdd => 107, NumpadSubtract => 109, NumpadDivide => 111,

        _ => 0,
    }
}

/// Human-readable label for a numeric VK code, for the settings UI.
/// Port of the previous TypeScript `getKeyName`.
pub fn vk_to_label(code: u32) -> String {
    // Letters and digits map directly to their ASCII character.
    if (65..=90).contains(&code) || (48..=57).contains(&code) {
        if let Some(c) = char::from_u32(code) {
            return c.to_string();
        }
    }
    let s = match code {
        1 => "M1", 2 => "M2", 4 => "M3",
        8 => "Backspace", 9 => "Tab", 13 => "Enter", 16 => "Shift", 17 => "Ctrl",
        18 => "Alt", 20 => "CapsLock", 27 => "Esc", 32 => "Space", 33 => "PageUp",
        34 => "PageDown", 35 => "End", 36 => "Home", 37 => "←", 38 => "↑", 39 => "→",
        40 => "↓", 45 => "Insert", 46 => "Delete",
        96 => "Num0", 97 => "Num1", 98 => "Num2", 99 => "Num3", 100 => "Num4",
        101 => "Num5", 102 => "Num6", 103 => "Num7", 104 => "Num8", 105 => "Num9",
        106 => "Num*", 107 => "Num+", 109 => "Num-", 111 => "Num/",
        112 => "F1", 113 => "F2", 114 => "F3", 115 => "F4", 116 => "F5", 117 => "F6",
        118 => "F7", 119 => "F8", 120 => "F9", 121 => "F10", 122 => "F11", 123 => "F12",
        186 => ";", 187 => "=", 188 => ",", 189 => "-", 190 => ".", 191 => "/",
        192 => "`", 219 => "[", 220 => "\\", 221 => "]", 222 => "'",
        0 => "None",
        _ => return format!("Code {code}"),
    };
    s.to_string()
}
