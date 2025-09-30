use bevy::prelude::*;
use bevy_egui::egui;

use super::super::map_parser::MapData;

/// Holds the currently loaded map with all pins we render.
#[derive(Resource, Debug, Clone, Default)]
pub struct MapPreview {
    pub map: Option<MapData>,
}

/// Pan/zoom state for the workspace map view.
#[derive(Resource, Debug, Clone)]
pub struct MapView {
    pub offset: egui::Vec2,
    pub zoom: f32,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            offset: egui::vec2(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

/// Small UI toggles + selection for the workspace overlay.
#[derive(Resource, Debug, Clone)]
pub struct WorkspaceSettings {
    pub show_grid: bool,
    pub selected: Option<(i32, i32)>,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            show_grid: true,
            selected: None,
        }
    }
}
