use core::f32::consts::PI;
use crate::app::control_primitives::KinState;
use embedded_time::duration::Microseconds;
use embedded_time::fixed_point::FixedPoint;
use libm::{cosf, sinf};
use crate::app::parameter_manager::{parameter_manager, SPCirclingRadius, SPCirclingTime};
use crate::utils::usec2sec;

pub struct SetPointGenCH {}
impl SetPointGen for SetPointGenCH {
    fn reset(&mut self, _time: Microseconds<u64>) {}

    fn get_sp(&mut self, _time: Microseconds<u64>) -> [KinState; 2] {
        [KinState {
            pos: 0.0,
            speed: 0.0,
            accel: 0.0,
        }; 2]
    }
}
pub trait SetPointGen {
    fn reset(&mut self, time: Microseconds<u64>);
    fn get_sp(&mut self, time: Microseconds<u64>) -> [KinState; 2];
}

#[derive(Default)]
pub struct SetPointGenCircling {
    start_time: Microseconds<u64>
}
impl SetPointGen for SetPointGenCircling {
    fn reset(&mut self, time: Microseconds<u64>) {
        self.start_time = time;
    }

    fn get_sp(&mut self, time: Microseconds<u64>) -> [KinState; 2] {
        let delta_t = usec2sec((time - self.start_time).integer());
        let radius = parameter_manager().get::<SPCirclingRadius>();
        let period = parameter_manager().get::<SPCirclingTime>();
        let motion_profile = CircularMotion::new(radius, period);
        let (px,py) = motion_profile.position(delta_t);
        let (vx,vy) = motion_profile.velocity(delta_t);
        let (ax,ay) = motion_profile.acceleration(delta_t);
        [KinState {
            pos: px,
            speed: vx,
            accel: ax,
        }, KinState{
            pos: py,
            speed: vy,
            accel: ay,
        }]
    }
}
struct CircularMotion {
    radius: f32,
    omega: f32,
}

impl CircularMotion {
    fn angular_velocity_from_secs(period:f32) -> f32 {
        2.0 * PI / period
    }
    fn new(radius: f32, period: f32) -> Self {
        let omega = Self::angular_velocity_from_secs(period);
        CircularMotion { radius, omega }
    }


    fn position(&self, t: f32) -> (f32, f32) {
        let x = self.radius * cosf(self.omega * t);
        let y = self.radius * sinf(self.omega * t);
        (x, y)
    }

    fn velocity(&self, t: f32) -> (f32, f32) {
        let v_x = -self.radius * self.omega * sinf(self.omega * t);
        let v_y = self.radius * self.omega * cosf(self.omega * t);
        (v_x, v_y)
    }

    fn acceleration(&self, t: f32) -> (f32, f32) {
        let a_x = -self.radius * self.omega * self.omega * cosf(self.omega * t);
        let a_y = -self.radius * self.omega * self.omega * sinf(self.omega * t);
        (a_x, a_y)
    }
}
