// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod input;
mod keycodes;
mod render;
mod skin;
mod ui;

use app::MeowApp;

fn main() -> eframe::Result<()> {
    let skins_dir = skin::find_skins_dir();
    let skin_name = "default".to_string();
    let size = skin::probe_canvas_size(&skins_dir, &skin_name);

    let viewport = egui::ViewportBuilder::default()
        .with_title("Meowverlay")
        .with_inner_size(size)
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_resizable(false)
        .with_mouse_passthrough(false);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Meowverlay",
        options,
        Box::new(move |cc| Ok(Box::new(MeowApp::new(&cc.egui_ctx, skins_dir, &skin_name)))),
    )
}
