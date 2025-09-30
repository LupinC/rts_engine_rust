use bevy::prelude::{EventWriter, Res, ResMut};
use bevy_egui::{EguiContexts, egui};

use crate::backend::{
    EditorLayout, EditorObjects, MapPreview, MapView, OpenMap, ProjectState, ToolState,
    WorkspaceCommand, WorkspaceSettings,
};

use super::{canvas, close_dialog, tabs};

pub fn ui_workspace(
    mut ctx: EguiContexts,
    mut layout: ResMut<EditorLayout>,
    preview: Res<MapPreview>,
    mut view: ResMut<MapView>,
    mut settings: ResMut<WorkspaceSettings>,
    tool: ResMut<ToolState>,
    objs: ResMut<EditorObjects>,
    project: Res<ProjectState>,
    mut open_map_writer: EventWriter<OpenMap>,
    mut workspace_writer: EventWriter<WorkspaceCommand>,
) {
    let egui_ctx = ctx.ctx_mut();

    if egui_ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.command) {
        workspace_writer.send(WorkspaceCommand::SaveActive);
    }

    {
        let layout = layout.as_mut();
        tabs::show_tabs(
            egui_ctx,
            layout,
            project.as_ref(),
            &mut open_map_writer,
            &mut workspace_writer,
        );
    }

    {
        let layout = layout.as_mut();
        close_dialog::handle_pending_close(egui_ctx, layout, &mut workspace_writer);
    }

    canvas::render_canvas(
        egui_ctx,
        preview.as_ref(),
        view.as_mut(),
        settings.as_mut(),
        tool.current,
        objs.as_ref(),
    );
}
