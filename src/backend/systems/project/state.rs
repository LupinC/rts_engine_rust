use anyhow::{Context, Result};
use bevy::prelude::*;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::backend::map::{blank_map, save_mpr};

/// UI layout prefs/state
#[derive(Resource, Default, Debug, Clone)]
pub struct EditorLayout {
    pub show_explorer: bool,
    /// Set of folder IDs that are expanded. We use `Node.id` which is the full path
    /// (string) for stability across frames.
    pub open_folders: HashSet<String>,
    pub rename: Option<RenameSession>,
    pub pending_close: Option<PendingCloseTab>,
}

impl EditorLayout {
    pub fn begin_rename(&mut self, target_path: String, original_name: String) {
        self.rename = Some(RenameSession {
            target_path,
            original_name: original_name.clone(),
            buffer: original_name,
            focus_requested: true,
        });
    }

    pub fn cancel_rename(&mut self) {
        self.rename = None;
    }

    pub fn request_close(&mut self, path: String, name: String, requires_save: bool) {
        self.pending_close = Some(PendingCloseTab {
            path,
            name,
            requires_save,
        });
    }

    pub fn clear_pending_close(&mut self) {
        self.pending_close = None;
    }

    pub fn reset(&mut self) {
        self.show_explorer = false;
        self.open_folders.clear();
        self.cancel_rename();
        self.clear_pending_close();
    }

    pub fn reset_with_root(&mut self, root_id: String) {
        self.show_explorer = true;
        self.open_folders.clear();
        self.open_folders.insert(root_id);
        self.cancel_rename();
        self.clear_pending_close();
    }
}

#[derive(Debug, Clone)]
pub struct RenameSession {
    pub target_path: String,
    pub original_name: String,
    pub buffer: String,
    pub focus_requested: bool,
}

#[derive(Debug, Clone)]
pub struct PendingCloseTab {
    pub path: String,
    pub name: String,
    pub requires_save: bool,
}

#[derive(Debug, Clone)]
pub struct OpenMapEntry {
    pub path: String,
    pub name: String,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ProjectState {
    pub root: Option<Node>,
    pub root_path: Option<PathBuf>,
    pub open_maps: Vec<OpenMapEntry>,
    pub active_map: Option<String>,
    pub dirty_maps: HashSet<String>,
}

impl ProjectState {
    pub fn clear_workspace(&mut self) {
        self.open_maps.clear();
        self.active_map = None;
        self.dirty_maps.clear();
    }

    pub fn reset_all(&mut self) {
        self.root = None;
        self.root_path = None;
        self.clear_workspace();
    }

    pub fn note_open_map<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref();
        let path_str = p.to_string_lossy().to_string();
        let root_clone = self.root_path.clone();
        let display = display_name_for(root_clone.as_ref(), p);

        if let Some(entry) = self.open_maps.iter_mut().find(|m| m.path == path_str) {
            entry.name = display.clone();
        } else {
            self.open_maps.push(OpenMapEntry {
                path: path_str.clone(),
                name: display.clone(),
            });
        }
        self.active_map = Some(path_str.clone());
        self.dirty_maps.remove(&path_str);
    }

    pub fn handle_renamed_path(&mut self, old_path: &Path, new_path: &Path, was_dir: bool) {
        let old_str = old_path.to_string_lossy().to_string();
        let new_str = new_path.to_string_lossy().to_string();
        let root_clone = self.root_path.clone();

        if was_dir {
            let mut dirty_replacements = Vec::new();
            for entry in &mut self.open_maps {
                let entry_path = PathBuf::from(&entry.path);
                if entry_path.starts_with(old_path) {
                    if let Ok(suffix) = entry_path.strip_prefix(old_path) {
                        let new_full = new_path.join(suffix);
                        entry.path = new_full.to_string_lossy().to_string();
                        entry.name = display_name_for(root_clone.as_ref(), &new_full);
                    }
                }
            }

            if let Some(active) = &self.active_map {
                let active_path = PathBuf::from(active);
                if active_path.starts_with(old_path) {
                    if let Ok(suffix) = active_path.strip_prefix(old_path) {
                        let new_full = new_path.join(suffix);
                        self.active_map = Some(new_full.to_string_lossy().to_string());
                    }
                }
            }

            self.dirty_maps.retain(|entry| {
                let entry_path = PathBuf::from(entry);
                if entry_path.starts_with(old_path) {
                    if let Ok(suffix) = entry_path.strip_prefix(old_path) {
                        let new_full = new_path.join(suffix);
                        dirty_replacements.push(new_full.to_string_lossy().to_string());
                    }
                    false
                } else {
                    true
                }
            });

            for rep in dirty_replacements {
                self.dirty_maps.insert(rep);
            }
        } else {
            if let Some(entry) = self.open_maps.iter_mut().find(|m| m.path == old_str) {
                entry.path = new_str.clone();
                entry.name = display_name_for(root_clone.as_ref(), new_path);
            }

            if let Some(active) = &self.active_map {
                if active == &old_str {
                    self.active_map = Some(new_str.clone());
                }
            }

            if self.dirty_maps.remove(&old_str) {
                self.dirty_maps.insert(new_str.clone());
            }
        }
    }

    /// Returns the next map to activate (if current was removed).
    pub fn handle_deleted_path(&mut self, path: &Path, was_dir: bool) -> Option<String> {
        let path_str = path.to_string_lossy().to_string();

        if was_dir {
            self.open_maps.retain(|entry| {
                let entry_path = PathBuf::from(&entry.path);
                !entry_path.starts_with(path)
            });

            if let Some(active) = &self.active_map {
                let active_path = PathBuf::from(active);
                if active_path.starts_with(path) {
                    self.active_map = None;
                }
            }

            self.dirty_maps.retain(|entry| {
                let entry_path = PathBuf::from(entry);
                !entry_path.starts_with(path)
            });
        } else {
            if let Some(pos) = self.open_maps.iter().position(|m| m.path == path_str) {
                self.open_maps.remove(pos);
            }

            if self.active_map.as_deref() == Some(&path_str) {
                self.active_map = None;
            }

            self.dirty_maps.remove(&path_str);
        }

        if self.active_map.is_none() {
            if let Some(next) = self.open_maps.first() {
                self.active_map = Some(next.path.clone());
                Some(next.path.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_dirty<S: AsRef<str>>(&self, path: S) -> bool {
        self.dirty_maps.contains(path.as_ref())
    }

    pub fn clear_dirty<S: AsRef<str>>(&mut self, path: S) {
        self.dirty_maps.remove(path.as_ref());
    }

    /// Remove a map from the open set. Returns the next map that should become active
    /// if the currently active map was closed.
    pub fn close_map<S: AsRef<str>>(&mut self, path: S) -> Option<String> {
        let path_ref = path.as_ref();
        let was_active = self.active_map.as_deref() == Some(path_ref);

        self.open_maps.retain(|entry| entry.path != path_ref);
        self.dirty_maps.remove(path_ref);

        if was_active {
            self.active_map = None;
            if let Some(next) = self.open_maps.first() {
                self.active_map = Some(next.path.clone());
                Some(next.path.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn display_name_for(root: Option<&PathBuf>, path: &Path) -> String {
    if let Some(root) = root {
        if let Ok(stripped) = path.strip_prefix(root) {
            let trimmed = stripped.to_string_lossy();
            let trimmed = trimmed.trim_start_matches(['/', '\\']);
            if !trimmed.is_empty() {
                return trimmed.replace('\\', "/");
            }
        }
    }

    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
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
