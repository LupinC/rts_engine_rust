use bevy_egui::{egui, EguiContexts};
use crate::backend::{MapPreview, theater_color}; // <-- import from backend re-exports

pub fn ui_workspace(mut ctx: EguiContexts, preview: bevy::prelude::Res<MapPreview>) {
    let ctx = ctx.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        let rect = ui.max_rect();
        let painter = ui.painter();

        if let Some(h) = &preview.header {
            // Fill with theater tone
            let bg = theater_color(h.theater);
            painter.rect_filled(rect, 0.0, bg);

            // Draw an isometric tile grid scaled to panel
            draw_iso_grid(ui, rect, h.width, h.height);
            // Overlay label
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Map: {}x{}  •  Theater: {:?}",
                        h.width, h.height, h.theater
                    ))
                    .size(14.0)
                    .color(egui::Color32::from_rgb(240, 240, 245)),
                );
                ui.small("Preview: tiles only (no objects/units yet)");
            });
        } else {
            // No map loaded yet: neutral background
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(24, 27, 33));
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        egui::RichText::new("Workspace")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(180, 190, 200)),
                    );
                    ui.label(
                        egui::RichText::new("Open a folder, then click a .map to preview its theater grid.")
                            .size(13.0)
                            .color(egui::Color32::from_gray(160)),
                    );
                });
            });
        }
    });
}

/// Lightweight isometric grid renderer (no textures).
fn draw_iso_grid(ui: &mut egui::Ui, rect: egui::Rect, w_tiles: i32, h_tiles: i32) {
    let painter = ui.painter();

    // Tile pixel size relative to panel; fit width nicely
    let panel_w = rect.width();
    let _panel_h = rect.height(); // not used now, keep for future scaling

    // For iso diamonds: width is ~tile_w, height is ~tile_h (tile_h = tile_w/2 looks RA2-ish)
    let tile_w = (panel_w / (w_tiles as f32 + h_tiles as f32).max(1.0)) * 2.0;
    let tile_h = tile_w * 0.5;

    // Center the map on screen
    let map_w_px = (w_tiles as f32 + h_tiles as f32) * (tile_w * 0.5);
    let map_h_px = (w_tiles as f32 + h_tiles as f32) * (tile_h * 0.5);
    let origin = egui::pos2(
        rect.center().x - map_w_px * 0.5,
        rect.center().y - map_h_px * 0.25, // lift a bit for header text
    );

    let grid_color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 64);

    // Draw grid lines (diamond)
    for y in 0..=h_tiles {
        let a = cell_to_screen(0.0, y as f32, tile_w, tile_h, origin);
        let b = cell_to_screen(w_tiles as f32, y as f32, tile_w, tile_h, origin);
        painter.line_segment([a, b], (1.0, grid_color));
    }
    for x in 0..=w_tiles {
        let a = cell_to_screen(x as f32, 0.0, tile_w, tile_h, origin);
        let b = cell_to_screen(x as f32, h_tiles as f32, tile_w, tile_h, origin);
        painter.line_segment([a, b], (1.0, grid_color));
    }
}

fn cell_to_screen(cx: f32, cy: f32, tile_w: f32, tile_h: f32, origin: egui::Pos2) -> egui::Pos2 {
    // Diamond iso projection:
    // x' = (x - y) * w/2
    // y' = (x + y) * h/2
    let x = (cx - cy) * (tile_w * 0.5);
    let y = (cx + cy) * (tile_h * 0.5);
    egui::pos2(origin.x + x, origin.y + y)
}
