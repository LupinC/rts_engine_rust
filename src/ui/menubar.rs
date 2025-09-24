use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn ui_menubar(mut ctx: EguiContexts) {
    let ctx = ctx.ctx_mut();

    egui::TopBottomPanel::top("menubar").exact_height(28.0).show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.add_space(4.0);

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
                "File",
                &[
                    ("New File…", "file.new"),
                    ("Open…", "file.open"),
                    ("Save", "file.save"),
                    ("Save As…", "file.save_as"),
                    ("Close", "file.close"),
                    ("Exit", "file.exit"),
                ],
            );
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
