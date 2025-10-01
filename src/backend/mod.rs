use bevy::prelude::*;

mod editor_objects;
mod events;
mod loader;
mod map_parser;
mod systems; // keep as a private module

pub use events::{CreateProject, ExplorerCommand, OpenFolder, OpenMap, WorkspaceCommand};
pub use systems::project::{EditorLayout, Node, NodeKind, ProjectState};
pub use systems::{MapPreview, MapView, WorkspaceSettings, theater_color};

// Re-export editor objects (including palette API) so ui can `use crate::backend::{...}`
pub use editor_objects::{
    EditorObjects, PaletteState, PaletteTab, Tool, ToolState, palette_entries,
};
pub struct BackendPlugin;

impl Plugin for BackendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectState>()
            .init_resource::<EditorLayout>()
            .init_resource::<editor_objects::PaletteState>()
            .init_resource::<MapPreview>()
            .init_resource::<MapView>()
            .init_resource::<WorkspaceSettings>()
            .init_resource::<ToolState>()
            .init_resource::<EditorObjects>()
            .add_event::<events::CreateProject>()
            .add_event::<events::ExplorerCommand>()
            .add_event::<events::OpenFolder>()
            .add_event::<events::OpenMap>()
            .add_event::<events::WorkspaceCommand>()
            .add_systems(
                Update,
                (
                    systems::handle_create_project,
                    systems::handle_open_folder,
                    systems::handle_explorer_command,
                    systems::handle_workspace_command,
                    systems::handle_open_map,
                ),
            );
    }
}
