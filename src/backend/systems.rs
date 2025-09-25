use bevy::prelude::*;
use bevy_egui::egui;

use super::events::{OpenFolder, OpenMap};
use super::loader::load_tree_from;
use super::map_parser::{parse_map_header, MapHeader, Theater};
use super::project::{EditorLayout, ProjectState};

/// Holds the currently previewed map (if any) for the workspace to render.
#[derive(Resource, Debug, Clone, Default)]
pub struct MapPreview {
    pub header: Option<MapHeader>,
}

/// Pan/zoom state for the workspace map view.
#[derive(Resource, Debug, Clone)]
pub struct MapView {
    pub offset: egui::Vec2, // pixels
    pub zoom: f32,          // 1.0 = default
}
impl Default for MapView {
    fn default() -> Self {
        Self { offset: egui::vec2(0.0, 0.0), zoom: 1.0 }
    }
}

/// Small UI toggles + selection for the workspace overlay.
#[derive(Resource, Debug, Clone)]
pub struct WorkspaceSettings {
    pub show_grid: bool,
    pub selected: Option<(i32, i32)>, // (x, y) in tile coords
}
impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self { show_grid: true, selected: None }
    }
}

pub fn handle_open_folder(
    mut evr: EventReader<OpenFolder>,
    mut project: ResMut<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    // ▼ Added: reset workspace when closing (or opening a new) folder
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
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

                            // Reset workspace view & selection when switching folders
                            preview.header = None;
                            *view = MapView::default();
                            ws.selected = None;

                            println!("[backend] Opened folder: {}", dir.display());
                        }
                        Err(e) => {
                            project.root = None;
                            project.root_path = None;
                            layout.show_explorer = false;
                            layout.open_folders.clear();

                            // Also reset workspace to blank
                            preview.header = None;
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
                layout.show_explorer = false;
                layout.open_folders.clear();

                // ▼ Ensure the workspace returns to its original blank view
                preview.header = None;
                *view = MapView::default();
                ws.selected = None;

                println!("[backend] Closed project; Explorer hidden; workspace reset.");
            }
        }
    }
}

pub fn handle_open_map(
    mut evr: EventReader<OpenMap>,
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
) {
    for ev in evr.read() {
        let is_map = ev.path.to_ascii_lowercase().ends_with(".map");
        if !is_map {
            continue;
        }
        match parse_map_header(&ev.path) {
            Ok(h) => {
                preview.header = Some(h);
                // Reset camera & selection so the new map appears centered.
                *view = MapView::default();
                ws.selected = None;
                println!("[backend] Loaded map header from {}", ev.path);
            }
            Err(e) => {
                preview.header = None;
                eprintln!("[backend] Failed to parse map {}: {e}", ev.path);
            }
        }
    }
}

/// Theater → base color for preview fill.
pub fn theater_color(theater: Theater) -> egui::Color32 {
    use egui::Color32;
    match theater {
        Theater::Temperate => Color32::from_rgb(70, 104, 68),
        Theater::Snow      => Color32::from_rgb(220, 232, 240),
        Theater::Urban     => Color32::from_rgb(95, 95, 102),
        Theater::NewUrban  => Color32::from_rgb(72, 78, 86),
        Theater::Desert    => Color32::from_rgb(204, 170, 102),
        Theater::Lunar     => Color32::from_rgb(180, 180, 190),
        Theater::Unknown   => Color32::from_rgb(120, 120, 130),
    }
}
