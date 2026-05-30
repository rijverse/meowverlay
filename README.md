# Meowverlay 🐱

A fast, native Bongo Cat input overlay for osu! written in Rust. It runs as a lightweight, transparent, always-on-top window.

Unlike overlays built on web stacks (Electron/Tauri), Meowverlay reads your global mouse and keyboard state directly for native performance. On Windows, macOS, and Linux/X11 it polls absolute cursor coordinates (zero drift). On Linux/Wayland it reads the kernel input devices (`/dev/input`, via the `input` group). That is the only way to capture global input under Wayland, where the compositor hides other windows' keyboard state and the pointer from apps.

Inspired by [`kuroni/bongocat-osu`](https://github.com/kuroni/bongocat-osu) and [`HamishDuncanson/pengu-overlay`](https://github.com/HamishDuncanson/pengu-overlay).

---

### Key Features

* **Native Performance:** GPU-accelerated window rendered with `egui` and `eframe`. Extremely low CPU/GPU usage.
* **Accurate Cursor Tracking:** Uses absolute screen coordinates on Windows, macOS, and X11, so the cat's hand never drifts from your cursor. On Wayland, which hides the global pointer from apps, it tracks relative motion, while graphics tablets are tracked absolutely.
* **Supported Modes:** Standard (osu!std), Taiko, Catch the Beat, and Mania (4K & 7K).
* **Skin Compatibility:** Drop-in support for any existing `bongocat-osu` skin folder. The window dynamically resizes to match the skin's background image dimensions.
* **On-the-fly Config:** Customize keybinds (including binding multiple keys to a single action), switch modes, toggle left-handed layouts, or adjust cursor smoothing from the in-app settings panel.
* **Click-through Overlay:** Lock the window to render it click-through, hiding the settings button so it stays completely out of the way of your gameplay.
* **Cross-Platform:** Works on Windows, macOS, and Linux (X11 & Wayland).

---

## Quick Start

You will need a stable Rust toolchain (**version 1.92 or newer**) installed.

```bash
# Clone the repository
git clone https://github.com/rijverse/meowverlay.git
cd meowverlay

# Run in release mode
cargo run --release
```

A transparent window will open containing the Bongo Cat overlay along with a settings panel.

### Troubleshooting Input Capture
If the cat's paws do not move or react to your inputs, run the standalone diagnostic tool:
```bash
cargo run --example input_probe
```
Move your mouse and press keys in the terminal. If key/button events print and the mouse-motion count climbs, global input capture is working on your system.

On **Linux/Wayland**, capture reads `/dev/input`, so your user must be in the `input` group (the usual desktop default). If the probe reports *no readable devices*, add yourself and re-log:
```bash
sudo usermod -aG input "$USER"   # then log out and back in
```

---

## Shortcuts

| Action | Shortcut | Description |
| :--- | :--- | :--- |
| **Lock / Unlock** | `Ctrl + Shift + L` | Toggles click-through mode and hides/shows the settings icon. |
| **Cancel Rebind** | `Esc` | Cancels a key-rebinding capture in progress. |

---

## Operating System Setup

* **Linux:** Runs on both X11 and Wayland. Global input is captured via `/dev/input`, so your user must belong to the `input` group (`sudo usermod -aG input "$USER"`, then re-log). This is the default on most desktops. On an X11 session without that membership it transparently falls back to X11 polling. Note that click-through (lock mode) is best-effort on some Wayland compositors due to window manager limitations.
* **macOS:** You must grant **Accessibility permission** for the application to poll global keyboard inputs. Enable this under *System Settings → Privacy & Security → Accessibility* on your first run.
* **Windows:** Works immediately out of the box.

---

## Adding Custom Skins

Skins are placed inside the `skins/` directory in the root of the repository:

```text
skins/
└── default/
    ├── config.json         # Keys, resolution mapping, and decorations
    └── img/
        ├── osu/            # osu! Standard sprites
        ├── taiko/          # Taiko sprites
        ├── catch/          # Catch the Beat sprites
        └── mania/          # Mania (4K & 7K) sprites
```

Because the format is fully compatible with the C++ `bongocat-osu` layout, you can extract any existing Bongo Cat skin folder into `skins/<name>/` and select it from the settings panel. Keybindings use standard virtual-key codes (e.g. `A` is 65, `Z` is 90).

---

## Codebase Architecture

The project is structured as follows:

```text
Cargo.toml                  Rust crate manifest
src/
  main.rs                   Bootstrap: probes skin size, builds the viewport, and runs the application
  app.rs                    Main state loop: polls input, coordinates rendering, and handles UI state
  config.rs                 Serde models for loading, saving, and parsing config.json files
  skin.rs                   Discovers skins and loads PNGs into egui GPU textures
  input/
    mod.rs                  Global input facade: evdev on Linux, device_query on Windows/macOS (+ X11 fallback)
    evdev_backend.rs        Linux /dev/input reader (works on X11 & Wayland) with KEY/REL/ABS handling
  keycodes.rs               Mappings between device_query keycodes and virtual-key integers
  render/
    mod.rs                  Coordinate mapping, layout mirroring, and render dispatch
    standard.rs             osu! Standard (includes bezier mouse arm and smoke trails)
    taiko.rs / catch.rs / mania.rs
  ui/
    settings.rs             egui configuration menu
examples/
  input_probe.rs            Input hardware diagnostic tool
skins/                      Skin directories
```

To prevent cursor drift (a common issue with relative mouse tracking), Meowverlay polls absolute global coordinates once per frame on Windows, macOS, and X11, matching the behavior of the original C++ implementations. Wayland deliberately withholds the global pointer position from applications, so there the cursor is approximated from relative motion (graphics tablets, which report absolute coordinates, remain exact).

---

## History

This project is a ground-up native Rust rewrite of the original Tauri and TypeScript/Canvas overlay.

---

## License

The code for Meowverlay is licensed under the [Apache License 2.0](LICENSE).

The default skin artwork is derived from [`kuroni/bongocat-osu`](https://github.com/kuroni/bongocat-osu) (MIT License). See the [NOTICE](NOTICE) file for full attribution and license details.
