use bevy::prelude::*;
use bevy_egui::egui;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use super::events::{CreateProject, ExplorerCommand, OpenFolder, OpenMap, WorkspaceCommand};
use super::loader::load_tree_from;
use super::map_parser::{MapData, Theater, blank_map, parse_map, save_mpr};
use super::project::{EditorLayout, ProjectState, scaffold_project};

/// Holds the currently loaded map with all pins we render.
#[derive(Resource, Debug, Clone, Default)]
pub struct MapPreview {
    pub map: Option<MapData>,
}

/// Pan/zoom state for the workspace map view.
#[derive(Resource, Debug, Clone)]
pub struct MapView {
    pub offset: egui::Vec2, // pixels
    pub zoom: f32,          // 1.0 = default
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
    pub selected: Option<(i32, i32)>, // (x, y) in tile coords
}
impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            show_grid: true,
            selected: None,
        }
    }
}

pub fn handle_open_folder(
    mut evr: EventReader<OpenFolder>,
    mut project: ResMut<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    // ▼ Reset workspace when switching/closing folder
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
) {
    for ev in evr.read() {
        match ev {
            OpenFolder::Pick => {
                if let Some(dir) = rfd::FileDialog::new().set_directory(".").pick_folder() {
                    let canonical_dir = dir.canonicalize().unwrap_or(dir.clone());

                    match load_tree_from(&canonical_dir, 4, 5000) {
                        Ok(root) => {
                            let root_id = root.id.clone();
                            project.root = Some(root);
                            project.root_path = Some(canonical_dir.clone());
                            project.clear_workspace();
                            layout.show_explorer = true;
                            layout.open_folders.clear();
                            layout.open_folders.insert(root_id);
                            layout.cancel_rename();
                            layout.clear_pending_close();

                            // Reset workspace view & selection
                            preview.map = None;
                            *view = MapView::default();
                            ws.selected = None;

                            println!("[backend] Opened folder: {}", dir.display());
                        }
                        Err(e) => {
                            project.root = None;
                            project.root_path = None;
                            project.clear_workspace();
                            layout.show_explorer = false;
                            layout.open_folders.clear();
                            layout.cancel_rename();
                            layout.clear_pending_close();

                            preview.map = None;
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
                project.clear_workspace();
                layout.show_explorer = false;
                layout.open_folders.clear();
                layout.cancel_rename();
                layout.clear_pending_close();

                preview.map = None;
                *view = MapView::default();
                ws.selected = None;

                println!("[backend] Closed project; Explorer hidden; workspace reset.");
            }
        }
    }
}

pub fn handle_create_project(
    mut evr: EventReader<CreateProject>,
    mut project: ResMut<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
) {
    for _ in evr.read() {
        if let Some(dir) = rfd::FileDialog::new().set_directory(".").pick_folder() {
            match scaffold_project(&dir) {
                Ok(default_map_path) => {
                    let canonical_dir = dir.canonicalize().unwrap_or(dir.clone());

                    match load_tree_from(&canonical_dir, 4, 5000) {
                        Ok(root) => {
                            let root_id = root.id.clone();
                            project.root = Some(root);
                            project.root_path = Some(canonical_dir.clone());
                            project.clear_workspace();
                            layout.show_explorer = true;
                            layout.open_folders.clear();
                            layout.open_folders.insert(root_id);
                            layout.cancel_rename();
                            layout.clear_pending_close();

                            // Reset workspace state
                            preview.map = None;
                            *view = MapView::default();
                            ws.selected = None;

                            let map_path_str = default_map_path.to_string_lossy().to_string();
                            match parse_map(&map_path_str) {
                                Ok(map) => {
                                    preview.map = Some(map);
                                    project.note_open_map(&default_map_path);
                                    println!(
                                        "[backend] Created project at {} and loaded {}",
                                        canonical_dir.display(),
                                        default_map_path.display()
                                    );
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[backend] Project scaffold created but failed to load map {}: {e}",
                                        default_map_path.display()
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[backend] Created project folder but failed to read tree {}: {e}",
                                canonical_dir.display()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[backend] Failed to scaffold project: {e:?}");
                }
            }
        } else {
            println!("[backend] Create Project canceled by user.");
        }
    }
}

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

fn refresh_project_tree(root: &Path, project: &mut ProjectState) -> anyhow::Result<()> {
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

pub fn handle_workspace_command(
    mut evr: EventReader<WorkspaceCommand>,
    mut project: ResMut<ProjectState>,
    mut layout: ResMut<EditorLayout>,
    mut preview: ResMut<MapPreview>,
    mut view: ResMut<MapView>,
    mut ws: ResMut<WorkspaceSettings>,
    mut open_map_writer: EventWriter<OpenMap>,
) {
    for cmd in evr.read() {
        match cmd {
            WorkspaceCommand::SaveActive => {
                if let Some(active) = project.active_map.clone() {
                    if let Some(map) = preview.map.as_ref() {
                        if let Err(e) = save_mpr(&active, map) {
                            eprintln!("[backend] Failed to save map {}: {e}", active);
                        } else {
                            project.clear_dirty(&active);
                            println!("[backend] Saved map {active}");
                        }
                    } else {
                        eprintln!("[backend] Save requested but no map preview is loaded");
                    }
                }
            }
            WorkspaceCommand::SaveAndClose { path } => {
                let mut saved = false;
                if project.active_map.as_deref() == Some(path) {
                    if let Some(map) = preview.map.as_ref() {
                        match save_mpr(path, map) {
                            Ok(_) => {
                                project.clear_dirty(path);
                                println!("[backend] Saved map {path} before closing");
                                saved = true;
                            }
                            Err(e) => {
                                eprintln!("[backend] Failed to save map {path} before close: {e}");
                            }
                        }
                    } else {
                        eprintln!("[backend] No preview available to save map {path}");
                    }
                } else {
                    // Treat non-active maps as clean for now (no edit tracking yet).
                    project.clear_dirty(path);
                    saved = true;
                }

                if saved {
                    close_map_and_select_next(
                        path,
                        &mut project,
                        &mut preview,
                        &mut view,
                        &mut ws,
                        &mut open_map_writer,
                    );
                }
            }
            WorkspaceCommand::CloseMap { path } => {
                close_map_and_select_next(
                    path,
                    &mut project,
                    &mut preview,
                    &mut view,
                    &mut ws,
                    &mut open_map_writer,
                );
            }
        }
    }

    if let Some(pending) = layout.pending_close.clone() {
        if !project.open_maps.iter().any(|m| m.path == pending.path) {
            layout.clear_pending_close();
        }
    }
}

fn close_map_and_select_next(
    path: &str,
    project: &mut ProjectState,
    preview: &mut MapPreview,
    view: &mut MapView,
    ws: &mut WorkspaceSettings,
    open_map_writer: &mut EventWriter<OpenMap>,
) {
    let was_active = project.active_map.as_deref() == Some(path);
    let next = project.close_map(path);

    if was_active {
        preview.map = None;
        *view = MapView::default();
        ws.selected = None;
    }

    if let Some(next_path) = next {
        open_map_writer.send(OpenMap { path: next_path });
    }
}

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
                // Reset camera & selection so the new map appears centered.
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
