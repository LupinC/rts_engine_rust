use bevy::prelude::*;

/// Placeholder for services/state (file IO, project model, commands, etc.)
pub struct BackendPlugin;

impl Plugin for BackendPlugin {
    fn build(&self, _app: &mut App) {
        // Add resources, events, and systems for data/model here later.
        // e.g., ProjectState, Command bus, FileSystemService, etc.
    }
}
