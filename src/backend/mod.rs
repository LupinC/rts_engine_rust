use bevy::prelude::*;

mod events;
mod project;
mod loader;
mod systems;
mod map_parser;

pub use events::{OpenFolder, OpenMap};
pub use project::{EditorLayout, Node, NodeKind, ProjectState};
pub use systems::{MapPreview, theater_color}; // <-- re-export both

pub struct BackendPlugin;

impl Plugin for BackendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<project::ProjectState>()
            .init_resource::<project::EditorLayout>()
            .init_resource::<MapPreview>() // holds current previewed map (if any)
            .add_event::<events::OpenFolder>()
            .add_event::<events::OpenMap>()
            .add_systems(Update, (systems::handle_open_folder, systems::handle_open_map));
    }
}
