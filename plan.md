# Review findings (2026-05-30) - all resolved

Full-project review. All issues below have been fixed. The tree is clippy-clean (`-D warnings`),
rustfmt-clean, and `cargo test` passes. Each item notes how it was addressed.

### Functional bugs
- [x] **1. Per-mode canvas size.** `skin.rs` now has `mode_bg_key()`, while `canvas_size()` and
  `probe_canvas_size()` pick the *current* mode's background (osu! mouse/tablet, mania 4K/7K) with a
  fallback chain. `ui/settings.rs` calls `app.request_resize()` when the mode, mouse/tablet, or
  4K/7K selection changes, so the window resizes on the next frame.
- [x] **2. Malformed config no longer overwrites.** `Skin::load` distinguishes missing vs malformed
  `config.json`: a parse failure loads defaults but records `Skin::config_error`. The error is
  surfaced via toast on load/skin-switch, and `save_config` refuses to write while a config didn't
  parse (so the user's real file is never clobbered). `Skin::load`'s `Result` is now meaningful -
  it returns `Err` only when the skin directory is missing, driving the default-skin fallback.
- [x] **3. Cursor normalization.** `app.rs` normalizes the raw cursor by `config.resolution`
  (the bongocat-osu convention) instead of a single egui-reported monitor, fixing the multi-monitor
  edge-pinning / HiDPI skew and resolving the doc contradiction in #11.

### Polish
- [x] **4. Smoke animation is now frame-rate-independent.** `render/standard.rs` spawns puffs at a
  fixed `SMOKE_SPAWN_PER_SEC` (dt-scaled with a fractional carry in `AnimState::smoke_spawn_accum`)
  and fades by `SMOKE_FADE_PER_SEC * dt`. `Frame` carries `dt`.
- [x] **5. Multi-key rebinding.** Rebinds no longer collapse arrays: the main button replaces, and a
  new ➕ button appends an additional key (de-duplicated) via `start_binding(.., append)` /
  `set_codes()`. Mania columns stay single-key.
- [x] **6. Lock hotkey no longer fires mid-rebind.** `ui()` routes input to the binder *or* the
  hotkey, never both, while capturing, and `handle_binding` ignores bare modifiers (Ctrl/Shift/Alt) so
  reaching a bind via a modifier combo doesn't capture the modifier.
- [x] **7. More keys bindable.** `keycode_to_vk` now covers every `device_query` 4.x `Keycode`
  (NumpadEnter/Decimal/Equals, F13–F20, Meta/Command/Option), and `vk_to_label` has matching labels.
  (PrintScreen/NumLock/Pause/Menu aren't exposed by device_query 4.x, so they can't be captured.)

### Hygiene / enhancements
- [x] **8. License + asset attribution.** Added `LICENSE` (Apache-2.0) and `NOTICE` crediting
  bongocat-osu (MIT) for the default skin assets and config format, and the README has a License section.
- [x] **9. CI.** `.github/workflows/ci.yml` runs fmt + clippy (`-D warnings`) + build + test on a
  Windows/macOS/Linux matrix, plus a separate MSRV (1.92) build job.
- [x] **10. Tests.** `keycodes` round-trip (every mapped Keycode has a real label) and `config`
  load→save preserving the `extra`/`custom` block + partial-config defaults + mania wire names.
- [x] **11. MSRV.** Standardized on **1.92**: `rust-version = "1.92"` in `Cargo.toml`, README, and a
  CI MSRV job. Normalize note reconciled with the code (see #3).
- [x] **12. Stray working-tree change.** Already committed (`cursorSmoothing` is in
  `skins/default/config.json`), so the tree is clean.
