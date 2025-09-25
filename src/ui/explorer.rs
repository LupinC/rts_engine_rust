use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::backend::{EditorLayout, Node, NodeKind, OpenMap, ProjectState};

const INDENT_PER_LEVEL: f32 = 14.0;
const ROW_HEIGHT: f32 = 22.0;
const ICON_SPACE: f32 = 18.0;

pub fn ui_explorer(
    mut ctx: EguiContexts,
    project: Res<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    mut ev_open_map: EventWriter<OpenMap>,
) {
    let ctx = ctx.ctx_mut();

    egui::SidePanel::left("left/explorer")
        .default_width(240.0)
        .min_width(200.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("EXPLORER");
            });
            ui.add_space(6.0);

            if let Some(root) = &project.root {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        draw_node(ui, root, 0, &mut layout, &mut ev_open_map);
                    });
            } else {
                ui.label(egui::RichText::new("Open a folder to view files").italics());
            }
        });
}

fn draw_node(
    ui: &mut egui::Ui,
    node: &Node,
    depth: usize,
    layout: &mut EditorLayout,
    ev_open_map: &mut EventWriter<OpenMap>,
) {
    match &node.kind {
        NodeKind::Folder { children } => {
            let opened = layout.open_folders.contains(&node.id);

            // Single interaction path: allocate with Sense::click and use its response.
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), ROW_HEIGHT),
                egui::Sense::click(),
            );

            // Hover background
            if resp.hovered() {
                ui.painter().rect_filled(
                    rect,
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
                );
            }

            // Draw triangle + folder icon + name
            let indent = INDENT_PER_LEVEL * depth as f32;
            let mut cursor = rect.left_top() + egui::vec2(indent, 0.0);
            let triangle = if opened { "▼" } else { "▶" };
            let folder_icon = "📁";
            let color = egui::Color32::from_gray(210);

            ui.painter().text(
                cursor + egui::vec2(2.0, 3.0),
                egui::Align2::LEFT_TOP,
                triangle,
                egui::FontId::monospace(14.0),
                color,
            );
            cursor.x += ICON_SPACE;

            ui.painter().text(
                cursor + egui::vec2(0.0, 3.0),
                egui::Align2::LEFT_TOP,
                folder_icon,
                egui::FontId::monospace(14.0),
                color,
            );
            cursor.x += ICON_SPACE;

            ui.painter().text(
                cursor + egui::vec2(0.0, 3.0),
                egui::Align2::LEFT_TOP,
                &node.name,
                egui::FontId::proportional(14.0),
                egui::Color32::from_gray(230),
            );

            // Toggle open/closed
            if resp.clicked() {
                if opened {
                    layout.open_folders.remove(&node.id);
                } else {
                    layout.open_folders.insert(node.id.clone());
                }
            }

            // Children
            if opened {
                for child in children {
                    draw_node(ui, child, depth + 1, layout, ev_open_map);
                }
            }
        }
        NodeKind::File { path, ext } => {
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), ROW_HEIGHT),
                egui::Sense::click(),
            );

            if resp.hovered() {
                ui.painter().rect_filled(
                    rect,
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8),
                );
            }

            let indent = INDENT_PER_LEVEL * depth as f32 + ICON_SPACE; // align under folder text
            let mut cursor = rect.left_top() + egui::vec2(indent, 0.0);

            let file_icon = "📄";
            let color = egui::Color32::from_gray(200);

            ui.painter().text(
                cursor + egui::vec2(0.0, 3.0),
                egui::Align2::LEFT_TOP,
                file_icon,
                egui::FontId::monospace(14.0),
                color,
            );
            cursor.x += ICON_SPACE;

            ui.painter().text(
                cursor + egui::vec2(0.0, 3.0),
                egui::Align2::LEFT_TOP,
                &node.name,
                egui::FontId::proportional(14.0),
                egui::Color32::from_gray(230),
            );

            if resp.clicked() {
                if ext.to_ascii_lowercase() == "map" {
                    ev_open_map.send(OpenMap { path: path.clone() });
                }
                // else: intentionally do nothing
            }
        }
    }
}
