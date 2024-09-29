mod ball_flight_state;
mod common;
mod components;
mod events;
mod resources;
mod systems;

pub mod prelude {
    use crate::events::*;
    use crate::resources::*;
    use crate::utils::*;
}

use crate::systems::*;

pub(crate) use crate::resources::*;
pub(crate) use avian3d::prelude::*;
pub(crate) use ball_flight_state::*;
pub(crate) use bevy::{math::*, prelude::*};
pub(crate) use common::*;
pub(crate) use components::*;
pub(crate) use constants::*;
pub(crate) use events::*;
pub(crate) use utils::*;

pub struct BaseballFlightPlugin<S: States> {
    pub ssw_on: bool,
    pub magnus_on: bool,
    pub drag_on: bool,
    pub state: S,
}

impl<S: States> Plugin for BaseballFlightPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<ActivateAerodynamicsEvent>();
        app.add_event::<PostActivateAerodynamicsEvent>();
        app.add_event::<DisableAerodynamicsEvent>();

        app.register_type::<BaseballFlightState>();

        // app.add_systems(FixedUpdate, _apply_physics_option_1);
        // app.add_systems(FixedUpdate, _apply_physics_option_2);
        app.add_systems(FixedUpdate, _apply_physics_option_3);

        app.add_systems(FixedUpdate, activate_aerodynamics);
        app.add_systems(FixedUpdate, disable_aerodynamics);
    }
}
