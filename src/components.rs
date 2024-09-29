use crate::*;

#[derive(Debug, Clone, Bundle)]
pub(crate) struct BaseballFlightBundle {
    pub state: BaseballFlightState,
    pub transform: Transform,
    pub collider: ColliderConstructor,
    pub rigid_body: RigidBody,
    pub linear_velocity: LinearVelocity,
    pub mass: Mass,
    pub gravity_scale: GravityScale,
}

impl Default for BaseballFlightBundle {
    fn default() -> Self {
        let collider = ColliderConstructor::Sphere { radius: RADIUS };
        Self {
            state: BaseballFlightState::default(),
            transform: Transform::default(),
            collider,
            rigid_body: RigidBody::Dynamic,
            linear_velocity: LinearVelocity::default(),
            mass: Mass(MASS),
            gravity_scale: GravityScale(0.0),
        }
    }
}
