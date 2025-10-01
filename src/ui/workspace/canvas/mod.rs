use bevy::prelude::Vec2;
use bevy_egui::egui;

use crate::backend::{
    map::{IsoStaggered, IsoTileSize},
    EditorObjects, MapPreview, MapView, Tool, WorkspaceSettings, theater_color,
};

use self::drawing::{
    draw_iso_grid, draw_marker_circle, draw_marker_diamond, draw_marker_kind,
    draw_marker_triangle, pick_cell, tile_center, tile_outline_points,
};

mod drawing;

pub(super) fn render_canvas(
    ctx: &egui::Context,
    preview: &MapPreview,
    view: &mut MapView,
    settings: &mut WorkspaceSettings,
    active_tool: Tool,
    objs: &EditorObjects,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::BLACK))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter().clone();

            let id = ui.make_persistent_id("workspace_canvas");
            let response = ui.interact(rect, id, egui::Sense::click_and_drag());

            handle_pan(ui, &response, view);
            handle_zoom(ui, rect, view);
            let left_clicked = detect_left_click(ui, &response);

            if let Some(map) = &preview.map {
                let panel_w = rect.width().max(1.0);
                let panel_h = rect.height().max(1.0);
                let w_tiles = map.width.max(1);
                let h_tiles = map.height.max(1);

                let width_factor = w_tiles as f32 + if h_tiles > 1 { 0.5 } else { 0.0 };
                let height_factor = (h_tiles as f32 + 1.0) * 0.5;

                let base_tile_w = (panel_w / width_factor)
                    .min(panel_h * 2.0 / height_factor)
                    .max(1.0);
                let tile_w = (base_tile_w * view.zoom).max(2.0);
                let tile_h = tile_w * 0.5;

                let mut iso = IsoStaggered {
                    tile: IsoTileSize::new(tile_w, tile_h),
                    origin: Vec2::ZERO,
                };
                let map_size = iso.map_world_size(w_tiles, h_tiles);
                let origin = egui::pos2(
                    rect.center().x - map_size.x * 0.5 + view.offset.x,
                    rect.center().y - map_size.y * 0.5 + view.offset.y,
                );
                iso.origin = Vec2::new(origin.x, origin.y);

                let bg = theater_color(map.theater);
                for y in 0..map.height {
                    for x in 0..map.width {
                        drawing::fill_tile(&painter, &iso, x, y, bg);
                    }
                }

                if settings.show_grid {
                    draw_iso_grid(&painter, &iso, map.width, map.height);
                }

                for (i, (wx, wy)) in map.waypoints.iter().enumerate() {
                    let lx = *wx - map.local_origin_x;
                    let ly = *wy - map.local_origin_y;
                    if lx < 0 || ly < 0 || lx >= map.width || ly >= map.height {
                        continue;
                    }

                    let color = if i < map.num_starting_points {
                        egui::Color32::from_rgb(60, 220, 120)
                    } else {
                        egui::Color32::from_rgb(245, 210, 60)
                    };

                    draw_marker_circle(&painter, &iso, lx, ly, color);

                    let center = tile_center(&iso, lx, ly);
                    let label_pos = egui::pos2(center.x, center.y - iso.tile.h * 0.35 - 6.0);
                    painter.text(
                        label_pos,
                        egui::Align2::CENTER_BOTTOM,
                        format!("W{}", i + 1),
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE,
                    );
                }

                for u in &map.units {
                    let lx = u.x - map.local_origin_x;
                    let ly = u.y - map.local_origin_y;
                    if lx < 0 || ly < 0 || lx >= map.width || ly >= map.height {
                        continue;
                    }
                    draw_marker_triangle(
                        &painter,
                        &iso,
                        lx,
                        ly,
                        egui::Color32::from_rgb(60, 200, 245),
                    );
                }

                for s in &map.structures {
                    let lx = s.x - map.local_origin_x;
                    let ly = s.y - map.local_origin_y;
                    if lx < 0 || ly < 0 || lx >= map.width || ly >= map.height {
                        continue;
                    }
                    draw_marker_diamond(
                        &painter,
                        &iso,
                        lx,
                        ly,
                        egui::Color32::from_rgb(220, 80, 80),
                    );
                }

                if left_clicked {
                    if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                        if let Some((cx, cy)) = pick_cell(cursor, &iso, map.width, map.height) {
                            match active_tool {
                                Tool::Select => settings.selected = Some((cx, cy)),
                                _ => settings.selected = Some((cx, cy)),
                            }
                        }
                    }
                }

                for p in &objs.items {
                    draw_marker_kind(&painter, p.kind, &iso, p.x, p.y);
                }

                if let Some((sx, sy)) = settings.selected {
                    if sx >= 0 && sy >= 0 && sx < map.width && sy < map.height {
                        let outline = tile_outline_points(&iso, sx, sy);
                        painter.add(egui::Shape::closed_line(
                            outline.to_vec(),
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(250, 230, 80)),
                        ));
                    }
                }

                egui::Area::new("workspace_overlay".into())
                    .movable(false)
                    .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
                    .show(ui.ctx(), |ui| {
                        egui::Frame::window(ui.ctx().style().as_ref())
                            .rounding(egui::Rounding::same(6.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("Fit").clicked() {
                                        let map_h_1: f32 = rect.width() * 0.5;
                                        let s_h: f32 = (rect.height() / map_h_1).min(1.0);
                                        let s: f32 = s_h.min(1.0);
                                        view.zoom = s.clamp(0.2, 5.0);
                                        view.offset = egui::Vec2::ZERO;
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
                draw_empty_state(ui, &painter, rect);
            }
        });
}

fn handle_pan(ui: &mut egui::Ui, response: &egui::Response, view: &mut MapView) {
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
}

fn handle_zoom(ui: &mut egui::Ui, rect: egui::Rect, view: &mut MapView) {
    let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
    if scroll_y != 0.0 {
        let old_zoom = view.zoom;
        let zoom_step = 1.0 + (-scroll_y * 0.0015);
        view.zoom = (view.zoom * zoom_step).clamp(0.2, 5.0);
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            let center = rect.center();
            let from_center_old = pos - center - view.offset;
            let from_center_new = from_center_old * (view.zoom / old_zoom);
            view.offset += from_center_old - from_center_new;
        }
    }
}

fn detect_left_click(ui: &egui::Ui, response: &egui::Response) -> bool {
    ui.input(|i| i.pointer.any_released())
        && ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
        && response.hovered()
}

fn draw_empty_state(ui: &mut egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
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
                egui::RichText::new(
                    "Create or open a project, then pick a .mpr map from the Explorer.\nRight/Middle drag to pan, scroll to zoom.\nRight-click folders to add new maps or subfolders."
                )
                .size(13.0)
                .color(egui::Color32::from_gray(150)),
            );
        });
    });
}
