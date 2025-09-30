use bevy::prelude::*;

mod explorer;
mod interact;
mod menubar;
mod statusbar;
mod workspace;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                menubar::ui_menubar,     // top
                explorer::ui_explorer,   // left
                interact::ui_interact,   // bottom (your red area)
                statusbar::ui_statusbar, // bottom status line
                workspace::ui_workspace, // central (must be last)
            )
                .chain(),
        );
    }
}
