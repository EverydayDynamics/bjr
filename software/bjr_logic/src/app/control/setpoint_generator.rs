use crate::app::control::poly_trajectory::trajectory_5th_degree;
use crate::app::control_primitives::KinState;
use crate::app::parameter_manager::{
    parameter_manager, SP2Point1X, SP2Point1Y, SP2Point2X, SP2Point2Y, SP2PointDwellTime,
    SP2PointPathDuration, SP3Point1X, SP3Point1Y, SP3Point2X, SP3Point2Y, SP3Point3X, SP3Point3Y,
    SP3PointDwellTime, SP3PointPathDuration, SPCirclingRadius, SPCirclingTime,
};
use crate::utils::usec2sec;
use core::f32::consts::PI;
use embedded_time::duration::Microseconds;
use embedded_time::fixed_point::FixedPoint;
use libm::{cosf, fmodf, sinf};

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
    start_time: Microseconds<u64>,
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
        let (px, py) = motion_profile.position(delta_t);
        let (vx, vy) = motion_profile.velocity(delta_t);
        let (ax, ay) = motion_profile.acceleration(delta_t);
        [
            KinState {
                pos: px,
                speed: vx,
                accel: ax,
            },
            KinState {
                pos: py,
                speed: vy,
                accel: ay,
            },
        ]
    }
}
struct CircularMotion {
    radius: f32,
    omega: f32,
}

impl CircularMotion {
    fn angular_velocity_from_secs(period: f32) -> f32 {
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
#[derive(Default)]
pub struct SetPointGen2Point {
    start_time: Microseconds<u64>,
}
impl SetPointGen for SetPointGen2Point {
    fn reset(&mut self, time: Microseconds<u64>) {
        self.start_time = time;
    }

    fn get_sp(&mut self, time: Microseconds<u64>) -> [KinState; 2] {
        let p1x = parameter_manager().get::<SP2Point1X>();
        let p1y = parameter_manager().get::<SP2Point1Y>();
        let p2x = parameter_manager().get::<SP2Point2X>();
        let p2y = parameter_manager().get::<SP2Point2Y>();
        let path_duration = parameter_manager().get::<SP2PointPathDuration>();
        let dwell_time = parameter_manager().get::<SP2PointDwellTime>();
        let sequence_duration = (path_duration + dwell_time) * 2.0;
        let elapsed_time = fmodf(usec2sec(time.0 - self.start_time.0), sequence_duration);
        let mut result = [KinState::default(); 2];

        if elapsed_time < dwell_time {
            // dwelling at p1
            result[0].pos = p1x;
            result[1].pos = p1y;
        } else if elapsed_time < dwell_time + path_duration {
            let trajectory_time = elapsed_time - dwell_time;
            //moving to p2
            result[0] = trajectory_5th_degree(p1x, p2x, path_duration, trajectory_time);
            result[1] = trajectory_5th_degree(p1y, p2y, path_duration, trajectory_time);
        } else if elapsed_time < dwell_time * 2.0 + path_duration {
            // dwelling at p2
            result[0].pos = p2x;
            result[1].pos = p2y;
        } else {
            let trajectory_time = elapsed_time - (dwell_time * 2.0 + path_duration);
            //moving to p1
            result[0] = trajectory_5th_degree(p2x, p1x, path_duration, trajectory_time);
            result[1] = trajectory_5th_degree(p2y, p1y, path_duration, trajectory_time);
        }
        result
    }
}

#[derive(Default)]
pub struct SetPointGen3Point {
    start_time: Microseconds<u64>,
}
impl SetPointGen for SetPointGen3Point {
    fn reset(&mut self, time: Microseconds<u64>) {
        self.start_time = time;
    }

    fn get_sp(&mut self, time: Microseconds<u64>) -> [KinState; 2] {
        let p1x = parameter_manager().get::<SP3Point1X>();
        let p1y = parameter_manager().get::<SP3Point1Y>();
        let p2x = parameter_manager().get::<SP3Point2X>();
        let p2y = parameter_manager().get::<SP3Point2Y>();
        let p3x = parameter_manager().get::<SP3Point3X>();
        let p3y = parameter_manager().get::<SP3Point3Y>();
        let path_duration = parameter_manager().get::<SP3PointPathDuration>();
        let dwell_time = parameter_manager().get::<SP3PointDwellTime>();
        let sequence_duration = (path_duration + dwell_time) * 3.0;
        let elapsed_time = fmodf(usec2sec(time.0 - self.start_time.0), sequence_duration);
        let mut result = [KinState::default(); 2];

        if elapsed_time < dwell_time {
            // dwelling at p1
            result[0].pos = p1x;
            result[1].pos = p1y;
        } else if elapsed_time < dwell_time + path_duration {
            let trajectory_time = elapsed_time - dwell_time;
            //moving to p2
            result[0] = trajectory_5th_degree(p1x, p2x, path_duration, trajectory_time);
            result[1] = trajectory_5th_degree(p1y, p2y, path_duration, trajectory_time);
        } else if elapsed_time < dwell_time * 2.0 + path_duration {
            // dwelling at p2
            result[0].pos = p2x;
            result[1].pos = p2y;
        } else if elapsed_time < (dwell_time + path_duration) * 2.0 {
            let trajectory_time = elapsed_time - (dwell_time * 2.0 + path_duration);
            //moving to p1
            result[0] = trajectory_5th_degree(p2x, p3x, path_duration, trajectory_time);
            result[1] = trajectory_5th_degree(p2y, p3y, path_duration, trajectory_time);
        } else if elapsed_time < (dwell_time + path_duration) * 2.0 + dwell_time {
            // dwelling at p3
            result[0].pos = p3x;
            result[1].pos = p3y;
        } else {
            let trajectory_time = elapsed_time - ((dwell_time + path_duration) * 2.0 + dwell_time);
            //moving to p1
            result[0] = trajectory_5th_degree(p3x, p1x, path_duration, trajectory_time);
            result[1] = trajectory_5th_degree(p3y, p1y, path_duration, trajectory_time);
        }
        result
    }
}
