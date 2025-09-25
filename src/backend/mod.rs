use bevy::prelude::*;

mod events;
mod project;
mod loader;
mod systems;
mod map_parser;

pub use events::{OpenFolder, OpenMap};
pub use project::{EditorLayout, Node, NodeKind, ProjectState};
pub use systems::{MapPreview, MapView, WorkspaceSettings, theater_color}; // ← re-export settings too

pub struct BackendPlugin;

impl Plugin for BackendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<project::ProjectState>()
            .init_resource::<project::EditorLayout>()
            .init_resource::<MapPreview>()      // current previewed map (if any)
            .init_resource::<MapView>()         // pan/zoom state
            .init_resource::<WorkspaceSettings>() // ← UI toggles & selection
            .add_event::<events::OpenFolder>()
            .add_event::<events::OpenMap>()
            .add_systems(Update, (systems::handle_open_folder, systems::handle_open_map));
    }
}
