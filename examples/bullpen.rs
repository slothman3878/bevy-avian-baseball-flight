use avian3d::prelude::*;
use bevy::{diagnostic::LogDiagnosticsPlugin, prelude::*, window::WindowResolution};
use bevy_avian_baseball_flight::{prelude::*, BaseballFlightPlugin};

const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1024.0;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Rpg - Baseball".to_string(),
            resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resizable: false,
            ..Default::default()
        }),
        ..Default::default()
    }));
    #[cfg(debug_assertions)]
    {
        app.add_plugins(LogDiagnosticsPlugin::default());
        app.add_plugins(PhysicsDebugPlugin::default());
    }
}
