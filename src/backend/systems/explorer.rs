use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::Result;
use bevy::prelude::*;

use super::super::events::{ExplorerCommand, OpenMap};
use super::super::loader::load_tree_from;
use super::super::map_parser::{blank_map, save_mpr};
use super::super::project::{EditorLayout, ProjectState};
use super::resources::{MapPreview, MapView, WorkspaceSettings};

pub fn handle_explorer_command(
    mut evr: EventReader<ExplorerCommand>,
    mut project: ResMut<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    mut open_map_writer: EventWriter<OpenMap>,
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
) {
    for cmd in evr.read() {
        let Some(root_path) = project.root_path.clone() else {
            continue;
        };

        match cmd {
            ExplorerCommand::NewFile { parent } => {
                let parent_path = PathBuf::from(parent);
                if !parent_path.is_dir() {
                    eprintln!(
                        "[backend] Explorer NewFile target is not a directory: {}",
                        parent
                    );
                    continue;
                }
                if !parent_path.starts_with(&root_path) {
                    eprintln!(
                        "[backend] Explorer NewFile outside project root: {}",
                        parent
                    );
                    continue;
                }

                let file_path = unique_map_path(&parent_path);
                if let Err(e) = save_mpr(&file_path, &blank_map(64, 64)) {
                    eprintln!(
                        "[backend] Failed to create new map {}: {e}",
                        file_path.display()
                    );
                    continue;
                }

                if let Err(e) = refresh_project_tree(&root_path, project.as_mut()) {
                    eprintln!("[backend] Map created but failed to refresh explorer: {e}");
                }

                layout.open_folders.insert(parent.clone());
                open_map_writer.send(OpenMap {
                    path: file_path.to_string_lossy().to_string(),
                });
            }
            ExplorerCommand::NewFolder { parent } => {
                let parent_path = PathBuf::from(parent);
                if !parent_path.is_dir() {
                    eprintln!(
                        "[backend] Explorer NewFolder target is not a directory: {}",
                        parent
                    );
                    continue;
                }
                if !parent_path.starts_with(&root_path) {
                    eprintln!(
                        "[backend] Explorer NewFolder outside project root: {}",
                        parent
                    );
                    continue;
                }

                let folder_path = unique_folder_path(&parent_path);
                if let Err(e) = fs::create_dir(&folder_path) {
                    eprintln!(
                        "[backend] Failed to create folder {}: {e}",
                        folder_path.display()
                    );
                    continue;
                }

                if let Err(e) = refresh_project_tree(&root_path, project.as_mut()) {
                    eprintln!("[backend] Folder created but failed to refresh explorer: {e}");
                }

                layout.open_folders.insert(parent.clone());
                layout.open_folders.insert(
                    folder_path
                        .canonicalize()
                        .unwrap_or(folder_path.clone())
                        .to_string_lossy()
                        .to_string(),
                );
            }
            ExplorerCommand::RenameEntry { from, new_name } => {
                let target_path = PathBuf::from(from);
                if !target_path.exists() {
                    eprintln!("[backend] Explorer Rename target missing: {}", from);
                    continue;
                }
                if !target_path.starts_with(&root_path) {
                    eprintln!("[backend] Explorer Rename outside project root: {}", from);
                    continue;
                }

                let trimmed = new_name.trim();
                if trimmed.is_empty() {
                    eprintln!("[backend] Explorer Rename requires a non-empty name");
                    continue;
                }
                if !is_simple_name(trimmed) {
                    eprintln!("[backend] Explorer Rename rejected invalid name: {trimmed}");
                    continue;
                }

                let was_dir = target_path.is_dir();
                let old_canonical = target_path.canonicalize().unwrap_or(target_path.clone());
                let active_before = project.active_map.clone();

                let parent_dir = target_path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or(root_path.clone());
                let new_path = parent_dir.join(trimmed);

                if new_path == target_path {
                    continue;
                }
                if new_path.exists() {
                    eprintln!(
                        "[backend] Explorer Rename destination already exists: {}",
                        new_path.display()
                    );
                    continue;
                }
                if !new_path.starts_with(&root_path) {
                    eprintln!(
                        "[backend] Explorer Rename must stay within project root: {}",
                        new_path.display()
                    );
                    continue;
                }
                if let Err(e) = fs::rename(&target_path, &new_path) {
                    eprintln!(
                        "[backend] Failed to rename entry {} -> {}: {e}",
                        target_path.display(),
                        new_path.display()
                    );
                    continue;
                }

                let new_canonical = new_path.canonicalize().unwrap_or(new_path.clone());

                if was_dir {
                    let mut replacements = Vec::new();
                    layout.open_folders.retain(|entry| {
                        let entry_path = PathBuf::from(entry);
                        if entry_path.starts_with(&old_canonical) {
                            if let Ok(suffix) = entry_path.strip_prefix(&old_canonical) {
                                let new_full = new_canonical.join(suffix);
                                replacements.push(new_full.to_string_lossy().to_string());
                            }
                            false
                        } else {
                            true
                        }
                    });
                    for rep in replacements {
                        layout.open_folders.insert(rep);
                    }
                }

                project.handle_renamed_path(&target_path, &new_path, was_dir);

                if let Err(e) = refresh_project_tree(&root_path, project.as_mut()) {
                    eprintln!("[backend] Entry renamed but failed to refresh explorer: {e}");
                }

                if let Some(before) = active_before {
                    if project.active_map.as_ref() != Some(&before) {
                        if let Some(ref new_active) = project.active_map {
                            open_map_writer.send(OpenMap {
                                path: new_active.clone(),
                            });
                        } else {
                            preview.map = None;
                            *view = MapView::default();
                            ws.selected = None;
                        }
                    }
                }
            }
            ExplorerCommand::DeleteEntry { path } => {
                let target_path = PathBuf::from(path);
                if !target_path.exists() {
                    eprintln!("[backend] Explorer Delete target missing: {}", path);
                    continue;
                }
                if !target_path.starts_with(&root_path) {
                    eprintln!("[backend] Explorer Delete outside project root: {}", path);
                    continue;
                }

                let canonical_target = target_path.canonicalize().unwrap_or(target_path.clone());
                let was_dir = target_path.is_dir();

                if target_path.is_file() {
                    if let Err(e) = fs::remove_file(&target_path) {
                        eprintln!(
                            "[backend] Failed to delete file {}: {e}",
                            target_path.display()
                        );
                        continue;
                    }
                } else if was_dir {
                    if let Err(e) = fs::remove_dir_all(&target_path) {
                        eprintln!(
                            "[backend] Failed to delete folder {}: {e}",
                            target_path.display()
                        );
                        continue;
                    }
                } else {
                    eprintln!(
                        "[backend] Explorer Delete unsupported target type: {}",
                        path
                    );
                    continue;
                }

                layout.open_folders.retain(|entry| {
                    let entry_path = PathBuf::from(entry);
                    !entry_path.starts_with(&canonical_target)
                });

                let active_before = project.active_map.clone();
                let next_path = project.handle_deleted_path(&target_path, was_dir);

                if let Err(e) = refresh_project_tree(&root_path, project.as_mut()) {
                    eprintln!("[backend] Entry deleted but failed to refresh explorer: {e}");
                }

                if let Some(next) = next_path {
                    open_map_writer.send(OpenMap { path: next });
                } else if active_before.is_some() {
                    preview.map = None;
                    *view = MapView::default();
                    ws.selected = None;
                }
            }
        }
    }
}

fn refresh_project_tree(root: &Path, project: &mut ProjectState) -> Result<()> {
    match load_tree_from(root, 4, 5000) {
        Ok(node) => {
            project.root = Some(node);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn unique_map_path(parent: &Path) -> PathBuf {
    let mut index = 0;
    loop {
        let name = if index == 0 {
            "untitled.mpr".to_string()
        } else {
            format!("untitled-{index}.mpr")
        };
        let candidate = parent.join(&name);
        if !candidate.exists() {
            return candidate;
        }
        index += 1;
    }
}

fn unique_folder_path(parent: &Path) -> PathBuf {
    let mut index = 0;
    loop {
        let name = if index == 0 {
            "New Folder".to_string()
        } else {
            format!("New Folder {index}")
        };
        let candidate = parent.join(&name);
        if !candidate.exists() {
            return candidate;
        }
        index += 1;
    }
}

fn is_simple_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let candidate = Path::new(name);
    candidate
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
}
