use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::backend::{EditorLayout, ExplorerCommand, Node, NodeKind, OpenMap, ProjectState};

const INDENT_PER_LEVEL: f32 = 14.0;
const ROW_HEIGHT: f32 = 22.0;
const ICON_SPACE: f32 = 18.0;

pub fn ui_explorer(
    mut ctx: EguiContexts,
    project: Res<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    mut ev_open_map: EventWriter<OpenMap>,
    mut ev_command: EventWriter<ExplorerCommand>,
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
                        draw_node(ui, root, 0, &mut layout, &mut ev_open_map, &mut ev_command);
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
    ev_command: &mut EventWriter<ExplorerCommand>,
) {
    enum RenameAction {
        None,
        Commit {
            target: String,
            value: String,
            original: String,
        },
        Cancel,
    }

    match &node.kind {
        NodeKind::Folder { children } => {
            let opened = layout.open_folders.contains(&node.id);
            let renaming_this = layout
                .rename
                .as_ref()
                .map_or(false, |r| r.target_path == node.id);

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

            // Toggle open/closed (disabled while renaming)
            if resp.clicked() && !renaming_this {
                if opened {
                    layout.open_folders.remove(&node.id);
                } else {
                    layout.open_folders.insert(node.id.clone());
                }
            }

            resp.context_menu(|ui| {
                if ui.button("New Map").clicked() {
                    ev_command.send(ExplorerCommand::NewFile {
                        parent: node.id.clone(),
                    });
                    ui.close_menu();
                }
                if ui.button("New Folder").clicked() {
                    ev_command.send(ExplorerCommand::NewFolder {
                        parent: node.id.clone(),
                    });
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Rename").clicked() {
                    layout.begin_rename(node.id.clone(), node.name.clone());
                    ui.close_menu();
                }
                if ui.button("Delete").clicked() {
                    ev_command.send(ExplorerCommand::DeleteEntry {
                        path: node.id.clone(),
                    });
                    ui.close_menu();
                }
            });

            if renaming_this {
                let mut action = RenameAction::None;
                {
                    if let Some(rename_state) = layout.rename.as_mut() {
                        if rename_state.target_path == node.id {
                            let edit_rect = egui::Rect::from_min_max(
                                egui::pos2(cursor.x, rect.top() + 2.0),
                                egui::pos2(rect.right() - 4.0, rect.bottom() - 2.0),
                            );

                            let response = ui.put(
                                edit_rect,
                                egui::TextEdit::singleline(&mut rename_state.buffer)
                                    .clip_text(false)
                                    .desired_width(edit_rect.width()),
                            );

                            if rename_state.focus_requested {
                                response.request_focus();
                                rename_state.focus_requested = false;
                            }

                            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));

                            if enter {
                                let trimmed = rename_state.buffer.trim().to_string();
                                if trimmed.is_empty() {
                                    action = RenameAction::Cancel;
                                } else {
                                    action = RenameAction::Commit {
                                        target: rename_state.target_path.clone(),
                                        value: trimmed,
                                        original: rename_state.original_name.clone(),
                                    };
                                }
                            } else if escape {
                                action = RenameAction::Cancel;
                            } else if response.lost_focus() && !response.has_focus() {
                                action = RenameAction::Cancel;
                            }
                        }
                    }
                }

                match action {
                    RenameAction::Commit {
                        target,
                        value,
                        original,
                    } => {
                        layout.cancel_rename();
                        if value != original {
                            ev_command.send(ExplorerCommand::RenameEntry {
                                from: target,
                                new_name: value,
                            });
                        }
                    }
                    RenameAction::Cancel => layout.cancel_rename(),
                    RenameAction::None => {}
                }
            }

            // Children
            if opened {
                for child in children {
                    draw_node(ui, child, depth + 1, layout, ev_open_map, ev_command);
                }
            }
        }
        NodeKind::File { path, ext } => {
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), ROW_HEIGHT),
                egui::Sense::click(),
            );

            let renaming_this = layout
                .rename
                .as_ref()
                .map_or(false, |r| r.target_path == *path);

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

            if resp.clicked() && !renaming_this {
                let ext_lower = ext.to_ascii_lowercase();
                if ext_lower == "mpr" || ext_lower == "map" {
                    ev_open_map.send(OpenMap { path: path.clone() });
                }
                // else: intentionally do nothing
            }

            resp.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    layout.begin_rename(path.clone(), node.name.clone());
                    ui.close_menu();
                }
                if ui.button("Delete").clicked() {
                    ev_command.send(ExplorerCommand::DeleteEntry { path: path.clone() });
                    ui.close_menu();
                }
            });

            if renaming_this {
                let mut action = RenameAction::None;
                {
                    if let Some(rename_state) = layout.rename.as_mut() {
                        if rename_state.target_path == *path {
                            let edit_rect = egui::Rect::from_min_max(
                                egui::pos2(cursor.x, rect.top() + 2.0),
                                egui::pos2(rect.right() - 4.0, rect.bottom() - 2.0),
                            );

                            let response = ui.put(
                                edit_rect,
                                egui::TextEdit::singleline(&mut rename_state.buffer)
                                    .clip_text(false)
                                    .desired_width(edit_rect.width()),
                            );

                            if rename_state.focus_requested {
                                response.request_focus();
                                rename_state.focus_requested = false;
                            }

                            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));

                            if enter {
                                let trimmed = rename_state.buffer.trim().to_string();
                                if trimmed.is_empty() {
                                    action = RenameAction::Cancel;
                                } else {
                                    action = RenameAction::Commit {
                                        target: rename_state.target_path.clone(),
                                        value: trimmed,
                                        original: rename_state.original_name.clone(),
                                    };
                                }
                            } else if escape {
                                action = RenameAction::Cancel;
                            } else if response.lost_focus() && !response.has_focus() {
                                action = RenameAction::Cancel;
                            }
                        }
                    }
                }

                match action {
                    RenameAction::Commit {
                        target,
                        value,
                        original,
                    } => {
                        layout.cancel_rename();
                        if value != original {
                            ev_command.send(ExplorerCommand::RenameEntry {
                                from: target,
                                new_name: value,
                            });
                        }
                    }
                    RenameAction::Cancel => layout.cancel_rename(),
                    RenameAction::None => {}
                }
            }
        }
    }
}
