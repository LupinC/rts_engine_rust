use bevy::prelude::*;

mod menubar;
mod workspace;
mod statusbar;
mod explorer;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                menubar::ui_menubar,
                explorer::ui_explorer,
                workspace::ui_workspace,
                statusbar::ui_statusbar,
            ).chain(),
        );
    }
}
