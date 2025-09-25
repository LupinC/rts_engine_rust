use bevy_egui::{egui, EguiContexts};
use crate::backend::{MapPreview, MapView, theater_color};

pub fn ui_workspace(
    mut ctx: EguiContexts,
    preview: bevy::prelude::Res<MapPreview>,
    mut view: bevy::prelude::ResMut<MapView>,
) {
    let ctx = ctx.ctx_mut();

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::BLACK)) // black outside the map
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            // Interact area for pan/zoom
            let id = ui.make_persistent_id("workspace_canvas");
            let response = ui.interact(rect, id, egui::Sense::click_and_drag());

            // ---- Right-mouse drag to pan (per-frame delta) ----
            let delta = ui.input(|i| {
                if i.pointer.secondary_down() && response.hovered() {
                    i.pointer.delta()
                } else {
                    egui::Vec2::ZERO
                }
            });
            if delta != egui::Vec2::ZERO {
                view.offset += delta;
            }

            // ---- Mouse wheel zoom (scroll up => zoom in) ----
            let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_y != 0.0 {
                let old_zoom = view.zoom;
                let zoom_step = 1.0 + (-scroll_y * 0.0015);
                view.zoom = (view.zoom * zoom_step).clamp(0.2, 5.0);

                // keep cursor anchored while zooming
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let center = rect.center();
                    let from_center_old = pos - center - view.offset;
                    let from_center_new = from_center_old * (view.zoom / old_zoom);
                    view.offset += from_center_old - from_center_new;
                }
            }

            if let Some(h) = &preview.header {
                // Map fill (theater diamond) + grid
                let bg = theater_color(h.theater);

                // Tile size based on panel width and zoom
                let panel_w = rect.width();
                let w_tiles = h.width.max(1) as f32;
                let h_tiles = h.height.max(1) as f32;
                let base_tile_w = (panel_w / (w_tiles + h_tiles)) * 2.0;
                let tile_w = base_tile_w * view.zoom;
                let tile_h = tile_w * 0.5;

                // Base origin centered; then apply panning offset
                let map_w_px = (w_tiles + h_tiles) * (tile_w * 0.5);
                let map_h_px = (w_tiles + h_tiles) * (tile_h * 0.5);
                let base_origin = egui::pos2(
                    rect.center().x - map_w_px * 0.5,
                    rect.center().y - map_h_px * 0.25,
                );
                let origin = base_origin + view.offset;

                // Fill the diamond first (outside remains black)
                let left   = cell_to_screen(0.0, 0.0,          tile_w, tile_h, origin);
                let top    = cell_to_screen(w_tiles, 0.0,      tile_w, tile_h, origin);
                let right  = cell_to_screen(w_tiles, h_tiles,  tile_w, tile_h, origin);
                let bottom = cell_to_screen(0.0,     h_tiles,  tile_w, tile_h, origin);

                painter.add(egui::Shape::convex_polygon(
                    vec![left, top, right, bottom],
                    bg,
                    egui::Stroke::NONE,
                ));

                // Grid on top
                draw_iso_grid(ui, origin, h.width, h.height, tile_w, tile_h);

                // Overlay label
                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "Map: {}x{}  •  Zoom: {:.2}x",
                            h.width, h.height, view.zoom
                        ))
                        .size(14.0)
                        .color(egui::Color32::from_rgb(240, 240, 245)),
                    );
                    ui.small("Right-click drag to pan • Scroll to zoom");
                });
            } else {
                // No map loaded yet
                painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(
                            egui::RichText::new("Workspace")
                                .size(18.0)
                                .color(egui::Color32::from_gray(200)),
                        );
                        ui.label(
                            egui::RichText::new("Open a folder, then click a .map to preview it. Right-click drag to pan, scroll to zoom.")
                                .size(13.0)
                                .color(egui::Color32::from_gray(150)),
                        );
                    });
                });
            }
        });
}

/// Iso grid lines (diamond) using current origin & tile sizes.
fn draw_iso_grid(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    w_tiles: i32,
    h_tiles: i32,
    tile_w: f32,
    tile_h: f32,
) {
    let painter = ui.painter();
    let grid = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80);

    for y in 0..=h_tiles {
        let a = cell_to_screen(0.0, y as f32, tile_w, tile_h, origin);
        let b = cell_to_screen(w_tiles as f32, y as f32, tile_w, tile_h, origin);
        painter.add(egui::Shape::line_segment([a, b], egui::Stroke::new(1.0, grid)));
    }
    for x in 0..=w_tiles {
        let a = cell_to_screen(x as f32, 0.0, tile_w, tile_h, origin);
        let b = cell_to_screen(x as f32, h_tiles as f32, tile_w, tile_h, origin);
        painter.add(egui::Shape::line_segment([a, b], egui::Stroke::new(1.0, grid)));
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
