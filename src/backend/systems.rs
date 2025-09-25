use bevy::prelude::*;
use bevy_egui::egui; // <-- bring in egui for Color32

use super::events::{OpenFolder, OpenMap};
use super::loader::load_tree_from;
use super::map_parser::{parse_map_header, MapHeader, Theater};
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

/// Holds the currently previewed map (if any) for the workspace to render.
#[derive(Resource, Debug, Clone, Default)]
pub struct MapPreview {
    pub header: Option<MapHeader>,
}

pub fn handle_open_map(
    mut evr: EventReader<OpenMap>,
    mut preview: ResMut<MapPreview>,
) {
    for ev in evr.read() {
        // Only accept .map (case-insensitive)
        let is_map = ev.path.to_ascii_lowercase().ends_with(".map");
        if !is_map {
            // Spec says: non-.map does nothing.
            continue;
        }
        match parse_map_header(&ev.path) {
            Ok(h) => {
                preview.header = Some(h);
                println!("[backend] Loaded map header from {}", ev.path);
            }
            Err(e) => {
                preview.header = None;
                eprintln!("[backend] Failed to parse map {}: {e}", ev.path);
            }
        }
    }
}

/// Helper used by the workspace to pick a color per theater (no textures yet).
pub fn theater_color(theater: Theater) -> egui::Color32 {
    use egui::Color32;
    match theater {
        Theater::Temperate => Color32::from_rgb(70, 104, 68),   // greenish
        Theater::Snow      => Color32::from_rgb(220, 232, 240), // icy white
        Theater::Urban     => Color32::from_rgb(95, 95, 102),   // gray
        Theater::NewUrban  => Color32::from_rgb(72, 78, 86),    // darker gray
        Theater::Desert    => Color32::from_rgb(204, 170, 102), // sand
        Theater::Lunar     => Color32::from_rgb(180, 180, 190), // pale
        Theater::Unknown   => Color32::from_rgb(120, 120, 130), // neutral
    }
}
