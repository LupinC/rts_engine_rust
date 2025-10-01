use bevy::prelude::EventWriter;
use bevy_egui::egui;

use crate::backend::{EditorLayout, ExplorerCommand, Node, NodeKind, OpenMap};

mod file;
mod folder;
mod rename;

pub(super) const INDENT_PER_LEVEL: f32 = 14.0;
pub(super) const ROW_HEIGHT: f32 = 22.0;
pub(super) const ICON_SPACE: f32 = 18.0;

pub(super) fn render_tree(
    ui: &mut egui::Ui,
    node: &Node,
    layout: &mut EditorLayout,
    ev_open_map: &mut EventWriter<OpenMap>,
    ev_command: &mut EventWriter<ExplorerCommand>,
) {
    draw_node(ui, node, 0, layout, ev_open_map, ev_command);
}

fn draw_node(
    ui: &mut egui::Ui,
    node: &Node,
    depth: usize,
    layout: &mut EditorLayout,
    ev_open_map: &mut EventWriter<OpenMap>,
    ev_command: &mut EventWriter<ExplorerCommand>,
) {
    match &node.kind {
        NodeKind::Folder { children } => {
            let opened = layout.open_folders.contains(&node.id);
            let renaming_this = layout
                .rename
                .as_ref()
                .map_or(false, |r| r.target_path == node.id);

            let row = folder::paint_folder_row(ui, node, depth, opened);

            if row.response.clicked() && !renaming_this {
                if opened {
                    layout.open_folders.remove(&node.id);
                } else {
                    layout.open_folders.insert(node.id.clone());
                }
            }

            row.response.context_menu(|ui| {
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
                match rename::handle_rename(ui, layout, &node.id, &node.name, row.text_rect) {
                    rename::RenameOutcome::Commit {
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
                    rename::RenameOutcome::Cancel => layout.cancel_rename(),
                    rename::RenameOutcome::None => {}
                }
            }

            if opened {
                for child in children {
                    draw_node(ui, child, depth + 1, layout, ev_open_map, ev_command);
                }
            }
        }
        NodeKind::File { path, ext } => {
            let row = file::paint_file_row(ui, node, depth);

            let renaming_this = layout
                .rename
                .as_ref()
                .map_or(false, |r| r.target_path == *path);

            if row.response.clicked() && !renaming_this {
                let ext_lower = ext.to_ascii_lowercase();
                if ext_lower == "mpr" || ext_lower == "map" {
                    ev_open_map.send(OpenMap { path: path.clone() });
                }
            }

            row.response.context_menu(|ui| {
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
                match rename::handle_rename(ui, layout, path, &node.name, row.text_rect) {
                    rename::RenameOutcome::Commit {
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
                    rename::RenameOutcome::Cancel => layout.cancel_rename(),
                    rename::RenameOutcome::None => {}
                }
            }
        }
    }
}
