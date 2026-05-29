# Meowverlay 🐱

A fast, native **Bongo Cat / input overlay** for osu!, inspired by
[`kuroni/bongocat-osu`](https://github.com/kuroni/bongocat-osu) and
[`HamishDuncanson/pengu-overlay`](https://github.com/HamishDuncanson/pengu-overlay).

Meowverlay is a single self-contained binary written in **Rust** with
[`egui`/`eframe`](https://github.com/emilk/egui). It renders a transparent, borderless,
always-on-top overlay that reacts to your **real** global keyboard and mouse — and it tracks the
*actual* cursor position (no drift) using a polling approach that needs **no special permissions and
no `input` group**.

- 🪶 **Native & lightweight** — one binary, GPU-accelerated, no web stack / no Electron.
- 🖱️ **Accurate global input** — true absolute cursor + global key state via
  [`device_query`](https://crates.io/crates/device_query); works while another window is focused.
- 🎮 **All osu! modes** — Standard, Taiko, Catch the Beat, and Mania (4K & 7K).
- 🎨 **Drop-in skin compatibility** — load any `bongocat-osu` skin; the window auto-sizes to it.
- ⚙️ **In-app settings** — switch skin/mode, rebind keys, toggle left-handed / mouse-vs-tablet /
  smoke, all from an egui panel. Lock the overlay to make it click-through.
- 💻 **Cross-platform** — Windows, macOS, and Linux (X11 / XWayland).

---

## 🚀 Build & Run

You only need a [Rust toolchain](https://rustup.rs) (stable, **1.92+** — `egui 0.34` requires it;
run `rustup update stable` if your `rustc` is older).

```bash
cargo run            # debug
cargo run --release  # optimized
```

A transparent overlay window appears, sized to the default skin, with the settings panel open.

> **First run looks "dead"?** Run the input diagnostic to confirm global capture works on your
> machine — move the mouse and press keys while it runs:
> ```bash
> cargo run --example input_probe
> ```
> If the cursor coordinates change and pressed keys are listed, you're good.

### Platform notes
- **Linux:** Works on X11 and XWayland out of the box — **no `input` group, no `sudo`, no `evdev`
  setup**. Click-through (lock mode) is solid on most setups but is best-effort on some
  Wayland compositors (a `winit` limitation).
- **macOS:** The OS requires **Accessibility permission** for global input capture. Grant it under
  *System Settings → Privacy & Security → Accessibility* the first time you run the app.
- **Windows:** Works out of the box.

---

## 🎛️ Using the overlay

- **Move it:** drag the cat (when unlocked).
- **Open settings:** the ⚙ button (top-left) / the settings window.
- **Rebind a key:** click a key button in settings, then press the key you want — `Esc` cancels.
- **Save:** writes back to that skin's `config.json` (stays `bongocat-osu`-compatible).
- **Lock / click-through:** press **`Ctrl + Shift + L`** (or the 🔒 Lock button). Locking hides the
  settings and lets clicks pass through to the apps behind the overlay. Press the hotkey again to
  unlock.

---

## 🎨 Adding skins

Skins live in the `skins/` folder in the project root:

```text
skins/
└── default/
    ├── config.json         # mode, keybindings, mouse mapping, decorations
    └── img/
        ├── osu/            # Standard mode sprites
        ├── taiko/          # Taiko mode sprites
        ├── catch/          # Catch the Beat sprites
        └── mania/          # Mania (4K / 7K) sprites
```

The layout and `config.json` format are **drop-in compatible with `bongocat-osu` skins** — extract
any existing Bongo Cat skin into `skins/<name>/` and pick it from the skin selector. The overlay
window resizes itself to the skin's background dimensions automatically. Keybindings use the same
numeric key codes as `bongocat-osu` (e.g. `A`=65, `Z`=90).

---

## 🏗️ Architecture

```text
Cargo.toml                  binary crate "meowverlay"
src/
  main.rs                   eframe bootstrap: probe skin size, build transparent viewport, run
  app.rs                    MeowApp (eframe::App): owns state, per-frame poll → render loop
  config.rs                 typed serde structs for bongocat config.json (load / save)
  skin.rs                   skin discovery, PNG → egui textures, canvas sizing
  input.rs                  GlobalInput: device_query polling → pressed VK set + cursor
  keycodes.rs               device_query Keycode ↔ numeric VK code + human-readable labels
  render/
    mod.rs                  mode dispatch + shared canvas/mirror/sprite helpers
    standard.rs             osu! standard: paw frames, arm bezier, smoke particles
    taiko.rs / catch.rs / mania.rs
  ui/
    settings.rs             egui settings panel: skin/mode, toggles, rebind, save, lock
examples/
  input_probe.rs            diagnostic: prints live global cursor + keys
skins/                      drop-in bongocat-osu skins
```

Input is polled once per frame (egui repaints continuously), which is simple and robust and gives
the true cursor position the C++ references use — replacing the old relative-delta accumulation that
drifted from screen center.

---

## 📜 History

This is a ground-up **native Rust rewrite**. The previous Tauri v1 + TypeScript/Canvas version is
preserved in git history (commit *"Snapshot: Tauri v1 + TypeScript/Canvas version before native Rust
rewrite"*).
