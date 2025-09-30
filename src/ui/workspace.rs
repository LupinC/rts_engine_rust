use crate::backend::{
    EditorLayout, EditorObjects, MapPreview, MapView, OpenMap, ProjectState, Tool, ToolState,
    WorkspaceCommand, WorkspaceSettings, theater_color,
};
use bevy_egui::{EguiContexts, egui};

pub fn ui_workspace(
    mut ctx: EguiContexts,
    mut layout: bevy::prelude::ResMut<EditorLayout>,
    preview: bevy::prelude::Res<MapPreview>,
    mut view: bevy::prelude::ResMut<MapView>,
    mut settings: bevy::prelude::ResMut<WorkspaceSettings>,
    tool: bevy::prelude::ResMut<ToolState>,
    objs: bevy::prelude::ResMut<EditorObjects>,
    project: bevy::prelude::Res<ProjectState>,
    mut open_map_writer: bevy::prelude::EventWriter<OpenMap>,
    mut workspace_writer: bevy::prelude::EventWriter<WorkspaceCommand>,
) {
    let ctx = ctx.ctx_mut();

    if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.command) {
        workspace_writer.send(WorkspaceCommand::SaveActive);
    }

    egui::TopBottomPanel::top("workspace/tabs")
        .exact_height(24.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if project.open_maps.is_empty() {
                    ui.label(
                        egui::RichText::new("No maps open")
                            .italics()
                            .color(egui::Color32::from_gray(150)),
                    );
                } else {
                    for entry in &project.open_maps {
                        let is_active = project.active_map.as_deref() == Some(&entry.path);
                        let is_dirty = project.is_dirty(&entry.path);
                        let label = if is_dirty {
                            format!("{} •", entry.name)
                        } else {
                            entry.name.clone()
                        };
                        let response = ui.add(egui::SelectableLabel::new(is_active, label));

                        if response.clicked() && !is_active {
                            open_map_writer.send(OpenMap {
                                path: entry.path.clone(),
                            });
                        }

                        if response.middle_clicked() {
                            if is_dirty {
                                layout.request_close(entry.path.clone(), entry.name.clone(), true);
                            } else {
                                workspace_writer.send(WorkspaceCommand::CloseMap {
                                    path: entry.path.clone(),
                                });
                            }
                        }

                        response.context_menu(|ui| {
                            if ui.button("Close").clicked() {
                                if project.is_dirty(&entry.path) {
                                    layout.request_close(
                                        entry.path.clone(),
                                        entry.name.clone(),
                                        true,
                                    );
                                } else {
                                    workspace_writer.send(WorkspaceCommand::CloseMap {
                                        path: entry.path.clone(),
                                    });
                                }
                                ui.close_menu();
                            }
                        });
                    }
                }
            });
        });

    if let Some(pending) = layout.pending_close.clone() {
        if !pending.requires_save {
            workspace_writer.send(WorkspaceCommand::CloseMap {
                path: pending.path.clone(),
            });
            layout.clear_pending_close();
        } else {
            enum CloseAction {
                None,
                Save,
                Discard,
                Cancel,
            }

            let mut action = CloseAction::None;
            let mut keep_open = true;
            egui::Window::new("Unsaved changes")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .open(&mut keep_open)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "Save changes to \"{}\" before closing?",
                            pending.name
                        ));
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                action = CloseAction::Save;
                            }
                            if ui.button("Don't Save").clicked() {
                                action = CloseAction::Discard;
                            }
                            if ui.button("Cancel").clicked() {
                                action = CloseAction::Cancel;
                            }
                        });
                    });
                });

            if !keep_open {
                layout.clear_pending_close();
            } else {
                match action {
                    CloseAction::Save => {
                        workspace_writer.send(WorkspaceCommand::SaveAndClose {
                            path: pending.path.clone(),
                        });
                        layout.clear_pending_close();
                    }
                    CloseAction::Discard => {
                        workspace_writer.send(WorkspaceCommand::CloseMap {
                            path: pending.path.clone(),
                        });
                        layout.clear_pending_close();
                    }
                    CloseAction::Cancel => {
                        layout.clear_pending_close();
                    }
                    CloseAction::None => {}
                }
            }
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::BLACK))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter().clone();

            // Interaction scaffold
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
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let center = rect.center();
                    let from_center_old = pos - center - view.offset;
                    let from_center_new = from_center_old * (view.zoom / old_zoom);
                    view.offset += from_center_old - from_center_new;
                }
            }

            // Left click behavior (kept for future interactions)
            let left_clicked = ui.input(|i| i.pointer.any_released())
                && ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
                && response.hovered();

            if let Some(m) = &preview.map {
                // Geometry
                let panel_w = rect.width();
                let w_tiles = m.width.max(1) as f32;
                let h_tiles = m.height.max(1) as f32;
                let base_tile_w = (panel_w / (w_tiles + h_tiles)) * 2.0;
                let tile_w = base_tile_w * view.zoom;
                let tile_h = tile_w * 0.5;

                // True centering on tile-space midpoint + pan
                let cx = w_tiles * 0.5;
                let cy = h_tiles * 0.5;
                let center_offset = egui::vec2(
                    (cx - cy) * (tile_w * 0.5),
                    (cx + cy) * (tile_h * 0.5),
                );
                let origin = egui::pos2(
                    rect.center().x - center_offset.x + view.offset.x,
                    rect.center().y - center_offset.y + view.offset.y,
                );

                // Fill diamond with theater color
                let bg = theater_color(m.theater);
                let left  = cell_to_screen(0.0,      0.0,       tile_w, tile_h, origin);
                let top   = cell_to_screen(w_tiles,  0.0,       tile_w, tile_h, origin);
                let right = cell_to_screen(w_tiles,  h_tiles,   tile_w, tile_h, origin);
                let bottom= cell_to_screen(0.0,      h_tiles,   tile_w, tile_h, origin);
                painter.add(egui::Shape::convex_polygon(
                    vec![left, top, right, bottom], bg, egui::Stroke::NONE,
                ));

                // Grid
                if settings.show_grid {
                    draw_iso_grid(&painter, origin, m.width, m.height, tile_w, tile_h);
                }

                // --- Display parsed items ---

                // Waypoints (W1..)
                for (i, (wx, wy)) in m.waypoints.iter().enumerate() {
                    // Convert global coords to local (subtract local origin, if any)
                    let lx = *wx - m.local_origin_x;
                    let ly = *wy - m.local_origin_y;
                    if lx < 0 || ly < 0 || lx >= m.width || ly >= m.height {
                        continue; // out of local bounds; skip
                    }

                    let color = if i < m.num_starting_points { egui::Color32::from_rgb(60,220,120) }
                                else { egui::Color32::from_rgb(245,210,60) };

                    draw_marker_circle(
                        &painter,
                        lx as f32 + 0.5,
                        ly as f32 + 0.5,
                        tile_w, tile_h, origin,
                        color,
                    );

                    let pos = cell_to_screen(lx as f32 + 0.5, ly as f32 + 0.2, tile_w, tile_h, origin);
                    painter.text(
                        pos,
                        egui::Align2::CENTER_TOP,
                        format!("W{}", i + 1),
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE,
                    );
                }

                // Units (blue triangles)
                for u in &m.units {
                    let lx = u.x - m.local_origin_x;
                    let ly = u.y - m.local_origin_y;
                    if lx < 0 || ly < 0 || lx >= m.width || ly >= m.height {
                        continue;
                    }
                    draw_marker_triangle(
                        &painter,
                        lx as f32 + 0.5,
                        ly as f32 + 0.5,
                        tile_w, tile_h, origin,
                        egui::Color32::from_rgb(60,200,245),
                    );
                }

                // Structures (red squares/diamonds)
                for s in &m.structures {
                    let lx = s.x - m.local_origin_x;
                    let ly = s.y - m.local_origin_y;
                    if lx < 0 || ly < 0 || lx >= m.width || ly >= m.height {
                        continue;
                    }
                    draw_marker_diamond(
                        &painter,
                        lx as f32 + 0.5,
                        ly as f32 + 0.5,
                        tile_w, tile_h, origin,
                        egui::Color32::from_rgb(220,80,80),
                    );
                }

                // Click → selection (kept; no placing in this version)
                if left_clicked {
                    if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                        if let Some((cx, cy)) = pick_cell(cursor, origin, tile_w, tile_h, m.width, m.height) {
                            // With tools still present in state, retain select behavior only.
                            match tool.current {
                                Tool::Select => settings.selected = Some((cx, cy)),
                                _ => {
                                    // No placement in this version (just display).
                                    settings.selected = Some((cx, cy));
                                }
                            }
                        }
                    }
                }

                // Draw user-placed markers from previous sessions (optional)
                for p in &objs.items {
                    draw_marker_kind(&painter, p.kind, p.x as f32 + 0.5, p.y as f32 + 0.5, tile_w, tile_h, origin);
                }

                // Selection ring
                if let Some((sx, sy)) = settings.selected {
                    let d = diamond_points(sx as f32 + 0.5, sy as f32 + 0.5, tile_w, tile_h, origin);
                    painter.add(egui::Shape::closed_line(
                        d.to_vec(),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(250, 230, 80)),
                    ));
                }

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
                                        // Fit height at zoom <= 1
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
                            egui::RichText::new(
                                "Create or open a project, then pick a .mpr map from the Explorer.\nRight/Middle drag to pan, scroll to zoom.\nRight-click folders to add new maps or subfolders."
                            )
                            .size(13.0)
                            .color(egui::Color32::from_gray(150)),
                        );
                    });
                });
            }
        });
}

// ---------- drawing helpers ----------

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
        painter.add(egui::Shape::line_segment(
            [a, b],
            egui::Stroke::new(1.0, grid),
        ));
    }
    for x in 0..=w_tiles {
        let a = cell_to_screen(x as f32, 0.0, tile_w, tile_h, origin);
        let b = cell_to_screen(x as f32, h_tiles as f32, tile_w, tile_h, origin);
        painter.add(egui::Shape::line_segment(
            [a, b],
            egui::Stroke::new(1.0, grid),
        ));
    }
}

fn draw_marker_circle(
    painter: &egui::Painter,
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
    color: egui::Color32,
) {
    let c = cell_to_screen(cx_center, cy_center, tile_w, tile_h, origin);
    painter.circle_filled(c, (tile_h * 0.4) as f32, color);
    painter.circle_stroke(
        c,
        (tile_h * 0.4) as f32,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    );
}

fn draw_marker_triangle(
    painter: &egui::Painter,
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
    color: egui::Color32,
) {
    let size = tile_h * 0.55;
    let c = cell_to_screen(cx_center, cy_center, tile_w, tile_h, origin);
    let p1 = egui::pos2(c.x, c.y - size * 0.8);
    let p2 = egui::pos2(c.x - size * 0.7, c.y + size * 0.5);
    let p3 = egui::pos2(c.x + size * 0.7, c.y + size * 0.5);
    painter.add(egui::Shape::convex_polygon(
        vec![p1, p2, p3],
        color,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    ));
}

fn draw_marker_diamond(
    painter: &egui::Painter,
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
    color: egui::Color32,
) {
    let d = diamond_points(cx_center, cy_center, tile_w, tile_h, origin);
    painter.add(egui::Shape::convex_polygon(
        d.to_vec(),
        color,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    ));
}

fn draw_marker_kind(
    painter: &egui::Painter,
    kind: Tool,
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
) {
    match kind {
        Tool::Spawn => draw_marker_circle(
            painter,
            cx_center,
            cy_center,
            tile_w,
            tile_h,
            origin,
            egui::Color32::from_rgb(60, 220, 120),
        ),
        Tool::Resource => draw_marker_diamond(
            painter,
            cx_center,
            cy_center,
            tile_w,
            tile_h,
            origin,
            egui::Color32::from_rgb(245, 210, 60),
        ),
        Tool::Unit => draw_marker_triangle(
            painter,
            cx_center,
            cy_center,
            tile_w,
            tile_h,
            origin,
            egui::Color32::from_rgb(60, 200, 245),
        ),
        Tool::Select => {}
    }
}

fn cell_to_screen(cx: f32, cy: f32, tile_w: f32, tile_h: f32, origin: egui::Pos2) -> egui::Pos2 {
    let x = (cx - cy) * (tile_w * 0.5);
    let y = (cx + cy) * (tile_h * 0.5);
    egui::pos2(origin.x + x, origin.y + y)
}

fn screen_to_cell(px: f32, py: f32, tile_w: f32, tile_h: f32, origin: egui::Pos2) -> (f32, f32) {
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
    let sx = cx.floor() as i32;
    let sy = cy.floor() as i32;
    if sx >= 0 && sy >= 0 && sx < w_tiles && sy < h_tiles {
        Some((sx, sy))
    } else {
        None
    }
}

fn diamond_points(
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
) -> [egui::Pos2; 4] {
    let c = cell_to_screen(cx_center, cy_center, tile_w, tile_h, origin);
    [
        egui::pos2(c.x - tile_w * 0.5, c.y),
        egui::pos2(c.x, c.y - tile_h * 0.5),
        egui::pos2(c.x + tile_w * 0.5, c.y),
        egui::pos2(c.x, c.y + tile_h * 0.5),
    ]
}
