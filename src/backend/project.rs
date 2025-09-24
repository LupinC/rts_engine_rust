use bevy::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind { File, Folder }

#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,      // unique id (path-like)
    pub name: String,    // display name
    pub kind: NodeKind,
    pub children: Vec<Node>,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ProjectState {
    pub root: Option<Node>,
    pub root_path: Option<PathBuf>,
}

/// UI layout state (Explorer visibility + which folders are expanded)
#[derive(Resource, Debug, Clone)]
pub struct EditorLayout {
    pub show_explorer: bool,
    pub open_folders: HashSet<String>,
}

impl Default for EditorLayout {
    fn default() -> Self {
        Self {
            show_explorer: false,
            open_folders: HashSet::new(),
        }
    }
}
