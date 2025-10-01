use bevy::prelude::*;

use super::project::{EditorLayout, ProjectState};
use super::resources::{MapPreview, MapView, WorkspaceSettings};
use crate::backend::events::{OpenMap, WorkspaceCommand};
use crate::backend::map::save_mpr;

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
