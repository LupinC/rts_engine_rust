use anyhow::{Context, Result};
use bevy::prelude::*;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use super::map_parser::{blank_map, save_mpr};

/// UI layout prefs/state
#[derive(Resource, Default, Debug, Clone)]
pub struct EditorLayout {
    pub show_explorer: bool,
    /// Set of folder IDs that are expanded. We use `Node.id` which is the full path
    /// (string) for stability across frames.
    pub open_folders: HashSet<String>,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ProjectState {
    pub root: Option<Node>,
    pub root_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Folder { children: Vec<Node> },
    File { path: String, ext: String },
}

#[derive(Debug, Clone)]
pub struct Node {
    /// Stable unique id (we use the absolute/normalized path as string)
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
}

/// Ensure the selected directory contains the expected project scaffold and return the default map path.
pub fn scaffold_project(root: &Path) -> Result<PathBuf> {
    if !root.exists() {
        fs::create_dir_all(root)
            .with_context(|| format!("Failed to create project directory {}", root.display()))?;
    }

    let maps_dir = root.join("maps");
    fs::create_dir_all(&maps_dir)
        .with_context(|| format!("Failed to create maps directory {}", maps_dir.display()))?;

    let map_path = maps_dir.join("main.mpr");
    if !map_path.exists() {
        let map = blank_map(64, 64);
        save_mpr(&map_path, &map)
            .with_context(|| format!("Failed to write default map {}", map_path.display()))?;
    }

    Ok(map_path)
}
