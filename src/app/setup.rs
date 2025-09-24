use bevy::prelude::*;

pub struct CoreWindowPlugin;

impl Plugin for CoreWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RTS Editor (Shell)".into(),
                resolution: (1280., 800.).into(),
                resizable: true,
                resize_constraints: WindowResizeConstraints {
                    min_width: 800.,
                    min_height: 600.,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
