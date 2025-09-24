use bevy::prelude::*;

use super::events::OpenFolder;
use super::loader::load_tree_from;
use super::project::{EditorLayout, ProjectState};

pub fn handle_open_folder(
    mut evr: EventReader<OpenFolder>,
    mut project: ResMut<ProjectState>,
    mut layout: ResMut<EditorLayout>,
) {
    for ev in evr.read() {
        match ev {
            OpenFolder::Pick => {
                if let Some(dir) = rfd::FileDialog::new().set_directory(".").pick_folder() {
                    match load_tree_from(&dir, 4, 5000) {
                        Ok(root) => {
                            let root_id = root.id.clone();

                            project.root = Some(root);
                            project.root_path = Some(dir.clone());
                            layout.show_explorer = true;

                            // Reset and expand the root folder by default
                            layout.open_folders.clear();
                            layout.open_folders.insert(root_id);

                            println!("[backend] Opened folder: {}", dir.display());
                        }
                        Err(e) => {
                            project.root = None;
                            project.root_path = None;
                            layout.show_explorer = false;
                            layout.open_folders.clear();
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
                layout.show_explorer = false;
                layout.open_folders.clear();
                println!("[backend] Closed project; Explorer hidden.");
            }
        }
    }
}
