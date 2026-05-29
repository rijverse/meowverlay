//! The egui settings panel: skin/mode selection, toggles, key rebinding, save, and lock.

use crate::app::{Bind, MeowApp};
use crate::keycodes::vk_to_label;
use std::collections::HashSet;

/// Render the settings UI (gear button when collapsed, full panel when open).
pub fn show(ctx: &egui::Context, app: &mut MeowApp, pressed: &HashSet<u32>) {
    if !app.settings_open {
        egui::Area::new("gear".into())
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(6.0, 6.0))
            .show(ctx, |ui| {
                if ui.button("⚙").on_hover_text("Open settings").clicked() {
                    app.settings_open = true;
                }
            });
        return;
    }

    let mut open = true;
    egui::Window::new("🐱 Meowverlay")
        .resizable(false)
        .collapsible(true)
        .open(&mut open)
        .default_pos(egui::pos2(8.0, 8.0))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                general_section(ctx, app, ui);
                ui.separator();
                bindings_section(app, ui, pressed);
                ui.separator();
                footer(ctx, app, ui);
            });
        });
    if !open {
        app.settings_open = false;
    }
}

fn general_section(ctx: &egui::Context, app: &mut MeowApp, ui: &mut egui::Ui) {
    // Skin selector.
    let names = app.skin_names.clone();
    let mut chosen = app.current_skin.clone();
    egui::ComboBox::from_label("Skin")
        .selected_text(chosen.clone())
        .show_ui(ui, |ui| {
            for name in &names {
                ui.selectable_value(&mut chosen, name.clone(), name);
            }
        });
    if chosen != app.current_skin {
        app.reload_skin(ctx, &chosen);
    }

    // Game mode.
    let cfg = &mut app.skin.config;
    egui::ComboBox::from_label("Mode")
        .selected_text(mode_label(cfg.mode))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut cfg.mode, 1, "Standard (osu!)");
            ui.selectable_value(&mut cfg.mode, 2, "Taiko");
            ui.selectable_value(&mut cfg.mode, 3, "Catch the Beat");
            ui.selectable_value(&mut cfg.mode, 4, "Mania");
        });

    ui.checkbox(&mut cfg.decoration.left_handed, "Left-handed layout");
    ui.checkbox(&mut cfg.osu.mouse, "Use mouse (vs. tablet)");
    ui.checkbox(&mut cfg.osu.toggle_smoke, "Toggle smoke (vs. hold)");
}

fn bindings_section(app: &mut MeowApp, ui: &mut egui::Ui, pressed: &HashSet<u32>) {
    let mode = app.skin.config.mode;
    match mode {
        2 => {
            ui.heading("Taiko");
            bind_row(app, ui, pressed, "Left Rim (Don)", Bind::TaikoLeftRim);
            bind_row(app, ui, pressed, "Left Centre (Ka)", Bind::TaikoLeftCentre);
            bind_row(app, ui, pressed, "Right Centre (Ka)", Bind::TaikoRightCentre);
            bind_row(app, ui, pressed, "Right Rim (Don)", Bind::TaikoRightRim);
        }
        3 => {
            ui.heading("Catch");
            bind_row(app, ui, pressed, "Move Left", Bind::CatchLeft);
            bind_row(app, ui, pressed, "Move Right", Bind::CatchRight);
            bind_row(app, ui, pressed, "Dash", Bind::CatchDash);
        }
        4 => {
            ui.heading("Mania");
            let four_k = app.skin.config.mania.four_k;
            let mut is_4k = four_k;
            ui.horizontal(|ui| {
                ui.selectable_value(&mut is_4k, true, "4K");
                ui.selectable_value(&mut is_4k, false, "7K");
            });
            if is_4k != four_k {
                app.skin.config.mania.four_k = is_4k;
            }
            let count = if app.skin.config.mania.four_k { 4 } else { 7 };
            for i in 0..count {
                bind_row(app, ui, pressed, &format!("Column {}", i + 1), Bind::ManiaColumn(i));
            }
        }
        _ => {
            ui.heading("osu! Standard");
            bind_row(app, ui, pressed, "Key 1 (Left)", Bind::OsuKey1);
            bind_row(app, ui, pressed, "Key 2 (Right)", Bind::OsuKey2);
            bind_row(app, ui, pressed, "Smoke", Bind::OsuSmoke);
            bind_row(app, ui, pressed, "Wave", Bind::OsuWave);
        }
    }
}

fn bind_row(app: &mut MeowApp, ui: &mut egui::Ui, pressed: &HashSet<u32>, label: &str, kind: Bind) {
    let is_active = app.binding == Some(kind);
    let text = if is_active { "Press a key…".to_string() } else { current_label(app, kind) };
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.button(text).clicked() {
            app.start_binding(kind, pressed);
        }
    });
}

fn current_label(app: &MeowApp, kind: Bind) -> String {
    let cfg = &app.skin.config;
    let codes: &[u32] = match kind {
        Bind::OsuKey1 => &cfg.osu.key1,
        Bind::OsuKey2 => &cfg.osu.key2,
        Bind::OsuSmoke => &cfg.osu.smoke,
        Bind::OsuWave => &cfg.osu.wave,
        Bind::TaikoLeftRim => &cfg.taiko.left_rim,
        Bind::TaikoLeftCentre => &cfg.taiko.left_centre,
        Bind::TaikoRightCentre => &cfg.taiko.right_centre,
        Bind::TaikoRightRim => &cfg.taiko.right_rim,
        Bind::CatchLeft => &cfg.catch_cfg.left,
        Bind::CatchRight => &cfg.catch_cfg.right,
        Bind::CatchDash => &cfg.catch_cfg.dash,
        Bind::ManiaColumn(i) => {
            let arr = if cfg.mania.four_k { &cfg.mania.key4k } else { &cfg.mania.key7k };
            return arr.get(i).map(|c| vk_to_label(*c)).unwrap_or_else(|| "None".into());
        }
    };
    codes_label(codes)
}

fn codes_label(codes: &[u32]) -> String {
    if codes.is_empty() {
        "None".to_string()
    } else {
        codes.iter().map(|c| vk_to_label(*c)).collect::<Vec<_>>().join(" / ")
    }
}

fn footer(ctx: &egui::Context, app: &mut MeowApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("💾 Save").clicked() {
            app.save_config();
        }
        if ui.button("🔒 Lock").clicked() {
            app.set_lock(ctx, true);
        }
    });
    ui.label("Drag the cat to move • Ctrl+Shift+L locks/unlocks");
}

fn mode_label(mode: u32) -> &'static str {
    match mode {
        2 => "Taiko",
        3 => "Catch the Beat",
        4 => "Mania",
        _ => "Standard (osu!)",
    }
}
