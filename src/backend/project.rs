use bevy::prelude::*;
use std::collections::HashSet;

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
