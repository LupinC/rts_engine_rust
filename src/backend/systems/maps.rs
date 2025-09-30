use std::path::Path;

use bevy::prelude::*;
use bevy_egui::egui;

use super::super::events::OpenMap;
use super::super::map_parser::{Theater, parse_map};
use super::super::project::ProjectState;
use super::resources::{MapPreview, MapView, WorkspaceSettings};

pub fn handle_open_map(
    mut evr: EventReader<OpenMap>,
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
    mut project: ResMut<ProjectState>,
) {
    for ev in evr.read() {
        let lower = ev.path.to_ascii_lowercase();
        if !(lower.ends_with(".map") || lower.ends_with(".mpr")) {
            continue;
        }
        match parse_map(&ev.path) {
            Ok(m) => {
                preview.map = Some(m);
                *view = MapView::default();
                ws.selected = None;
                project.note_open_map(Path::new(&ev.path));
                println!("[backend] Loaded map {}", ev.path);
            }
            Err(e) => {
                preview.map = None;
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
        Theater::Snow => Color32::from_rgb(220, 232, 240),
        Theater::Urban => Color32::from_rgb(95, 95, 102),
        Theater::NewUrban => Color32::from_rgb(72, 78, 86),
        Theater::Desert => Color32::from_rgb(204, 170, 102),
        Theater::Lunar => Color32::from_rgb(180, 180, 190),
        Theater::Unknown => Color32::from_rgb(120, 120, 130),
    }
}
