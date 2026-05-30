# Changelog

All notable changes to Meowverlay are recorded here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-31

First public release. This is a ground-up native Rust rewrite of the original
Tauri and TypeScript/Canvas overlay, built on egui and eframe.

### Added

- Native, GPU-accelerated overlay window with very low CPU and GPU usage.
- Linux Wayland support through an evdev backend that reads `/dev/input`
  directly, which is the only way to capture global input under Wayland. On an
  X11 session without `input` group access it falls back to X11 polling.
- Absolute cursor tracking for graphics tablets and touchpads via
  `BTN_TOOL_PEN` and `BTN_TOOL_FINGER`, so the paw stays exact on those devices.
- Game modes for osu! Standard, Taiko, Catch the Beat, and Mania (4K and 7K).
- Drop-in compatibility with existing `bongocat-osu` skin folders. The window
  resizes itself to match the skin background dimensions.
- Per-paw start and end coordinates in the skin configuration and renderer.
- An unbound key fallback that drives the animation across every game mode
  using key-code parity when no specific bind matches.
- Cursor smoothing with a configurable easing time.
- Adaptive frame rate that lowers idle resource use and speeds up while you play.
- In-app settings panel for keybinds (including multiple keys per action),
  mode switching, left-handed layouts, and smoothing.
- Click-through lock mode that hides the settings button so the overlay stays
  out of the way during play.
- `input_probe` diagnostic example for confirming global input capture works
  on a given machine.

### Changed

- Replaced the Tauri v1 plus TypeScript and Canvas implementation with native
  Rust. Cursor position is read as absolute screen coordinates on Windows,
  macOS, and X11 to avoid the drift common to relative tracking.
- Rewrote the README with clearer setup and per-OS instructions.

### Fixed

- Canvas sizing and config parsing issues found during a full-project review.

### CI and tooling

- Multi-OS CI running rustfmt, Clippy, build, and tests on Linux, Windows, and
  macOS, plus a minimum supported Rust version check against 1.92.
- A release workflow that builds and publishes binaries for Linux x86_64,
  Windows x86_64, and macOS on both Apple Silicon and Intel, each archive
  bundling the skins and license alongside a checksum.

## [0.1.0]

The original Tauri v1 and TypeScript/Canvas overlay, kept as the pre-rewrite
snapshot. It was never published as a tagged release.

[0.2.0]: https://github.com/rijverse/meowverlay/releases/tag/v0.2.0
