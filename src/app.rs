//! The eframe application: owns all state and drives the per-frame update/poll/render loop.

use crate::input::GlobalInput;
use crate::render::{self, AnimState, Canvas, Frame};
use crate::skin::{self, Skin};
use crate::ui;
use egui::{Color32, Sense, ViewportCommand};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// VK codes for the global lock hotkey: Ctrl + Shift + L.
const LOCK_COMBO: [u32; 3] = [17, 16, 76];

/// VK codes for the bare modifier keys (Ctrl, Shift, Alt), ignored while capturing a rebind.
fn is_modifier(code: u32) -> bool {
    matches!(code, 16..=18)
}

/// Apply a captured key to an action's key list: append (de-duplicated) or replace.
fn set_codes(target: &mut Vec<u32>, code: u32, append: bool) {
    if append {
        if !target.contains(&code) {
            target.push(code);
        }
    } else {
        *target = vec![code];
    }
}

/// Repaint cap while something is animating (keys held, cursor easing, smoke alive). The overlay is
/// cosmetic, so 60 is plenty smooth.
const ACTIVE_FPS: f32 = 60.0;
/// Repaint cap when fully idle. Input is still polled at this rate, so the worst-case lag before the
/// paw reacts after a pause is ~1/IDLE_FPS — imperceptible for a cosmetic overlay, and it roughly
/// halves idle GPU/CPU versus rendering flat-out.
const IDLE_FPS: f32 = 30.0;

/// Which config binding is currently capturing a key press.
#[derive(Clone, Copy, PartialEq)]
pub enum Bind {
    OsuKey1,
    OsuKey2,
    OsuSmoke,
    OsuWave,
    TaikoLeftRim,
    TaikoLeftCentre,
    TaikoRightCentre,
    TaikoRightRim,
    CatchLeft,
    CatchRight,
    CatchDash,
    ManiaColumn(usize),
}

pub struct MeowApp {
    pub(crate) skins_dir: PathBuf,
    pub(crate) skin_names: Vec<String>,
    pub(crate) current_skin: String,
    pub(crate) skin: Skin,
    input: GlobalInput,
    anim: AnimState,
    /// Rendered cursor position (normalized [0,1]), eased toward the raw poll each frame.
    /// `None` until the first poll, so we snap to the initial position instead of swooping from 0.
    cursor_smooth: Option<(f32, f32)>,

    pub(crate) locked: bool,
    pub(crate) settings_open: bool,
    pub(crate) binding: Option<Bind>,
    /// When the active binding should *add* a key (multi-key) rather than replace the existing list.
    binding_append: bool,
    binding_baseline: HashSet<u32>,
    lock_hotkey_was_down: bool,
    resize_pending: bool,

    toast: Option<(String, Instant)>,
}

impl MeowApp {
    pub fn new(ctx: &egui::Context, skins_dir: PathBuf, skin_name: &str) -> Self {
        let skin_names = skin::discover_skins(&skins_dir);
        let skin = Skin::load(ctx, &skins_dir, skin_name).unwrap_or_else(|e| {
            eprintln!("[meowverlay] failed to load skin '{skin_name}': {e}");
            Skin::load(ctx, &skins_dir, "default").expect("default skin must load")
        });
        let config_error = skin.config_error.clone();
        let mut app = Self {
            skins_dir,
            skin_names,
            current_skin: skin_name.to_string(),
            skin,
            input: GlobalInput::new(),
            anim: AnimState::default(),
            cursor_smooth: None,
            locked: false,
            settings_open: true,
            binding: None,
            binding_append: false,
            binding_baseline: HashSet::new(),
            lock_hotkey_was_down: false,
            resize_pending: false,
            toast: None,
        };
        if let Some(err) = config_error {
            app.toast(format!(
                "⚠ Using defaults — couldn't parse config.json: {err}"
            ));
        }
        app
    }

    /// Request the window to resize to the current skin/mode canvas size on the next frame.
    pub(crate) fn request_resize(&mut self) {
        self.resize_pending = true;
    }

    pub(crate) fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    pub(crate) fn reload_skin(&mut self, ctx: &egui::Context, name: &str) {
        match Skin::load(ctx, &self.skins_dir, name) {
            Ok(s) => {
                let err = s.config_error.clone();
                self.skin = s;
                self.current_skin = name.to_string();
                self.resize_pending = true;
                match err {
                    Some(e) => self.toast(format!(
                        "⚠ {name}: using defaults — couldn't parse config.json: {e}"
                    )),
                    None => self.toast(format!("Loaded skin: {name}")),
                }
            }
            Err(e) => self.toast(format!("Failed to load skin: {e}")),
        }
    }

    pub(crate) fn save_config(&mut self) {
        // Never clobber a config we couldn't parse — the in-memory copy is all-defaults and would
        // destroy the user's real settings. Make them fix or remove the file first.
        if let Some(err) = self.skin.config_error.clone() {
            self.toast(format!(
                "❌ Not saving: config.json didn't parse ({err}). Fix or remove it first."
            ));
            return;
        }
        let path = self.skins_dir.join(&self.current_skin).join("config.json");
        match self.skin.config.save(&path) {
            Ok(()) => self.toast("✔ Configuration saved"),
            Err(e) => self.toast(format!("❌ Save failed: {e}")),
        }
    }

    pub(crate) fn set_lock(&mut self, ctx: &egui::Context, locked: bool) {
        self.locked = locked;
        self.settings_open = !locked;
        ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(locked));
        if locked {
            self.toast("🔒 Locked (click-through). Ctrl+Shift+L to unlock.");
        } else {
            self.toast("🔓 Unlocked.");
        }
    }

    /// Begin capturing a key for `kind`. With `append`, the captured key is *added* to the action's
    /// key list (multi-key support); otherwise it replaces the list.
    pub(crate) fn start_binding(&mut self, kind: Bind, pressed: &HashSet<u32>, append: bool) {
        self.binding = Some(kind);
        self.binding_append = append;
        self.binding_baseline = pressed.clone();
    }

    /// Toggle lock when Ctrl+Shift+L transitions to held.
    fn handle_hotkey(&mut self, pressed: &HashSet<u32>, ctx: &egui::Context) {
        let down = LOCK_COMBO.iter().all(|k| pressed.contains(k));
        if down && !self.lock_hotkey_was_down {
            self.set_lock(ctx, !self.locked);
        }
        self.lock_hotkey_was_down = down;
    }

    /// If a key bind is active, capture the first newly-pressed key.
    fn handle_binding(&mut self, pressed: &HashSet<u32>) {
        let Some(kind) = self.binding else { return };
        // Drop released keys from the baseline so the same key can be re-pressed to bind.
        self.binding_baseline.retain(|k| pressed.contains(k));
        // Ignore bare modifiers while capturing: it lets the user reach a bind via Ctrl/Shift+key
        // (e.g. the Ctrl+Shift+L lock combo) without the modifier itself being captured.
        let candidate = pressed
            .iter()
            .find(|k| !self.binding_baseline.contains(k) && !is_modifier(**k));
        let Some(&code) = candidate else { return };
        self.binding = None;
        if code == 27 {
            self.toast("Binding cancelled");
            return;
        }
        self.apply_binding(kind, code);
        self.toast(if self.binding_append {
            "✔ Key added"
        } else {
            "✔ Key bound"
        });
    }

    fn apply_binding(&mut self, kind: Bind, code: u32) {
        let append = self.binding_append;
        let cfg = &mut self.skin.config;
        match kind {
            Bind::OsuKey1 => set_codes(&mut cfg.osu.key1, code, append),
            Bind::OsuKey2 => set_codes(&mut cfg.osu.key2, code, append),
            Bind::OsuSmoke => set_codes(&mut cfg.osu.smoke, code, append),
            Bind::OsuWave => set_codes(&mut cfg.osu.wave, code, append),
            Bind::TaikoLeftRim => set_codes(&mut cfg.taiko.left_rim, code, append),
            Bind::TaikoLeftCentre => set_codes(&mut cfg.taiko.left_centre, code, append),
            Bind::TaikoRightCentre => set_codes(&mut cfg.taiko.right_centre, code, append),
            Bind::TaikoRightRim => set_codes(&mut cfg.taiko.right_rim, code, append),
            Bind::CatchLeft => set_codes(&mut cfg.catch_cfg.left, code, append),
            Bind::CatchRight => set_codes(&mut cfg.catch_cfg.right, code, append),
            Bind::CatchDash => set_codes(&mut cfg.catch_cfg.dash, code, append),
            // Mania columns are a single key each (flat array indexed by column), so always replace.
            Bind::ManiaColumn(i) => {
                let arr = if cfg.mania.four_k {
                    &mut cfg.mania.key4k
                } else {
                    &mut cfg.mania.key7k
                };
                if i < arr.len() {
                    arr[i] = code;
                }
            }
        }
    }

    fn draw_toast(&mut self, ctx: &egui::Context) {
        let Some((msg, started)) = &self.toast else {
            return;
        };
        if started.elapsed().as_secs_f32() > 3.5 {
            self.toast = None;
            return;
        }
        let msg = msg.clone();
        egui::Area::new("toast".into())
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -8.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(Color32::from_black_alpha(200))
                    .show(ui, |ui| {
                        ui.colored_label(Color32::WHITE, msg);
                    });
            });
    }
}

impl eframe::App for MeowApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // fully transparent
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The provided `ui` already covers the full viewport with no background, so we paint the
        // transparent overlay directly onto it. `ctx` is a cheap Arc clone used for viewport
        // commands and the deferred settings/toast windows.
        let ctx = ui.ctx().clone();

        let input = self.input.poll();
        // While capturing a rebind, route input to the binder only — otherwise the Ctrl+Shift+L
        // used to reach a bind would also toggle the lock mid-capture.
        if self.binding.is_some() {
            self.handle_binding(&input.pressed);
        } else {
            self.handle_hotkey(&input.pressed, &ctx);
        }

        // Resize the OS window to the (possibly new) skin canvas size.
        if self.resize_pending {
            self.resize_pending = false;
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(self.skin.canvas_size()));
        }

        // Normalize the real cursor against the configured screen resolution (the bongocat-osu
        // convention). device_query reports virtual-desktop pixels spanning *all* monitors, so
        // dividing by a single egui-reported monitor pinned the paw to an edge on secondary screens
        // and skewed under HiDPI. Using config.resolution keeps the mapping deterministic and
        // user-tunable; set it to your play resolution if the paw doesn't track edge-to-edge.
        let res = &self.skin.config.resolution;
        let target = (
            (input.cursor.0 as f32 / (res.width as f32).max(1.0)).clamp(0.0, 1.0),
            (input.cursor.1 as f32 / (res.height as f32).max(1.0)).clamp(0.0, 1.0),
        );

        // Ease the rendered cursor toward the raw target. `alpha = 1 - e^(-dt/tau)` makes the
        // smoothing frame-rate independent; `stable_dt` is egui's spike-clamped frame time. The tau
        // (seconds) comes from the skin config slider; tau <= 0 means "off" (snap to the raw poll).
        let tau = self.skin.config.cursor_smoothing;
        let dt = ctx.input(|i| i.stable_dt);
        let cursor_norm = match self.cursor_smooth {
            Some((cx, cy)) if tau > 0.0 => {
                let alpha = (1.0 - (-dt / tau).exp()).clamp(0.0, 1.0);
                (cx + (target.0 - cx) * alpha, cy + (target.1 - cy) * alpha)
            }
            // First frame, or smoothing off: snap so we don't swoop from (0,0).
            _ => target,
        };
        self.cursor_smooth = Some(cursor_norm);

        // ── Render the overlay ──
        let rect = ui.max_rect();

        // Dragging the background moves the OS window (only when unlocked).
        if !self.locked {
            let resp = ui.interact(rect, ui.id().with("window-drag"), Sense::click_and_drag());
            if resp.drag_started() {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }
        }

        // Disjoint borrows: skin (shared) + anim (mutable).
        let skin = &self.skin;
        let anim = &mut self.anim;
        let painter = ui.painter().clone();
        let canvas = Canvas::new(rect, skin.canvas_size(), skin.config.decoration.left_handed);
        let overlay = Frame {
            painter: &painter,
            canvas,
            config: &skin.config,
            skin,
            pressed: &input.pressed,
            cursor_norm,
            dt,
        };
        render::draw(&overlay, anim);

        // ── Settings UI ──
        if !self.locked {
            ui::settings::show(&ctx, self, &input.pressed);
        }

        self.draw_toast(&ctx);

        // Adaptive repaint cap. Stay at 60fps while anything moves; otherwise idle at a lower rate
        // that still polls input promptly. (We drive our own clock since global input is polled, not
        // event-driven, so egui won't wake us on key/mouse activity by itself.)
        let still_easing =
            (cursor_norm.0 - target.0).abs() > 1e-4 || (cursor_norm.1 - target.1).abs() > 1e-4;
        let animating = !input.pressed.is_empty() || !self.anim.smoke.is_empty() || still_easing;
        let fps = if animating { ACTIVE_FPS } else { IDLE_FPS };
        ctx.request_repaint_after(Duration::from_secs_f32(1.0 / fps));
    }
}
