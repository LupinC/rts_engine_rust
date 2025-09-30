use bevy::prelude::*;

use super::super::events::CreateProject;
use super::super::events::OpenFolder;
use super::super::loader::load_tree_from;
use super::super::map_parser::parse_map;
use super::super::project::{EditorLayout, ProjectState, scaffold_project};
use super::resources::{MapPreview, MapView, WorkspaceSettings};

pub fn handle_open_folder(
    mut evr: EventReader<OpenFolder>,
    mut project: ResMut<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
) {
    for ev in evr.read() {
        match ev {
            OpenFolder::Pick => {
                if let Some(dir) = rfd::FileDialog::new().set_directory(".").pick_folder() {
                    let canonical_dir = dir.canonicalize().unwrap_or(dir.clone());

                    match load_tree_from(&canonical_dir, 4, 5000) {
                        Ok(root) => {
                            let root_id = root.id.clone();
                            project.root = Some(root);
                            project.root_path = Some(canonical_dir.clone());
                            project.clear_workspace();
                            layout.show_explorer = true;
                            layout.open_folders.clear();
                            layout.open_folders.insert(root_id);
                            layout.cancel_rename();
                            layout.clear_pending_close();

                            preview.map = None;
                            *view = MapView::default();
                            ws.selected = None;

                            println!("[backend] Opened folder: {}", dir.display());
                        }
                        Err(e) => {
                            project.root = None;
                            project.root_path = None;
                            project.clear_workspace();
                            layout.show_explorer = false;
                            layout.open_folders.clear();
                            layout.cancel_rename();
                            layout.clear_pending_close();

                            preview.map = None;
                            *view = MapView::default();
                            ws.selected = None;

                            eprintln!("[backend] Failed to open folder: {e}");
                        }
                    }
                } else {
                    println!("[backend] Open Folder canceled by user.");
                }
            }
            OpenFolder::Close => {
                project.root = None;
                project.root_path = None;
                project.clear_workspace();
                layout.show_explorer = false;
                layout.open_folders.clear();
                layout.cancel_rename();
                layout.clear_pending_close();

                preview.map = None;
                *view = MapView::default();
                ws.selected = None;

                println!("[backend] Closed project; Explorer hidden; workspace reset.");
            }
        }
    }
}

pub fn handle_create_project(
    mut evr: EventReader<CreateProject>,
    mut project: ResMut<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
) {
    for _ in evr.read() {
        if let Some(dir) = rfd::FileDialog::new().set_directory(".").pick_folder() {
            match scaffold_project(&dir) {
                Ok(default_map_path) => {
                    let canonical_dir = dir.canonicalize().unwrap_or(dir.clone());

                    match load_tree_from(&canonical_dir, 4, 5000) {
                        Ok(root) => {
                            let root_id = root.id.clone();
                            project.root = Some(root);
                            project.root_path = Some(canonical_dir.clone());
                            project.clear_workspace();
                            layout.show_explorer = true;
                            layout.open_folders.clear();
                            layout.open_folders.insert(root_id);
                            layout.cancel_rename();
                            layout.clear_pending_close();

                            preview.map = None;
                            *view = MapView::default();
                            ws.selected = None;

                            let map_path_str = default_map_path.to_string_lossy().to_string();
                            match parse_map(&map_path_str) {
                                Ok(map) => {
                                    preview.map = Some(map);
                                    project.note_open_map(&default_map_path);
                                    println!(
                                        "[backend] Created project at {} and loaded {}",
                                        canonical_dir.display(),
                                        default_map_path.display()
                                    );
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[backend] Project scaffold created but failed to load map {}: {e}",
                                        default_map_path.display()
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[backend] Created project folder but failed to read tree {}: {e}",
                                canonical_dir.display()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[backend] Failed to scaffold project: {e:?}");
                }
            }
        } else {
            println!("[backend] Create Project canceled by user.");
        }
    }
}
