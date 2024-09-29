use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::{
    diagnostic::LogDiagnosticsPlugin, input::common_conditions::input_just_released, math::DVec3,
    prelude::*, window::WindowResolution,
};
use bevy_avian_baseball_flight::prelude::*;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use blenvy::*;

const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1024.0;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bullpen".to_string(),
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

    app.add_plugins(BlenvyPlugin::default());
    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(NoCameraPlayerPlugin);
    app.add_plugins(BaseballFlightPlugin {
        ssw_on: true,
        magnus_on: true,
        drag_on: true,
    });

    app.add_systems(PostStartup, (setup_scene, spawn_camera.after(setup_scene)));

    app.add_systems(
        Update,
        spawn_ball.run_if(input_just_released(KeyCode::KeyR)),
    );

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("fly cam"),
        FlyCam,
        Camera3dBundle {
            camera: Camera {
                is_active: true,
                order: 0,
                ..default()
            },
            transform: Transform::from_xyz(-0.0, 5.0, -0.)
                .looking_at(Vec3::new(0., 1.2, 0.), Vec3::Y),
            ..default()
        },
    ));
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        BlueprintInfo::from_path("levels/Bullpen.glb"),
        SpawnBlueprint,
        HideUntilReady,
        GameWorldTag,
    ));
}

fn spawn_ball(
    mut commands: Commands,
    mut ev_activate_aerodynamics: EventWriter<ActivateAerodynamicsEvent>,
) {
    let gyro_pole = GyroPole::default();
    let spin_efficiency: f32 = 0.0;
    let spin_rate: f32 = 2400.;
    let velocity: f32 = 96. * MPH_TO_FTS;
    let spin_rate: f32 = 2400.;
    let seam_z_angle: f32 = PI / 2.;
    let tilt = Tilt::from_hour_mintes(12, 0);

    let fixed_spin_rate = if spin_rate == 0. { 1. } else { spin_rate };

    let gyro = match gyro_pole {
        GyroPole::Left => spin_efficiency.asin(),
        GyroPole::Right => PI - spin_efficiency.asin(),
    };

    let spin_x_0 = fixed_spin_rate * (spin_efficiency * tilt.get().sin());
    let spin_y_0 = fixed_spin_rate * gyro.cos(); // ((1. - spin_efficiency.powi(2)).sqrt());
    let spin_z_0 = -fixed_spin_rate * (spin_efficiency * tilt.get().cos());
    let spin = Vec3::new(
        spin_x_0 * RPM_TO_RADS,
        spin_y_0 * RPM_TO_RADS, // - RPM_TO_RAD ???
        spin_z_0 * RPM_TO_RADS,
    );

    let entity = commands
        .spawn((
            Name::new("ball"),
            //
            BaseballFlightBundle::default(),
            //
            ExternalForce::new(Vec3::ZERO),
            Transform::from_translation(Vec3::new(0.48, 1.82, 16.764)),
            LinearVelocity((-Vec3::Y * velocity).from_baseball_coord_to_bevy()),
            AngularVelocity(spin.from_baseball_coord_to_bevy()),
            //
            Restitution {
                coefficient: 0.546,
                combine_rule: CoefficientCombine::Min,
            },
        ))
        .id();

    ev_activate_aerodynamics.send(ActivateAerodynamicsEvent {
        entity,
        seam_y_angle: 0.,
        seam_z_angle,
    });
}
