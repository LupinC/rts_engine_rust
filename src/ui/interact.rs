use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::backend::{PaletteTab, PaletteState, palette_entries};

pub fn ui_interact(
    mut ctx: EguiContexts,
    mut palette: ResMut<PaletteState>,
) {
    let ctx = ctx.ctx_mut();

    egui::TopBottomPanel::bottom("bottom/interact")
        .resizable(true)
        .default_height(140.0)
        .min_height(90.0)
        .show_separator_line(true)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(16, 16, 16))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        )
        .show(ctx, |ui| {
            // --- Tabs row ---
            ui.horizontal(|ui| {
                tab_button(ui, &mut palette.tab, PaletteTab::Resource, "resource");
                tab_button(ui, &mut palette.tab, PaletteTab::SpawnPoints, "spawn p");
                ui.add_space(8.0);

                // (Optional) show current selection
                if let Some(idx) = palette.selected_idx {
                    let entries = palette_entries(palette.tab);
                    if let Some(e) = entries.get(idx) {
                        ui.label(
                            egui::RichText::new(format!("Selected: {}", e.name))
                                .color(egui::Color32::GRAY)
                        );
                    }
                }
            });

            ui.add_space(6.0);

            // --- Bordered inner palette area ---
            egui::Frame::none()
                .stroke(egui::Stroke::new(2.0, egui::Color32::from_gray(60)))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    let entries = palette_entries(palette.tab);

                    ui.horizontal_wrapped(|ui| {
                        for (i, item) in entries.iter().enumerate() {
                            let sel = palette.selected_idx == Some(i);
                            let (bg, txt, border) = if sel {
                                (item.color, egui::Color32::BLACK, egui::Color32::WHITE)
                            } else {
                                (egui::Color32::from_gray(34), egui::Color32::from_gray(230), egui::Color32::from_gray(80))
                            };

                            let button = egui::Button::new(
                                egui::RichText::new(format!("{}\n{}", item.emoji, item.name))
                                    .size(16.0)
                                    .color(txt)
                                    .strong()
                            )
                            .min_size(egui::vec2(80.0, 80.0))
                            .rounding(8.0)
                            .fill(bg)
                            .stroke(egui::Stroke::new(2.0, border));

                            let resp = ui.add(button);
                            if resp.clicked() {
                                palette.selected_idx = Some(i);
                            }

                            ui.add_space(10.0);
                        }
                    });
                });
        });
}

fn tab_button(ui: &mut egui::Ui, current: &mut PaletteTab, me: PaletteTab, label: &str) {
    let on = *current == me;
    let (bg, fg, border) = if on {
        (egui::Color32::from_gray(230), egui::Color32::BLACK, egui::Color32::WHITE)
    } else {
        (egui::Color32::from_gray(40), egui::Color32::from_gray(220), egui::Color32::from_gray(70))
    };

    let btn = egui::Button::new(egui::RichText::new(label).strong())
        .min_size(egui::vec2(120.0, 28.0))
        .rounding(6.0)
        .fill(bg)
        .stroke(egui::Stroke::new(1.5, border));

    if ui.add(btn).clicked() {
        *current = me;
    }
}
