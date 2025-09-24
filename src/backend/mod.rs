use bevy::prelude::*;

mod events;
mod project;
mod loader;
mod systems;

pub use events::OpenFolder;
pub use project::{EditorLayout, Node, NodeKind, ProjectState};

pub struct BackendPlugin;

impl Plugin for BackendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<project::ProjectState>()
            .init_resource::<project::EditorLayout>()
            .add_event::<events::OpenFolder>()
            .add_systems(Update, systems::handle_open_folder);
    }
}
