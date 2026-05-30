//! Diagnostic: confirm global mouse/keyboard capture works on this machine.
//!
//! Run with `cargo run --example input_probe`, then **move the mouse and press keys** for ~12s.
//!
//! On Linux this exercises the same `evdev` (`/dev/input/event*`) path Meowverlay itself uses, so it
//! works under both X11 and Wayland — provided you can read the device nodes (membership of the
//! `input` group). If keys/buttons print and the mouse-motion count climbs as you move, Meowverlay's
//! input path is healthy here. On Windows/macOS it uses `device_query` (Accessibility permission
//! required on macOS).

#[cfg(target_os = "linux")]
fn main() {
    use evdev::EventType;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    println!("Linux evdev probe — opening /dev/input devices...\n");
    let motion = Arc::new(AtomicU64::new(0));
    let mut opened = 0u32;

    for (path, mut dev) in evdev::enumerate() {
        let has_keys = dev.supported_keys().is_some();
        let has_rel = dev.supported_relative_axes().is_some();
        let has_abs = dev.supported_absolute_axes().is_some();
        if !(has_keys || has_rel || has_abs) {
            continue;
        }
        let name = dev.name().unwrap_or("?").to_string();
        println!(
            "  opened {:<20} [{name}]{}{}{}",
            path.display(),
            if has_keys { " keys" } else { "" },
            if has_rel { " rel" } else { "" },
            if has_abs { " abs" } else { "" },
        );
        opened += 1;
        let motion = Arc::clone(&motion);
        thread::spawn(move || loop {
            let events = match dev.fetch_events() {
                Ok(events) => events,
                Err(_) => return,
            };
            for ev in events {
                match ev.event_type() {
                    // KEY covers keyboard keys *and* mouse buttons (BTN_*).
                    EventType::KEY => {
                        println!("  KEY  code={:<4} value={}", ev.code(), ev.value())
                    }
                    EventType::RELATIVE | EventType::ABSOLUTE => {
                        motion.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
        });
    }

    if opened == 0 {
        eprintln!(
            "\nNo readable /dev/input devices. Add yourself to the 'input' group and re-log:\n  \
             sudo usermod -aG input $USER"
        );
        return;
    }

    println!("\nOpened {opened} device(s). MOVE THE MOUSE and PRESS KEYS now (~12s)...\n");
    thread::sleep(Duration::from_secs(12));
    let n = motion.load(Ordering::Relaxed);
    println!(
        "\n{n} mouse-motion events seen. If keys printed above and this is > 0, global input works \
         here. If both are empty while you were typing/moving, evdev is not delivering input."
    );
}

#[cfg(not(target_os = "linux"))]
fn main() {
    use device_query::{DeviceQuery, DeviceState};
    use std::{thread, time::Duration, time::Instant};

    let device = DeviceState::new();
    println!("Probing global input for ~12s: MOVE THE MOUSE and PRESS KEYS now.\n");
    let start = Instant::now();
    let mut last = String::new();
    let mut changes = 0u32;
    while start.elapsed() < Duration::from_secs(12) {
        let mouse = device.get_mouse();
        let keys = device.get_keys();
        let now = format!(
            "cursor={:?} buttons={:?} keys={:?}",
            mouse.coords, mouse.button_pressed, keys
        );
        if now != last {
            println!("[{:>5.1}s] {now}", start.elapsed().as_secs_f32());
            last = now;
            changes += 1;
        }
        thread::sleep(Duration::from_millis(30));
    }
    println!("\n{changes} state changes detected. If this is ~1 (only the first print) while you were\nmoving/typing, global polling is NOT seeing input on this session.");
}
