use crate::*;

// option 1 - update transform
pub(crate) fn _apply_physics_option_1(
    time_fixed: Res<Time<Fixed>>,
    time_physics: Res<Time<Physics>>,
    baseball_plugin_config: Res<BaseballPluginConfig>,
    mut query_baseball: Query<(&mut BaseballFlightState, &mut Transform)>,
) {
    // 0.0167
    let delta_t = time_fixed.delta_seconds_f64() * time_physics.relative_speed_f64();
    for (mut state, mut transform) in &mut query_baseball {
        state.update_state(&baseball_plugin_config, delta_t);
        transform.translation = state.translation.as_vec3().from_baseball_coord_to_bevy();
    }
}

// option 2 - update velocity
pub(crate) fn _apply_physics_option_2(
    time_fixed: Res<Time<Fixed>>,
    time_physics: Res<Time<Physics>>,
    baseball_plugin_config: Res<BaseballPluginConfig>,
    mut query_baseball: Query<(
        &mut BaseballFlightState,
        &Transform,
        &mut LinearVelocity,
        &mut GravityScale,
    )>,
) {
    // 0.0167
    let delta_t = time_fixed.delta_seconds_f64() * time_physics.relative_speed_f64();
    for (mut state, transform, mut l_velo, mut gravity_scale) in &mut query_baseball {
        if state.active {
            let new_velo = state.update_state_and_get_velo(
                &baseball_plugin_config,
                transform
                    .translation
                    .from_bevy_to_baseball_coord()
                    .as_dvec3(),
                delta_t,
            );
            l_velo.0 = new_velo.from_baseball_coord_to_bevy().as_vec3();
        } else {
            gravity_scale.0 = 1.;
        }
    }
}

// preferred
// option 3 - apply external force
pub(crate) fn _apply_physics_option_3(
    time_fixed: Res<Time<Fixed>>,
    time_physics: Res<Time<Physics>>,
    baseball_plugin_config: Res<BaseballPluginConfig>,
    mut query_baseball: Query<(
        &mut BaseballFlightState,
        &Transform,
        &LinearVelocity,
        &mut ExternalForce,
    )>,
) {
    let delta_t = time_fixed.delta_seconds_f64() * time_physics.relative_speed_f64();
    for (mut state, transform, l_velo, mut force) in &mut query_baseball {
        if state.active {
            let a = state.update_state_and_get_acceleration(
                &baseball_plugin_config,
                transform
                    .translation
                    .from_bevy_to_baseball_coord()
                    .as_dvec3(),
                l_velo.0.from_bevy_to_baseball_coord().as_dvec3(),
                delta_t,
            );
            force.set_force(a.from_baseball_coord_to_bevy().as_vec3() * MASS);
            force.persistent = true;
        } else {
            // info!("inactive aerodynamics");
        }
    }
}

pub(crate) fn activate_aerodynamics(
    mut ball_physics_query: Query<(
        &mut BaseballFlightState,
        &mut ExternalForce,
        &mut GravityScale,
        &Transform,
        &LinearVelocity,
        &AngularVelocity,
    )>,
    mut ev_activate_aerodynamics_event: EventReader<ActivateAerodynamicsEvent>,
    mut ev_post_activate_aerodynamics_event: EventWriter<PostActivateAerodynamicsEvent>,
) {
    for ev in ev_activate_aerodynamics_event.read() {
        if let Ok((mut ball, mut force, mut gravity_scale, transform, l_velo, a_velo)) =
            ball_physics_query.get_mut(ev.entity)
        {
            if !ball.active {
                // just in case
                force.set_force(Vec3::ZERO);
                gravity_scale.0 = 0.;
                //
                *ball = BaseballFlightState::from_params(
                    transform
                        .translation
                        .from_bevy_to_baseball_coord()
                        .as_dvec3(),
                    l_velo.0.from_bevy_to_baseball_coord().as_dvec3(),
                    a_velo.0.as_dvec3(),
                    ev.seam_y_angle,
                    ev.seam_z_angle,
                );
                //
                ev_post_activate_aerodynamics_event.send(PostActivateAerodynamicsEvent(ev.entity));
            }
        }
    }
}

pub(crate) fn disable_aerodynamics(
    mut ball_physics_query: Query<(
        &mut BaseballFlightState,
        &mut ExternalForce,
        &mut GravityScale,
    )>,
    mut ev_disable_aerodynamics_event: EventReader<DisableAerodynamicsEvent>,
) {
    for ev in ev_disable_aerodynamics_event.read() {
        if let Ok((mut ball, mut force, mut gravity_scale)) = ball_physics_query.get_mut(ev.0) {
            if ball.active {
                ball.deactivate();
                force.set_force(Vec3::ZERO);
                gravity_scale.0 = 1.;
            }
        }
    }
}
