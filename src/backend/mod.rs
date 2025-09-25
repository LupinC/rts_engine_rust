use bevy::prelude::*;

mod events;
mod project;
mod loader;
mod systems;
mod map_parser;
mod editor_objects; // keep as a private module

pub use events::{OpenFolder, OpenMap};
pub use project::{EditorLayout, Node, NodeKind, ProjectState};
pub use systems::{MapPreview, MapView, WorkspaceSettings, theater_color};

// Re-export editor objects (including palette API) so ui can `use crate::backend::{...}`
pub use editor_objects::{
    Tool, ToolState, EditorObjects, Placement,
    PaletteTab, PaletteState, palette_entries, // <-- added
};

pub struct BackendPlugin;

impl Plugin for BackendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<project::ProjectState>()
            .init_resource::<project::EditorLayout>()
            .init_resource::<editor_objects::PaletteState>()
            .init_resource::<MapPreview>()
            .init_resource::<MapView>()
            .init_resource::<WorkspaceSettings>()
            .init_resource::<ToolState>()
            .init_resource::<EditorObjects>()
            .add_event::<events::OpenFolder>()
            .add_event::<events::OpenMap>()
            .add_systems(Update, (systems::handle_open_folder, systems::handle_open_map));
    }
}
