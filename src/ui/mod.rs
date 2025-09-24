use bevy::prelude::*;

mod menubar;
mod statusbar;
mod workspace;

/// Bundles all UI pieces into a single plugin for convenience
pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                menubar::ui_menubar,
                workspace::ui_workspace,
                statusbar::ui_statusbar,
            ),
        );
    }
}
