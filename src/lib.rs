use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod app;
mod ui;
mod frontend;
mod backend;

/// Public entry point used by `main.rs`
pub fn run() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.11)))
        .add_plugins((
            // Core Bevy plugins & window setup done in `app::setup`
            app::setup::CoreWindowPlugin,
            EguiPlugin,
            ui::EditorUiPlugin,     // Menubar + workspace + statusbar
            frontend::FrontendPlugin, // Placeholder for “game view”, panels, etc.
            backend::BackendPlugin,   // Placeholder for data/model services
        ))
        .run();
}
