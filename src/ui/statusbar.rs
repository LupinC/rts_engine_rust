use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn ui_statusbar(mut ctx: EguiContexts) {
    let ctx = ctx.ctx_mut();

    egui::TopBottomPanel::bottom("statusbar")
        .exact_height(22.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.small("Ready");
                ui.separator();
                ui.small("UTF-8");
                ui.separator();
                ui.small("Rust • Bevy • egui");
            });
        });
}
