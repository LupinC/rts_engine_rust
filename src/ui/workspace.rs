use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn ui_workspace(mut ctx: EguiContexts) {
    let ctx = ctx.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        // Big empty workspace with a subtle tint
        let rect = ui.max_rect();
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(24, 27, 33));

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(
                    egui::RichText::new("Workspace")
                        .size(18.0)
                        .color(egui::Color32::from_rgb(180, 190, 200)),
                );
                ui.label(
                    egui::RichText::new("…Explorer, tabs, and editor will go here.")
                        .size(13.0)
                        .color(egui::Color32::from_gray(160)),
                );
            });
        });
    });
}
