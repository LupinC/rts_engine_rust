use bevy_egui::egui;

use crate::backend::{EditorObjects, MapPreview, MapView, Tool, WorkspaceSettings, theater_color};

use self::drawing::{
    cell_to_screen, diamond_points, draw_iso_grid, draw_marker_circle, draw_marker_diamond,
    draw_marker_kind, draw_marker_triangle, pick_cell,
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
                let panel_w = rect.width();
                let w_tiles = map.width.max(1) as f32;
                let h_tiles = map.height.max(1) as f32;
                let base_tile_w = (panel_w / (w_tiles + h_tiles)) * 2.0;
                let tile_w = base_tile_w * view.zoom;
                let tile_h = tile_w * 0.5;

                let cx = w_tiles * 0.5;
                let cy = h_tiles * 0.5;
                let center_offset =
                    egui::vec2((cx - cy) * (tile_w * 0.5), (cx + cy) * (tile_h * 0.5));
                let origin = egui::pos2(
                    rect.center().x - center_offset.x + view.offset.x,
                    rect.center().y - center_offset.y + view.offset.y,
                );

                let bg = theater_color(map.theater);
                let left = cell_to_screen(0.0, 0.0, tile_w, tile_h, origin);
                let top = cell_to_screen(w_tiles, 0.0, tile_w, tile_h, origin);
                let right = cell_to_screen(w_tiles, h_tiles, tile_w, tile_h, origin);
                let bottom = cell_to_screen(0.0, h_tiles, tile_w, tile_h, origin);
                painter.add(egui::Shape::convex_polygon(
                    vec![left, top, right, bottom],
                    bg,
                    egui::Stroke::NONE,
                ));

                if settings.show_grid {
                    draw_iso_grid(&painter, origin, map.width, map.height, tile_w, tile_h);
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

                    draw_marker_circle(
                        &painter,
                        lx as f32 + 0.5,
                        ly as f32 + 0.5,
                        tile_w,
                        tile_h,
                        origin,
                        color,
                    );

                    let pos =
                        cell_to_screen(lx as f32 + 0.5, ly as f32 + 0.2, tile_w, tile_h, origin);
                    painter.text(
                        pos,
                        egui::Align2::CENTER_TOP,
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
                        lx as f32 + 0.5,
                        ly as f32 + 0.5,
                        tile_w,
                        tile_h,
                        origin,
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
                        lx as f32 + 0.5,
                        ly as f32 + 0.5,
                        tile_w,
                        tile_h,
                        origin,
                        egui::Color32::from_rgb(220, 80, 80),
                    );
                }

                if left_clicked {
                    if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                        if let Some((cx, cy)) =
                            pick_cell(cursor, origin, tile_w, tile_h, map.width, map.height)
                        {
                            match active_tool {
                                Tool::Select => settings.selected = Some((cx, cy)),
                                _ => settings.selected = Some((cx, cy)),
                            }
                        }
                    }
                }

                for p in &objs.items {
                    draw_marker_kind(
                        &painter,
                        p.kind,
                        p.x as f32 + 0.5,
                        p.y as f32 + 0.5,
                        tile_w,
                        tile_h,
                        origin,
                    );
                }

                if let Some((sx, sy)) = settings.selected {
                    let d =
                        diamond_points(sx as f32 + 0.5, sy as f32 + 0.5, tile_w, tile_h, origin);
                    painter.add(egui::Shape::closed_line(
                        d.to_vec(),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(250, 230, 80)),
                    ));
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
