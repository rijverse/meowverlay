// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rdev::{listen, EventType};
use std::path::PathBuf;
use std::env;
use tauri::Manager;
use base64::{Engine as _, engine::general_purpose};

#[cfg(target_os = "linux")]
use evdev::Device;

#[derive(Clone, serde::Serialize)]
struct InputPayload {
    action: String,
    key_code: u32,
}

#[derive(Clone, serde::Serialize)]
struct MousePayload {
    x: f64,
    y: f64,
}

struct MouseState {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn key_to_code(key: rdev::Key) -> u32 {
    use rdev::Key::*;
    match key {
        KeyA => 65, KeyB => 66, KeyC => 67, KeyD => 68, KeyE => 69, KeyF => 70,
        KeyG => 71, KeyH => 72, KeyI => 73, KeyJ => 74, KeyK => 75, KeyL => 76,
        KeyM => 77, KeyN => 78, KeyO => 79, KeyP => 80, KeyQ => 81, KeyR => 82,
        KeyS => 83, KeyT => 84, KeyU => 85, KeyV => 86, KeyW => 87, KeyX => 88,
        KeyY => 89, KeyZ => 90,
        Num0 => 48, Num1 => 49, Num2 => 50, Num3 => 51, Num4 => 52, Num5 => 53,
        Num6 => 54, Num7 => 55, Num8 => 56, Num9 => 57,
        Backspace => 8, Tab => 9, Return => 13, Escape => 27, Space => 32,
        PageUp => 33, PageDown => 34, End => 35, Home => 36,
        LeftArrow => 37, UpArrow => 38, RightArrow => 39, DownArrow => 40,
        Delete => 46,
        ShiftLeft => 16, ShiftRight => 16, ControlLeft => 17, ControlRight => 17,
        Alt => 18, AltGr => 18, CapsLock => 20, NumLock => 144, ScrollLock => 145,
        SemiColon => 186, Equal => 187, Comma => 188, Minus => 189, Dot => 190,
        Slash => 191, BackQuote => 192, LeftBracket => 219, BackSlash => 220,
        RightBracket => 221, Quote => 222,
        Kp0 => 96, Kp1 => 97, Kp2 => 98, Kp3 => 99, Kp4 => 100, Kp5 => 101,
        Kp6 => 102, Kp7 => 103, Kp8 => 104, Kp9 => 105,
        KpMultiply => 106, KpPlus => 107, KpMinus => 109, KpDivide => 111,
        F1 => 112, F2 => 113, F3 => 114, F4 => 115, F5 => 116, F6 => 117,
        F7 => 118, F8 => 119, F9 => 120, F10 => 121, F11 => 122, F12 => 123,
        _ => 0
    }
}

fn button_to_code(button: rdev::Button) -> u32 {
    use rdev::Button::*;
    match button {
        Left => 1,
        Right => 2,
        Middle => 4,
        _ => 0,
    }
}

#[cfg(target_os = "linux")]
fn evdev_to_keycode(code: u16) -> u32 {
    match code {
        1 => 27, // ESC
        2 => 49, 3 => 50, 4 => 51, 5 => 52, 6 => 53, 7 => 54, 8 => 55, 9 => 56, 10 => 57, 11 => 48, // 1..0
        12 => 189, // MINUS
        13 => 187, // EQUAL
        14 => 8, // BACKSPACE
        15 => 9, // TAB
        16 => 81, 17 => 87, 18 => 69, 19 => 82, 20 => 84, 21 => 89, 22 => 85, 23 => 73, 24 => 79, 25 => 80, // Q..P
        26 => 219, 27 => 221, // [ ]
        28 => 13, // ENTER
        29 => 17, // LCTRL
        30 => 65, 31 => 83, 32 => 68, 33 => 70, 34 => 71, 35 => 72, 36 => 74, 37 => 75, 38 => 76, // A..L
        39 => 186, 40 => 222, // ; '
        41 => 192, // `
        42 => 16, // LSHIFT
        43 => 220, // \
        44 => 90, 45 => 88, 46 => 67, 47 => 86, 48 => 66, 49 => 78, 50 => 77, // Z..M
        51 => 188, 52 => 190, 53 => 191, // , . /
        54 => 16, // RSHIFT
        56 => 18, // LALT
        57 => 32, // SPACE
        58 => 20, // CAPSLOCK
        97 => 17, // RCTRL
        100 => 18, // RALT
        103 => 38, // UP
        105 => 37, // LEFT
        106 => 39, // RIGHT
        108 => 40, // DOWN
        110 => 45, // INSERT
        111 => 46, // DELETE
        272 => 1, // BTN_LEFT
        273 => 2, // BTN_RIGHT
        274 => 4, // BTN_MIDDLE
        _ => 0
    }
}

fn find_skins_dir() -> PathBuf {
    if let Ok(cwd) = env::current_dir() {
        let path = cwd.join("skins");
        if path.exists() && path.is_dir() {
            return path;
        }
        if let Some(parent) = cwd.parent() {
            let path = parent.join("skins");
            if path.exists() && path.is_dir() {
                return path;
            }
        }
    }
    if let Ok(exe_path) = env::current_exe() {
        let mut dir = exe_path.parent();
        while let Some(d) = dir {
            let path = d.join("skins");
            if path.exists() && path.is_dir() {
                return path;
            }
            dir = d.parent();
        }
    }
    PathBuf::from("skins")
}

#[tauri::command]
fn get_skins() -> Result<Vec<String>, String> {
    let skins_dir = find_skins_dir();
    let mut skins = Vec::new();
    if skins_dir.exists() && skins_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(skins_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            skins.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    if skins.is_empty() {
        skins.push("default".to_string());
    }
    Ok(skins)
}

#[tauri::command]
fn read_skin_config(skin_name: String) -> Result<String, String> {
    let skins_dir = find_skins_dir();
    let file_path = skins_dir.join(&skin_name).join("config.json");
    if !file_path.exists() {
        return Err(format!("Config not found: {:?}", file_path));
    }
    std::fs::read_to_string(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_skin_image(skin_name: String, rel_path: String) -> Result<String, String> {
    let skins_dir = find_skins_dir();
    let file_path = skins_dir.join(&skin_name).join(&rel_path);
    if !file_path.exists() {
        return Err(format!("File not found: {:?}", file_path));
    }
    let bytes = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let b64 = general_purpose::STANDARD.encode(&bytes);
    let mime_type = if rel_path.ends_with(".png") {
        "image/png"
    } else if rel_path.ends_with(".jpg") || rel_path.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "image/png"
    };
    Ok(format!("data:{};base64,{}", mime_type, b64))
}

#[tauri::command]
fn write_skin_config(skin_name: String, config_str: String) -> Result<(), String> {
    let skins_dir = find_skins_dir();
    let file_path = skins_dir.join(&skin_name).join("config.json");
    std::fs::write(&file_path, config_str).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_screen_resolution(
    width: f64,
    height: f64,
    mouse_state: tauri::State<'_, std::sync::Arc<std::sync::Mutex<MouseState>>>,
) {
    let mut state = mouse_state.lock().unwrap();
    state.width = width;
    state.height = height;
    // Recenter mouse pointer inside target bounds
    state.x = width / 2.0;
    state.y = height / 2.0;
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("GDK_BACKEND", "x11");
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let mouse_state = std::sync::Arc::new(std::sync::Mutex::new(MouseState {
        x: 960.0,
        y: 540.0,
        width: 1920.0,
        height: 1080.0,
    }));
    let mouse_state_clone = mouse_state.clone();

    tauri::Builder::default()
        .manage(mouse_state)
        .setup(move |app| {
            let app_handle = app.handle();
            let last_emit = std::sync::Arc::new(std::sync::Mutex::new(std::time::Instant::now()));

            // ─── Linux Wayland/X11 raw evdev input listener ───
            #[cfg(target_os = "linux")]
            {
                let app_handle = app_handle.clone();
                let last_emit = last_emit.clone();
                let mouse_state = mouse_state_clone.clone();

                std::thread::spawn(move || {
                    let devices = match evdev::enumerate().collect::<Vec<_>>() {
                        list if !list.is_empty() => list,
                        _ => {
                            println!("[meowverlay] No evdev devices found. Global input inactive.");
                            return;
                        }
                    };

                    println!("[meowverlay] Found {} input devices for global tracking.", devices.len());

                    for (path, mut device) in devices {
                        let app_handle = app_handle.clone();
                        let last_emit = last_emit.clone();
                        let mouse_state = mouse_state.clone();

                        std::thread::spawn(move || {
                            loop {
                                match device.fetch_events() {
                                    Ok(events) => {
                                        for event in events {
                                            let et = event.event_type();
                                            if et == evdev::EventType::KEY {
                                                let code = evdev_to_keycode(event.code());
                                                if code != 0 {
                                                    let action = match event.value() {
                                                        0 => "keyup",
                                                        1 | 2 => "keydown",
                                                        _ => continue,
                                                    };
                                                    let _ = app_handle.emit_all("input-event", InputPayload {
                                                        action: action.to_string(),
                                                        key_code: code,
                                                    });
                                                }
                                            } else if et == evdev::EventType::RELATIVE {
                                                let code = event.code();
                                                let val = event.value() as f64;
                                                let mut state = mouse_state.lock().unwrap();
                                                
                                                if code == 0 { // REL_X
                                                    state.x = (state.x + val).clamp(0.0, state.width);
                                                } else if code == 1 { // REL_Y
                                                    state.y = (state.y + val).clamp(0.0, state.height);
                                                } else {
                                                    continue;
                                                }
                                                
                                                let x = state.x;
                                                let y = state.y;
                                                drop(state);

                                                let mut last = last_emit.lock().unwrap();
                                                if last.elapsed() >= std::time::Duration::from_millis(8) { // ~120Hz
                                                    *last = std::time::Instant::now();
                                                    let _ = app_handle.emit_all("mouse-move", MousePayload { x, y });
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        // Device disconnected or read failed
                                        break;
                                    }
                                }
                            }
                        });
                    }
                });
            }

            // ─── Windows & macOS rdev global hook listener ───
            #[cfg(not(target_os = "linux"))]
            {
                std::thread::spawn(move || {
                    if let Err(error) = listen(move |event| {
                        match event.event_type {
                            EventType::KeyPress(key) => {
                                let code = key_to_code(key);
                                if code != 0 {
                                    let _ = app_handle.emit_all("input-event", InputPayload {
                                        action: "keydown".to_string(),
                                        key_code: code,
                                    });
                                }
                            }
                            EventType::KeyRelease(key) => {
                                let code = key_to_code(key);
                                if code != 0 {
                                    let _ = app_handle.emit_all("input-event", InputPayload {
                                        action: "keyup".to_string(),
                                        key_code: code,
                                    });
                                }
                            }
                            EventType::ButtonPress(button) => {
                                let code = button_to_code(button);
                                if code != 0 {
                                    let _ = app_handle.emit_all("input-event", InputPayload {
                                        action: "keydown".to_string(),
                                        key_code: code,
                                    });
                                }
                            }
                            EventType::ButtonRelease(button) => {
                                let code = button_to_code(button);
                                if code != 0 {
                                    let _ = app_handle.emit_all("input-event", InputPayload {
                                        action: "keyup".to_string(),
                                        key_code: code,
                                    });
                                }
                            }
                            EventType::MouseMove { x, y } => {
                                let mut last = last_emit.lock().unwrap();
                                if last.elapsed() >= std::time::Duration::from_millis(8) { // ~120Hz cap
                                    *last = std::time::Instant::now();
                                    let _ = app_handle.emit_all("mouse-move", MousePayload { x, y });
                                }
                            }
                            _ => {}
                        }
                    }) {
                        println!("Error listening to inputs: {:?}", error);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_skins,
            read_skin_config,
            read_skin_image,
            write_skin_config,
            update_screen_resolution
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
