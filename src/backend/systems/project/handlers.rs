use bevy::prelude::*;

use crate::backend::events::{CreateProject, OpenFolder};
use crate::backend::loader::load_tree_from;
use crate::backend::map_parser::parse_map;

use super::super::resources::{MapPreview, MapView, WorkspaceSettings};
use super::state::{EditorLayout, ProjectState};

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
                            layout.reset_with_root(root_id);

                            preview.map = None;
                            *view = MapView::default();
                            ws.selected = None;

                            println!("[backend] Opened folder: {}", dir.display());
                        }
                        Err(e) => {
                            project.reset_all();
                            layout.reset();
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
                project.reset_all();
                layout.reset();
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
            match super::state::scaffold_project(&dir) {
                Ok(default_map_path) => {
                    let canonical_dir = dir.canonicalize().unwrap_or(dir.clone());

                    match load_tree_from(&canonical_dir, 4, 5000) {
                        Ok(root) => {
                            let root_id = root.id.clone();
                            project.root = Some(root);
                            project.root_path = Some(canonical_dir.clone());
                            project.clear_workspace();
                            layout.reset_with_root(root_id);

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
