# Plan: fix global input on Linux/Wayland

## Problem
On a Wayland session (Ubuntu GNOME here), the overlay window renders fine but key
presses, mouse buttons, and cursor movement never appear. Root cause: `device_query`
4.0.1 reads Linux input only through Xlib (`XQueryKeymap` / `XQueryPointer`). Under
Wayland those go through XWayland, which deliberately hides other windows' keyboard
state and the global pointer from X11 clients, so `poll()` always comes back empty.

## Fix
Add an `evdev` backend that reads `/dev/input/event*` directly. evdev sits below the
display server, so it works identically on X11 and Wayland. It needs read access to the
device nodes (membership of the `input` group, the usual desktop default). I verified the
nodes are readable here.

### Design
- `src/input/mod.rs`: `GlobalInput` facade plus the `device_query` backend.
  - Linux: try evdev. If no device opens, fall back to `device_query` (X11) and print a
    hint to join the `input` group. This keeps the old zero-permission X11 path working.
  - Windows/macOS: `device_query` as before.
  - New `set_resolution(w, h)` so the evdev cursor can be clamped/scaled to the configured
    play resolution (no-op for device_query).
- `src/input/evdev_backend.rs`: one blocking reader thread per device updating shared
  state behind a `Mutex`:
  - `EV_KEY`: map KEY_*/BTN_* codes -> VK (same numbers as `keycode_to_vk`), tracked as a
    held-set via press/release edges.
  - `EV_REL` (mice): accumulate REL_X/REL_Y, clamp to [0, resolution]. Wayland hides the
    true pointer, so this is an approximation (raw counts, no acceleration).
  - `EV_ABS` (tablets): map ABS_X/ABS_Y within their reported range to [0, resolution],
    which is exact.
- `app.rs` calls `input.set_resolution(config.resolution)` each frame before `poll()`.

## Status
- [x] Diagnose (Wayland + device_query/X11)
- [x] evdev backend (`src/input/evdev_backend.rs`)
- [x] facade refactor + resolution wiring (`src/input/mod.rs`, `app.rs`)
- [x] build + tests + README/Wayland docs
- [x] probe (`examples/input_probe.rs`) updated to exercise evdev on Linux

## Verified
- `cargo clippy --tests` clean. `cargo test` green (incl. new evdev mapping tests).
- App launches on this Wayland session and selects the evdev backend (no fallback warning, no panic).
- `input_probe` opened 9 devices and reported live EV_KEY + 1896 motion events, so capture works end to end.

## Notes / follow-ups
- Touchpads expose both an absolute node and a relative "mouse" node. ABS is only honoured for
  devices advertising `BTN_TOOL_PEN` (real tablets), so the touchpad's relative node drives the paw.
- Wayland mouse position is approximate (accumulated relative counts, no pointer acceleration). If
  the paw moves too slowly/quickly, lower/raise `resolution` in the skin's `config.json`.
- A held key at startup or a device unplugged mid-press could leave a stuck key until next press.
  That is acceptable for a cosmetic overlay. Revisit if it bites.
