use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::backend::{EditorLayout, Node, NodeKind, ProjectState};

pub fn ui_explorer(
    mut ctx: EguiContexts,
    mut layout: ResMut<EditorLayout>,
    project: Res<ProjectState>,
) {
    if !layout.show_explorer { return; }

    let ctx = ctx.ctx_mut();
    egui::SidePanel::left("explorer")
        .resizable(true)
        .default_width(260.0)
        .min_width(180.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("EXPLORER")
                        .color(egui::Color32::from_gray(180))
                        .size(11.0)
                );
                ui.add_space(6.0);

                if let Some(root) = &project.root {
                    // Ensure root is shown open if not explicitly set yet
                    if !layout.open_folders.contains(&root.id) {
                        // (Optional) leave as-is; systems.rs already opens root on load.
                    }
                    draw_node(ui, root, 0, &mut layout.open_folders);
                } else {
                    ui.label(egui::RichText::new("No folder open").italics());
                    ui.small("Use Folder → Open Folder…");
                }
            });
        });
}

/// Draw a single node with indentation and VS Code–like folder toggling.
/// - Folders: click to expand/collapse.
/// - Files: click does nothing.
fn draw_node(
    ui: &mut egui::Ui,
    node: &Node,
    depth: usize,
    open_set: &mut std::collections::HashSet<String>,
) {
    let indent_px = 12.0 * depth as f32;

    match node.kind {
        NodeKind::Folder => {
            let is_open = open_set.contains(&node.id);

            // Row: [icon] [name]
            let row = ui.horizontal(|ui| {
                ui.add_space(indent_px);

                // Triangle icon like VS Code: ▶ (closed) / ▼ (open)
                let icon = if is_open { "▼" } else { "▶" };
                let icon_resp = ui.add(egui::Label::new(icon).selectable(false));

                // Folder name (clickable)
                let name_resp = ui.selectable_label(false, &node.name);

                // Clicking either the icon or the name toggles open
                if icon_resp.clicked() || name_resp.clicked() {
                    if is_open {
                        open_set.remove(&node.id);
                    } else {
                        open_set.insert(node.id.clone());
                    }
                }
            });
            row.response.on_hover_text(node.id.clone());

            // Children
            if is_open {
                for child in &node.children {
                    draw_node(ui, child, depth + 1, open_set);
                }
            }
        }
        NodeKind::File => {
            // File row: do nothing on click by design
            ui.horizontal(|ui| {
                ui.add_space(indent_px + 12.0); // align with folder text (icon width)
                let resp = ui.selectable_label(false, &node.name);
                if resp.clicked() {
                    // NO-OP (intentionally do nothing)
                }
            });
        }
    }
}
