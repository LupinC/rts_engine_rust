use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::backend::{CreateProject, OpenFolder, WorkspaceCommand};

pub fn ui_menubar(
    mut ctx: EguiContexts,
    mut open_ev: EventWriter<OpenFolder>,
    mut create_ev: EventWriter<CreateProject>,
    mut workspace_ev: EventWriter<WorkspaceCommand>,
) {
    let ctx = ctx.ctx_mut();

    egui::TopBottomPanel::top("menubar")
        .exact_height(28.0)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                ui.add_space(4.0);

                // File menu (project + folder management)
                ui.menu_button("File", |ui| {
                    if ui.button("Create Project…").clicked() {
                        create_ev.send(CreateProject);
                        ui.close_menu();
                    }
                    if ui.button("Open Folder…").clicked() {
                        open_ev.send(OpenFolder::Pick); // <-- now requests a real OS dialog
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        workspace_ev.send(WorkspaceCommand::SaveActive);
                        ui.close_menu();
                    }
                    if ui.button("Close Folder").clicked() {
                        open_ev.send(OpenFolder::Close);
                        ui.close_menu();
                    }
                });

                // (Rest unchanged)
                let mut menu = |title: &str, items: &[(&str, &str)]| {
                    ui.menu_button(title, |ui| {
                        for (label, id) in items {
                            if ui.button(*label).clicked() {
                                println!("[menu] {} -> {}", title, id);
                                ui.close_menu();
                            }
                        }
                    });
                };

                menu(
                    "Edit",
                    &[
                        ("Undo", "edit.undo"),
                        ("Redo", "edit.redo"),
                        ("Cut", "edit.cut"),
                        ("Copy", "edit.copy"),
                        ("Paste", "edit.paste"),
                        ("Find…", "edit.find"),
                    ],
                );

                menu(
                    "Selection",
                    &[
                        ("Select All", "sel.all"),
                        ("Expand Selection", "sel.expand"),
                        ("Shrink Selection", "sel.shrink"),
                    ],
                );

                menu(
                    "View",
                    &[
                        ("Toggle Sidebar", "view.sidebar"),
                        ("Toggle Status Bar", "view.statusbar"),
                        ("Zoom In", "view.zoomin"),
                        ("Zoom Out", "view.zoomout"),
                        ("Reset Zoom", "view.resetzoom"),
                    ],
                );

                menu(
                    "Go",
                    &[
                        ("Go to File…", "go.file"),
                        ("Go to Line…", "go.line"),
                        ("Back", "go.back"),
                        ("Forward", "go.forward"),
                    ],
                );

                menu(
                    "Run",
                    &[
                        ("Start", "run.start"),
                        ("Stop", "run.stop"),
                        ("Restart", "run.restart"),
                    ],
                );

                menu(
                    "Terminal",
                    &[
                        ("New Terminal", "term.new"),
                        ("Split Terminal", "term.split"),
                        ("Kill Terminal", "term.kill"),
                    ],
                );

                menu(
                    "Help",
                    &[
                        ("Welcome", "help.welcome"),
                        ("Docs", "help.docs"),
                        ("About", "help.about"),
                    ],
                );
            });
        });
}
