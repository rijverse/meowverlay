# Review findings (2026-05-30)
Full-project review. Code is clean (clippy zero-warnings, no `unsafe`, no fallible `unwrap`,
defensive `.get().unwrap_or()` throughout). Issues below ordered by impact.

### Functional bugs
- [ ] **1. Per-mode canvas size is wrong — breaks "drop-in skin" feature.** `skin.rs:71`
  (`canvas_size`) and `skin.rs:116` (`probe_canvas_size`) always return the *standard* (`mousebg`)
  background size; they never consult `config.mode`. Switching mode in the combo (`ui/settings.rs:57`)
  mutates `cfg.mode` but never sets `resize_pending`, so the window never resizes on mode change.
  Latent in the default skin (all mode backgrounds are coincidentally 612×354), but real bongocat-osu
  skins ship differently-sized Taiko/Catch/Mania backgrounds → stretched render in a wrong-sized
  window. Fix: make `canvas_size()` pick the current mode's background (+ `mania.four_k`); set
  `resize_pending = true` when the mode combo changes.
- [ ] **2. Malformed config silently overwrites the user's file with defaults.** `skin.rs:86`
  `Config::load(...).unwrap_or_else(|_| Config::default())` swallows parse errors → app loads defaults
  silently, then Save (`app.rs:102`) writes them back, destroying the real config. The `Result` on
  `Skin::load` is also misleading (it can never return `Err`), making the `.expect("default skin must
  load")` at `app.rs:66` dead code. Fix: surface parse failures via toast and refuse to save / back up
  a config that didn't parse.
- [ ] **3. Cursor normalization wrong on multi-monitor / HiDPI.** `app.rs:217` device_query returns
  virtual-desktop coords spanning all monitors, but we divide by a single monitor's `monitor_size` and
  `clamp(0,1)` → paw pins to an edge on the 2nd monitor; HiDPI logical-vs-physical can skew it. Also
  contradicts the "normalize by config.resolution" note above. Cosmetic (±44px paw wiggle) but it's the
  "tracks your real cursor" promise. Fix: normalize by the full virtual-desktop pixel extent (or
  `config.resolution`).

### Polish
- [ ] **4. Smoke animation is frame-rate-dependent.** `render/standard.rs:53` — one puff/frame and
  `alpha -= 0.015`/frame, so density/lifetime scale with FPS (inconsistent with the dt-based cursor
  easing; masked today only because live smoke forces ACTIVE_FPS). Scale both by `dt`.
- [ ] **5. Rebinding collapses multi-key arrays to one.** `app.rs:155` `cfg.osu.key1 = vec![code]`
  discards bongocat's multi-key-per-action support; no UI way to keep/add a second binding.
- [ ] **6. Lock hotkey fires mid-rebind.** `app.rs:127`/`app.rs:136` — Ctrl+Shift+L during a "Press a
  key…" capture both toggles lock and captures a modifier. Fix: skip `handle_hotkey` while
  `binding.is_some()` and/or ignore bare modifiers when capturing.
- [ ] **7. Some keys can't be bound.** `keycodes.rs:55` — `keycode_to_vk` returns 0 for PrintScreen,
  NumLock, NumpadEnter, NumpadDecimal, Pause, Menu, etc.

### Hygiene / enhancements
- [ ] **8. No `LICENSE` file** (README states none). `skins/default/img/**` appears derived from
  bongocat-osu → note asset provenance/attribution.
- [ ] **9. No CI** — a GitHub Actions Win/macOS/Linux build matrix would catch platform regressions
  not visible from Linux.
- [ ] **10. No tests** — cheap high-value ones: `keycode_to_vk`↔`vk_to_label` round-trip; `Config`
  load→save preserving the `extra`/`custom` block.
- [ ] **11. Doc drift** — `README.md:26` says Rust 1.92+; `plan.md` verification implies 1.96 (1.87 too
  old). Pick one MSRV. Plan's "normalize by config.resolution" also disagrees with the code (see #3).
- [ ] **12. Stray working-tree change** — `skins/default/config.json` has an uncommitted
  `+"cursorSmoothing": 0.045`; commit or revert.
