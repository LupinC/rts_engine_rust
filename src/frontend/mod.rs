use bevy::prelude::*;

/// Placeholder for “view” side (panels, editors, scene viewport…)
pub struct FrontendPlugin;

impl Plugin for FrontendPlugin {
    fn build(&self, _app: &mut App) {
        // Add UI panels/systems here later
        // e.g., Explorer, Tab bar, Text editor, Scene view, etc.
    }
}
