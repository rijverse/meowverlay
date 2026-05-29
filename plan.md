# Meowverlay — Native Rust Rewrite Plan & Progress

Rewriting from Tauri v1 + TypeScript/Canvas → **pure-Rust native (egui/eframe)**.

Inspired by `kuroni/bongocat-osu` and `HamishDuncanson/pengu-overlay`.
Old version snapshotted on `master`; rewrite on branch `native-rewrite`.

## Goals
- **Cross-platform: Windows, macOS, Linux.** Chosen stack (eframe/egui, device_query, image, serde)
  is cross-platform; code stays platform-agnostic.
  - Caveat: macOS needs Accessibility permission for global input.
  - Caveat: click-through is solid on Win/macOS, best-effort on Linux X11/Wayland (winit limit).
- Real global mouse cursor + keyboard tracking, no `input` group needed (X11 via `device_query`).
- Transparent, borderless, always-on-top, click-through overlay.
- Drop-in `bongocat-osu` skin compatibility; window sized to skin background.
- Modular, typed Rust code. egui settings panel with key rebinding + lock.

## Architecture
```
Cargo.toml              root crate "meowverlay"
src/main.rs             eframe bootstrap
src/app.rs              MeowApp: eframe::App, state + update loop
src/config.rs           typed serde structs for bongocat config.json
src/skin.rs             skin discovery + PNG -> egui textures + sizing
src/input.rs            GlobalInput: device_query polling
src/keycodes.rs         device_query Keycode <-> numeric VK code + labels
src/render/{mod,standard,taiko,catch,mania}.rs
src/ui/{mod,settings}.rs
```

## Task checklist
- [x] Snapshot old version (git) + branch `native-rewrite`
- [x] Cargo.toml + dependencies, resolve versions
- [x] keycodes.rs (Keycode<->VK, labels)
- [x] config.rs (typed config load/save)
- [x] input.rs (device_query polling, lock hotkey)
- [x] skin.rs (discover, load PNGs to textures, sizes)
- [x] render/* (standard, taiko, catch, mania + mirror/helpers)
- [x] ui/settings.rs (panel, rebinding, save, lock, drag)
- [x] app.rs + main.rs (wire together, transparent viewport)
- [x] Remove old web/Tauri stack
- [x] cargo build clean (zero warnings)
- [x] cargo run — overlay launches, transparent, stable; global input verified
- [x] Rewrite README.md
- [x] examples/input_probe.rs diagnostic (verifies no-permission input path)

## Verification results (2026-05-29)
- Toolchain: `rustup update stable` → rustc 1.96.0 (was 1.87.0, too old for egui 0.34).
- `cargo build`: clean, zero warnings.
- `cargo run`: overlay process launches and stays healthy (multithreaded event loop,
  ~75MB RSS, no panics/errors), transparent borderless window sized to skin.
- Global input (`cargo run --example input_probe`): reads the **real absolute cursor**
  (e.g. (966,582)) + button/key arrays on this Wayland/XWayland session with **no `input`
  group and no elevated permissions** — confirms the original drift/permission bug is fixed.
- Screenshot-based visual capture unavailable: compositor blocks grim (not wlroots) and
  GNOME D-Bus screenshot ("not allowed"). App health verified via process + input probe instead.

## API adjustments for eframe/egui 0.34
- `eframe::App` required method is now `fn ui(&mut self, ui: &mut egui::Ui, frame)` (was
  `update(ctx, frame)`, now deprecated). App paints directly on the provided central `Ui`;
  `ctx` obtained via `ui.ctx().clone()` for viewport commands + deferred settings/toast windows.
- Dropped `CentralPanel`/`Frame::none()` wrapper (the provided `Ui` already has no background).

## Notes / decisions
- Keycode convention kept = numeric JS/VK codes from existing config.json (A=65, etc.).
- Cursor mapping reuses old math: normalize real cursor by config.resolution, map into
  mousePaw.pawEndingPoint + decoration offsets/scalar.
- Verified APIs: ViewportBuilder::{with_transparent,with_decorations,with_always_on_top,
  with_mouse_passthrough,with_inner_size,with_resizable}; ViewportCommand::MousePassthrough/InnerSize/StartDrag.
- device_query: DeviceState::{get_mouse().coords, get_keys()->Vec<Keycode>} — X11, no perms.
