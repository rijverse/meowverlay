//! The eframe application: owns all state and drives the per-frame update/poll/render loop.

use crate::input::GlobalInput;
use crate::render::{self, AnimState, Canvas, Frame};
use crate::skin::{self, Skin};
use crate::ui;
use egui::{Color32, Sense, Vec2, ViewportCommand};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

/// VK codes for the global lock hotkey: Ctrl + Shift + L.
const LOCK_COMBO: [u32; 3] = [17, 16, 76];

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

    pub(crate) locked: bool,
    pub(crate) settings_open: bool,
    pub(crate) binding: Option<Bind>,
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
        Self {
            skins_dir,
            skin_names,
            current_skin: skin_name.to_string(),
            skin,
            input: GlobalInput::new(),
            anim: AnimState::default(),
            locked: false,
            settings_open: true,
            binding: None,
            binding_baseline: HashSet::new(),
            lock_hotkey_was_down: false,
            resize_pending: false,
            toast: None,
        }
    }

    pub(crate) fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    pub(crate) fn reload_skin(&mut self, ctx: &egui::Context, name: &str) {
        match Skin::load(ctx, &self.skins_dir, name) {
            Ok(s) => {
                self.skin = s;
                self.current_skin = name.to_string();
                self.resize_pending = true;
                self.toast(format!("Loaded skin: {name}"));
            }
            Err(e) => self.toast(format!("Failed to load skin: {e}")),
        }
    }

    pub(crate) fn save_config(&mut self) {
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

    pub(crate) fn start_binding(&mut self, kind: Bind, pressed: &HashSet<u32>) {
        self.binding = Some(kind);
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
        let Some(&code) = pressed.iter().find(|k| !self.binding_baseline.contains(k)) else {
            return;
        };
        self.binding = None;
        if code == 27 {
            self.toast("Binding cancelled");
            return;
        }
        self.apply_binding(kind, code);
        self.toast("✔ Key bound");
    }

    fn apply_binding(&mut self, kind: Bind, code: u32) {
        let cfg = &mut self.skin.config;
        match kind {
            Bind::OsuKey1 => cfg.osu.key1 = vec![code],
            Bind::OsuKey2 => cfg.osu.key2 = vec![code],
            Bind::OsuSmoke => cfg.osu.smoke = vec![code],
            Bind::OsuWave => cfg.osu.wave = vec![code],
            Bind::TaikoLeftRim => cfg.taiko.left_rim = vec![code],
            Bind::TaikoLeftCentre => cfg.taiko.left_centre = vec![code],
            Bind::TaikoRightCentre => cfg.taiko.right_centre = vec![code],
            Bind::TaikoRightRim => cfg.taiko.right_rim = vec![code],
            Bind::CatchLeft => cfg.catch_cfg.left = vec![code],
            Bind::CatchRight => cfg.catch_cfg.right = vec![code],
            Bind::CatchDash => cfg.catch_cfg.dash = vec![code],
            Bind::ManiaColumn(i) => {
                let arr = if cfg.mania.four_k { &mut cfg.mania.key4k } else { &mut cfg.mania.key7k };
                if i < arr.len() {
                    arr[i] = code;
                }
            }
        }
    }

    fn draw_toast(&mut self, ctx: &egui::Context) {
        let Some((msg, started)) = &self.toast else { return };
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
        self.handle_hotkey(&input.pressed, &ctx);
        self.handle_binding(&input.pressed);

        // Resize the OS window to the (possibly new) skin canvas size.
        if self.resize_pending {
            self.resize_pending = false;
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(self.skin.canvas_size()));
        }

        // Normalize the real cursor across the monitor.
        let monitor = ctx
            .input(|i| i.viewport().monitor_size)
            .unwrap_or_else(|| {
                Vec2::new(self.skin.config.resolution.width as f32, self.skin.config.resolution.height as f32)
            });
        let cursor_norm = (
            (input.cursor.0 as f32 / monitor.x.max(1.0)).clamp(0.0, 1.0),
            (input.cursor.1 as f32 / monitor.y.max(1.0)).clamp(0.0, 1.0),
        );

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
        };
        render::draw(&overlay, anim);

        // ── Settings UI ──
        if !self.locked {
            ui::settings::show(&ctx, self, &input.pressed);
        }

        self.draw_toast(&ctx);

        // Keep animating.
        ctx.request_repaint();
    }
}
