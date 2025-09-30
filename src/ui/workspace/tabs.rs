use bevy::prelude::EventWriter;
use bevy_egui::egui;

use crate::backend::{EditorLayout, OpenMap, ProjectState, WorkspaceCommand};

pub(super) fn show_tabs(
    ctx: &egui::Context,
    layout: &mut EditorLayout,
    project: &ProjectState,
    open_map_writer: &mut EventWriter<OpenMap>,
    workspace_writer: &mut EventWriter<WorkspaceCommand>,
) {
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
}
