use bevy::prelude::{EventWriter, Res, ResMut};
use bevy_egui::{EguiContexts, egui};

use crate::backend::{EditorLayout, ExplorerCommand, OpenMap, ProjectState};

use super::tree;

pub fn ui_explorer(
    mut ctx: EguiContexts,
    project: Res<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    mut ev_open_map: EventWriter<OpenMap>,
    mut ev_command: EventWriter<ExplorerCommand>,
) {
    let egui_ctx = ctx.ctx_mut();

    egui::SidePanel::left("left/explorer")
        .default_width(240.0)
        .min_width(200.0)
        .resizable(true)
        .show(egui_ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("EXPLORER");
            });
            ui.add_space(6.0);

            if let Some(root) = &project.root {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        tree::render_tree(
                            ui,
                            root,
                            layout.as_mut(),
                            &mut ev_open_map,
                            &mut ev_command,
                        );
                    });
            } else {
                ui.label(egui::RichText::new("Open a folder to view files").italics());
            }
        });
}
