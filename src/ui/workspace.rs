use bevy_egui::{egui, EguiContexts};
use crate::backend::{MapPreview, MapView, WorkspaceSettings, theater_color};

pub fn ui_workspace(
    mut ctx: EguiContexts,
    preview: bevy::prelude::Res<MapPreview>,
    mut view: bevy::prelude::ResMut<MapView>,
    mut settings: bevy::prelude::ResMut<WorkspaceSettings>,
) {
    let ctx = ctx.ctx_mut();

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::BLACK))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter().clone(); // avoid borrowing ui

            // Interaction
            let id = ui.make_persistent_id("workspace_canvas");
            let response = ui.interact(rect, id, egui::Sense::click_and_drag());

            // Pan with right or middle mouse
            let delta = ui.input(|i| {
                if (i.pointer.secondary_down() || i.pointer.middle_down()) && response.hovered() {
                    i.pointer.delta()
                } else {
                    egui::Vec2::ZERO
                }
            });
            if delta != egui::Vec2::ZERO {
                view.offset += delta;
            }

            // Zoom with wheel
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

            // Left click selects tile (if any)
            let left_clicked = ui.input(|i| i.pointer.any_released())
                && ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
                && response.hovered();

            if let Some(h) = &preview.header {
                // Geometry
                let panel_w = rect.width();
                let w_tiles = h.width.max(1) as f32;
                let h_tiles = h.height.max(1) as f32;
                let base_tile_w = (panel_w / (w_tiles + h_tiles)) * 2.0;
                let tile_w = base_tile_w * view.zoom;
                let tile_h = tile_w * 0.5;

                // Center the map by placing the tile-space midpoint (w/2, h/2)
                // exactly at the panel center, then apply panning offset.
                let cx = w_tiles * 0.5;
                let cy = h_tiles * 0.5;
                let center_offset = egui::vec2(
                    (cx - cy) * (tile_w * 0.5),
                    (cx + cy) * (tile_h * 0.5),
                );
                // If origin were (0,0), the midpoint would be at `center_offset`.
                // To put the midpoint at `rect.center()`, move origin by -center_offset.
                let origin = egui::pos2(
                    rect.center().x - center_offset.x + view.offset.x,
                    rect.center().y - center_offset.y + view.offset.y,
                );

                // Fill diamond
                let bg = theater_color(h.theater);
                let left   = cell_to_screen(0.0, 0.0,          tile_w, tile_h, origin);
                let top    = cell_to_screen(w_tiles, 0.0,      tile_w, tile_h, origin);
                let right  = cell_to_screen(w_tiles, h_tiles,  tile_w, tile_h, origin);
                let bottom = cell_to_screen(0.0,     h_tiles,  tile_w, tile_h, origin);
                painter.add(egui::Shape::convex_polygon(
                    vec![left, top, right, bottom],
                    bg,
                    egui::Stroke::NONE,
                ));

                // Optional grid
                if settings.show_grid {
                    draw_iso_grid(&painter, origin, h.width, h.height, tile_w, tile_h);
                }

                // Selection
                if left_clicked {
                    if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                        if let Some((cx, cy)) = pick_cell(cursor, origin, tile_w, tile_h, h.width, h.height) {
                            settings.selected = Some((cx, cy));
                        } else {
                            settings.selected = None;
                        }
                    }
                }

                // Draw selection highlight (center at i+0.5, j+0.5)
                if let Some((sx, sy)) = settings.selected {
                    let diamond = diamond_points(
                        sx as f32 + 0.5,
                        sy as f32 + 0.5,
                        tile_w,
                        tile_h,
                        origin,
                    );
                    painter.add(egui::Shape::closed_line(
                        diamond.to_vec(),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(250, 230, 80)),
                    ));
                }

                // Overlay mini UI (top-right)
                // Overlay mini UI (top-right)
                egui::Area::new("workspace_overlay".into())
                    .movable(false)
                    .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
                    .show(ui.ctx(), |ui| {
                        egui::Frame::window(ui.ctx().style().as_ref())
                            .rounding(egui::Rounding::same(6.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("Fit").clicked() {
                                        // Map extents at zoom = 1.0
                                        let map_w_1: f32 = rect.width();
                                        let map_h_1: f32 = rect.width() * 0.5;

                                        // Compute the zoom that fits both dimensions
                                        let s_w: f32 = (rect.width() / map_w_1).min(1.0);      // always 1.0
                                        let s_h: f32 = (rect.height() / map_h_1).min(1.0);     // <= 1.0 if height is limiting
                                        let s: f32 = s_w.min(s_h);                             // final fit zoom

                                        // Apply and center
                                        view.zoom = s.clamp(0.2, 5.0);
                                        view.offset = egui::Vec2::ZERO;

                                        // Make the change visible immediately
                                        ui.ctx().request_repaint();
                                    }

                                    if ui.button("Reset").clicked() {
                                        view.zoom = 1.0;
                                        view.offset = egui::Vec2::ZERO;
                                    }
                                    ui.separator();
                                    ui.toggle_value(&mut settings.show_grid, "Grid");
                                });
                                if let Some((sx, sy)) = settings.selected {
                                    ui.label(format!("Tile: {}, {}", sx, sy));
                                } else {
                                    ui.label("Tile: —");
                                }
                                ui.label(format!("Zoom: {:.2}x", view.zoom));
                            });
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
                            egui::RichText::new("Open a folder, then click a .map to preview it. Right/Middle drag to pan, scroll to zoom.")
                                .size(13.0)
                                .color(egui::Color32::from_gray(150)),
                        );
                    });
                });
            }
        });
}

fn draw_iso_grid(
    painter: &egui::Painter,
    origin: egui::Pos2,
    w_tiles: i32,
    h_tiles: i32,
    tile_w: f32,
    tile_h: f32,
) {
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
    // x' = (x - y) * w/2
    // y' = (x + y) * h/2
    let x = (cx - cy) * (tile_w * 0.5);
    let y = (cx + cy) * (tile_h * 0.5);
    egui::pos2(origin.x + x, origin.y + y)
}

fn screen_to_cell(px: f32, py: f32, tile_w: f32, tile_h: f32, origin: egui::Pos2) -> (f32, f32) {
    // Inverse mapping
    let dx = px - origin.x;
    let dy = py - origin.y;
    let a = dx / (tile_w * 0.5);
    let b = dy / (tile_h * 0.5);
    let cx = (a + b) * 0.5;
    let cy = (b - a) * 0.5;
    (cx, cy)
}

fn pick_cell(
    mouse: egui::Pos2,
    origin: egui::Pos2,
    tile_w: f32,
    tile_h: f32,
    w_tiles: i32,
    h_tiles: i32,
) -> Option<(i32, i32)> {
    let (cx, cy) = screen_to_cell(mouse.x, mouse.y, tile_w, tile_h, origin);
    // Use floor to map continuous coords to the tile index
    let sx = cx.floor() as i32;
    let sy = cy.floor() as i32;
    if sx >= 0 && sy >= 0 && sx < w_tiles && sy < h_tiles {
        Some((sx, sy))
    } else {
        None
    }
}

fn diamond_points(cx_center: f32, cy_center: f32, tile_w: f32, tile_h: f32, origin: egui::Pos2) -> [egui::Pos2; 4] {
    // (cx_center, cy_center) are tile-center coords (i + 0.5, j + 0.5)
    let c = cell_to_screen(cx_center, cy_center, tile_w, tile_h, origin);
    [
        egui::pos2(c.x - tile_w * 0.5, c.y),              // left
        egui::pos2(c.x,                 c.y - tile_h*0.5),// top
        egui::pos2(c.x + tile_w * 0.5, c.y),              // right
        egui::pos2(c.x,                 c.y + tile_h*0.5),// bottom
    ]
}
