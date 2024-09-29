use crate::*;

#[derive(Debug, Clone, Copy, Reflect)]
pub(crate) enum GyroPole {
    Right,
    Left,
}

impl Default for GyroPole {
    fn default() -> Self {
        Self::Right
    }
}

#[derive(Debug, Reflect, Copy, Clone)]
pub(crate) struct Tilt(f32);
impl Tilt {
    pub(crate) fn from_hour_mintes(h: i8, m: i8) -> Self {
        assert!(h <= 12 && h > 0);
        let rad_hrs = (h - 3) as f32 * PI_32 / 6.;
        let rad_mins = m as f32 * PI_32 / 360.;
        Self(rad_hrs + rad_mins)
    }

    pub(crate) fn get(&self) -> f32 {
        self.0
    }
}

#[derive(Debug, Component, Reflect, Clone, Default)]
pub(crate) struct BaseballFlightState {
    pub translation: DVec3,
    pub v: DVec3,
    pub spin: DVec3,
    pub seams: Vec<DVec3>,
    pub time_elapsed: f64,
    //
    pub active: bool,
}

impl BaseballFlightState {
    pub(crate) fn deactivate(&mut self) {
        self.active = false;
        self.time_elapsed = 0.;
    }

    pub(crate) fn from_params(
        // position in baseball coord
        translation_: DVec3,
        // velocity in baseball coord
        velocity_: DVec3,
        // spin in rads
        spin_: DVec3,
        // in rad
        seam_y_angle_: f32,
        // in rad
        seam_z_angle_: f32,
        // other parameters...
    ) -> Self {
        let translation = translation_;
        let v = velocity_;
        let spin = spin_;
        let seam_y_angle = seam_y_angle_ as f64;
        let seam_z_angle = seam_z_angle_ as f64;

        let seams = (0..N_SEAMS)
            .map(|i| {
                let alpha =
                    (PI_64 * 2.) * (f64::from((i % N_SEAMS) as i16) / f64::from(N_SEAMS as i16));
                let x = (1. / 13.) * (9. * f64::cos(alpha) - 4. * f64::cos(3. * alpha));
                let y = (1. / 13.) * (9. * f64::sin(alpha) + 4. * f64::sin(3. * alpha));
                let z = (12. / 13.) * f64::cos(2. * alpha);
                DVec3::new(x, y, z) * (SEAM_DIAMETER / 2.)
            })
            .collect::<Vec<_>>();

        let seams_adjsuted = seams
            .iter()
            .map(|point| {
                // X axis of seams space should be the axis of rotation
                DQuat::from_rotation_arc(DVec3::X, spin.normalize()).mul_vec3(
                    DQuat::from_rotation_z(-seam_z_angle).mul_vec3(
                        DQuat::from_rotation_y(seam_y_angle).mul_vec3(
                            DQuat::from_rotation_y(PI_64 / 2.)
                                .mul_vec3(DQuat::from_rotation_x(-PI_64 / 2.).mul_vec3(*point)),
                        ),
                    ),
                )
            })
            .collect::<Vec<_>>();

        Self {
            translation,
            v,
            spin,
            seams: seams_adjsuted,
            time_elapsed: 0.,
            active: true,
        }
    }

    // option 3
    pub(crate) fn update_state_and_get_acceleration(
        &mut self,
        config: &BaseballPluginConfig,
        translation: DVec3,
        velocity: DVec3,
        delta_t: f64,
    ) -> DVec3 {
        self.translation = translation;
        self.v = velocity;

        self.update_state(config, delta_t);

        let distance = self.translation - translation;

        (self.v * self.v - velocity * velocity) / (2. * distance)
    }

    // option 2
    pub(crate) fn update_state_and_get_velo(
        &mut self,
        config: &BaseballPluginConfig,
        translation: DVec3,
        delta_t: f64,
    ) -> DVec3 {
        self.translation = translation;

        self.update_state(config, delta_t);

        (self.translation - translation) / delta_t
    }

    // option 1
    pub(crate) fn update_state(&mut self, config: &BaseballPluginConfig, delta_t: f64) {
        let iterations = (delta_t * 1000.).floor() as usize;

        for _ in 0..iterations {
            // rotate seams
            self.seams = self
                .seams
                .iter()
                .map(|point| {
                    // in seam space, the seams are rotating around the local x axis
                    DQuat::from_axis_angle(self.spin.normalize(), self.spin.length() * T_STEP)
                        .mul_vec3(*point)
                })
                .collect::<Vec<_>>();

            let active_seams = self.find_ssw_seams(&config.ssw);

            let a = self.rk4(config, &active_seams);

            self.time_elapsed += T_STEP;

            self.v += DVec3::new(a.x, a.y, a.z - 32.2) * T_STEP;
            self.translation += self.v * T_STEP;
        }
    }

    // find seam indices that affect ssw
    // note that the local x-axis of the seams is the rotational axis
    // we need to calculate the velocity vector in relation to the seams' local bases
    fn find_ssw_seams(&self, ssw: &SeamShiftedWake) -> Vec<usize> {
        // let v_adjusted = self.in_seam_space(self.v);
        let rot_v = DQuat::from_rotation_arc(-DVec3::Y, self.v.normalize());
        let rot_spin = DQuat::from_rotation_x(ssw.seam_shift_factor * T_STEP);
        let (max, min) = ssw.get_activation_region();

        (0..N_SEAMS)
            .filter(|&i| {
                let point_adjusted = rot_v.mul_vec3(
                    rot_v
                        .inverse()
                        .mul_vec3(rot_spin.inverse().mul_vec3(self.seams[i])),
                );
                if (point_adjusted.x < max.x)
                    && (point_adjusted.x > min.x)
                    // within y range
                    && (point_adjusted.y < max.y)
                    && (point_adjusted.y > min.y)
                    // within z range
                    && (point_adjusted.z < max.z)
                    && (point_adjusted.z > min.z)
                {
                    self.outside_separated_flow(ssw, i)
                } else {
                    false
                }
            })
            .collect::<Vec<_>>()
    }

    /// since seams in the activation region cannot cause a separated flow to
    /// become separated again this function will eliminate any inline seams
    fn outside_separated_flow(&self, ssw: &SeamShiftedWake, index: usize) -> bool {
        let point = &self.seams[index];
        let next_point = &self.seams[(index + 1) % N_SEAMS];
        let prev_point = &self.seams[(index + N_SEAMS - 1) % N_SEAMS];
        let normalized_v: &DVec3 = &self.v.normalize();

        let angle_d = normalized_v.dot((*point - *prev_point).normalize()).acos();
        let angle_u = normalized_v.dot((*next_point - *point).normalize()).acos();

        (angle_d - PI_64).abs() >= ssw.separated_flow_range
            && (angle_u - PI_64).abs() >= ssw.separated_flow_range
    }

    fn rk4(&self, config: &BaseballPluginConfig, active_seams: &Vec<usize>) -> DVec3 {
        let spin = &self.spin;
        let seams = &self.seams;
        let time_elapsed = self.time_elapsed as f64;

        let v_1 = self.v;
        let t_1 = time_elapsed;
        let a_1 = Self::derivs(config, &v_1, spin, seams, t_1, active_seams);

        let v_2 = v_1 + a_1 * T_STEP * 0.5;
        let t_2 = t_1 + T_STEP * 0.5;
        let a_2 = Self::derivs(config, &v_2, spin, seams, t_2, active_seams);

        let v_3 = v_2 + a_2 * T_STEP * 0.5;
        let t_3 = t_2 + T_STEP * 0.5;
        let a_3 = Self::derivs(config, &v_3, spin, seams, t_3, active_seams);

        let v_4 = v_3 + a_3 * T_STEP;
        let t_4 = t_3 + T_STEP;
        let a_4 = Self::derivs(config, &v_4, spin, seams, t_4, active_seams);

        let slope = (a_1 + 2. * (a_2 + a_3) + a_4) / 6.0;

        // if self.time_elapsed > 0. && self.time_elapsed < 0.012 {
        //     info!("a_1 {:?}", a_1);
        //     info!("a_2 {:?}", a_2);
        //     info!("a_3 {:?}", a_3);
        //     info!("a_4 {:?}", a_4);
        // }

        slope
    }

    fn derivs(
        config: &BaseballPluginConfig,
        v: &DVec3,
        spin: &DVec3,
        seams: &Vec<DVec3>,
        time_elapsed: f64,
        active_seams: &Vec<usize>,
    ) -> DVec3 {
        let v_tot = v.length();
        let spin_rate = spin.length();

        let rw = (DIAMETER / 2.) * spin_rate;
        let s = (rw / v_tot) * (-time_elapsed / SPIN_DECAY).exp();
        let cl = 1. / (2.42 + (0.4 / s));

        // drag force
        let a_drag = if config.drag_on {
            *v * -C_0 * CD_CONST * v_tot
        } else {
            DVec3::ZERO
        };

        // magnus force
        let a_spin = if config.magnus_on {
            let [u, v, w] = v.to_array();
            let [spin_x, spin_y, spin_z] = spin.to_array();
            DVec3::new(
                spin_y * w - spin_z * v,
                spin_z * u - spin_x * w,
                spin_x * v - spin_y * u,
            ) * C_0
                * (cl / spin_rate)
                * v_tot
        } else {
            DVec3::ZERO
        };

        // ssw
        let a_ssw = if config.ssw_on {
            let seams_length = active_seams
                .iter()
                .fold(DVec3::ZERO, |s_length, &i| s_length + seams[i]);
            seams_length * -C_0 * C_SEAMS * v_tot.powi(2)
        } else {
            DVec3::ZERO
        };

        a_drag + a_spin + a_ssw
    }
}
