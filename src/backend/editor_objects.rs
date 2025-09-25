use bevy::prelude::*;
use bevy_egui::egui;

// ----- Tools / placements -----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Spawn,
    Resource,
    Unit,
}

#[derive(Resource, Debug, Clone)]
pub struct ToolState {
    pub current: Tool,
}
impl Default for ToolState {
    fn default() -> Self {
        Self { current: Tool::Select }
    }
}

#[derive(Debug, Clone)]
pub struct Placement {
    pub kind: Tool,
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct EditorObjects {
    pub items: Vec<Placement>,
}

pub fn kind_color(kind: Tool) -> egui::Color32 {
    match kind {
        Tool::Select   => egui::Color32::from_rgb(200, 200, 220),
        Tool::Spawn    => egui::Color32::from_rgb( 60, 220, 120),
        Tool::Resource => egui::Color32::from_rgb(245, 210,  60),
        Tool::Unit     => egui::Color32::from_rgb( 60, 200, 245),
    }
}

// ----- Palette (tabs + entries) -----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteTab {
    Resource,
    SpawnPoints,
    // add Units, Structures, etc. later
}

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub name: &'static str,
    pub emoji: &'static str,
    pub color: egui::Color32,
}

#[derive(Resource, Debug, Clone)]
pub struct PaletteState {
    pub tab: PaletteTab,
    pub selected_idx: Option<usize>,
}

impl Default for PaletteState {
    fn default() -> Self {
        Self { tab: PaletteTab::Resource, selected_idx: None }
    }
}

// Return an owned Vec to avoid borrowing a temporary slice (fixes E0515).
pub fn palette_entries(tab: PaletteTab) -> Vec<PaletteEntry> {
    match tab {
        PaletteTab::Resource => vec![
            PaletteEntry { name: "Ore",   emoji: "🧱", color: egui::Color32::from_rgb(230, 70, 70) },
            PaletteEntry { name: "Gem",   emoji: "💎", color: egui::Color32::from_rgb(245, 220, 80) },
            PaletteEntry { name: "Oil",   emoji: "🛢️", color: egui::Color32::from_rgb(70, 200, 110) },
        ],
        PaletteTab::SpawnPoints => vec![
            PaletteEntry { name: "Player 1", emoji: "①", color: egui::Color32::from_rgb( 90, 180, 255) },
            PaletteEntry { name: "Player 2", emoji: "②", color: egui::Color32::from_rgb(255, 150, 120) },
            PaletteEntry { name: "Waypoint", emoji: "⭐", color: egui::Color32::from_rgb(170, 170, 255) },
        ],
    }
}
