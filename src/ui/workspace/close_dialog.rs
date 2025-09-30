use bevy::prelude::EventWriter;
use bevy_egui::egui;

use crate::backend::{EditorLayout, WorkspaceCommand};

pub(super) fn handle_pending_close(
    ctx: &egui::Context,
    layout: &mut EditorLayout,
    workspace_writer: &mut EventWriter<WorkspaceCommand>,
) {
    if let Some(pending) = layout.pending_close.clone() {
        if !pending.requires_save {
            workspace_writer.send(WorkspaceCommand::CloseMap {
                path: pending.path.clone(),
            });
            layout.clear_pending_close();
            return;
        }

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
            return;
        }

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
