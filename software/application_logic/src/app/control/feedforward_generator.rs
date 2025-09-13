use crate::app::control_primitives::{KinState, PlateState};
use crate::app::state_runner::CirclingParams;
use crate::utils::usec2sec;
use core::f32::consts::PI;
use embedded_time::duration::*;
use libm::{cosf, fabsf, sinf};

const DEG2RAD: f32 = 0.017_453_292;
pub trait FeedForwardGen {
    fn reset(&mut self, time: Microseconds<u64>);
    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateState;
    fn finished(&self) -> bool;
}

pub struct FFGenCH {}
impl FeedForwardGen for FFGenCH {
    fn reset(&mut self, _time: Microseconds<u64>) {}

    fn get_ff(&mut self, _time: Microseconds<u64>) -> PlateState {
        PlateState {
            height: Default::default(),
            angle: [KinState::default(); 2],
        }
    }

    fn finished(&self) -> bool {
        false
    }
}
#[derive(Default)]
pub struct FFGenContCircle {
    angulation_angle: f32,
    angulation_time: f32,
    height: f32,
    start_time: Microseconds<u64>,
}
impl FFGenContCircle {
    pub fn new(angulation_angle: f32, angulation_time: f32, height: f32) -> FFGenContCircle {
        FFGenContCircle {
            angulation_angle,
            angulation_time,
            height,
            start_time: Default::default(),
        }
    }
    pub fn update_params(&mut self, params: &CirclingParams) {
        self.angulation_time = params.angulation_time;
        self.angulation_angle = params.angulation_angle;
        self.height = params.height;
    }
}
impl FeedForwardGen for FFGenContCircle {
    fn reset(&mut self, time: Microseconds<u64>) {
        self.start_time = time;
    }

    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateState {
        let runtime = usec2sec((time - self.start_time).integer());
        let angle_alpha = sinf(2.0 * PI * runtime / self.angulation_time) * self.angulation_angle;
        let rate_alpha = 2.0 * PI / self.angulation_time
            * cosf(2.0 * PI * runtime / self.angulation_time)
            * self.angulation_angle;
        let accel_alpha = -4.0 * PI * PI / (self.angulation_time * self.angulation_time)
            * sinf(2.0 * PI * runtime / self.angulation_time)
            * self.angulation_angle;

        let angle_beta = cosf(2.0 * PI * runtime / self.angulation_time) * self.angulation_angle;
        let rate_beta = -2.0 * PI / self.angulation_time
            * sinf(2.0 * PI * runtime / self.angulation_time)
            * self.angulation_angle;
        let accel_beta = -4.0 * PI * PI / (self.angulation_time * self.angulation_time)
            * cosf(2.0 * PI * runtime / self.angulation_time)
            * self.angulation_angle;
        PlateState {
            height: KinState {
                pos: self.height,
                speed: 0.0,
                accel: 0.0,
            },
            angle: [
                KinState {
                    pos: angle_alpha,
                    speed: fabsf(rate_alpha),
                    accel: fabsf(accel_alpha),
                },
                KinState {
                    pos: angle_beta,
                    speed: fabsf(rate_beta),
                    accel: fabsf(accel_beta),
                },
            ],
        }
    }
    fn finished(&self) -> bool {
        false
    }
}
#[derive(Default)]
pub struct FFGenMotionDemo {
    start_time: Microseconds<u64>,
}
impl FFGenMotionDemo {
    pub fn new(angulation_angle: f32, angulation_time: f32, height: f32) -> FFGenContCircle {
        FFGenContCircle {
            angulation_angle,
            angulation_time,
            height,
            start_time: Default::default(),
        }
    }
}
impl FeedForwardGen for FFGenMotionDemo {
    fn reset(&mut self, time: Microseconds<u64>) {
        self.start_time = time;
    }

    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateState {
        let runtime = usec2sec((time - self.start_time).integer());
        let dwell_time = 2.0f32;
        let default = PlateState::new_with_default_sa(10e-3, 0.0, 0.0);
        let lin_max = PlateState::new_with_default_sa(20e-3, 0.0, 0.0);
        let lin_min = PlateState::new_with_default_sa(1e-3, 0.0, 0.0);
        let roll_max = PlateState::new_with_default_sa(10e-3, 20.0 * DEG2RAD, 0.0);
        let roll_min = PlateState::new_with_default_sa(10e-3, -20.0 * DEG2RAD, 0.0);
        let pitch_max = PlateState::new_with_default_sa(10e-3, 0.0, 20.0 * DEG2RAD);
        let pitch_min = PlateState::new_with_default_sa(10e-3, 0.0, -20.0 * DEG2RAD);
        let sequence = [
            &default, &lin_max, &lin_min, &lin_max, &lin_min, &default, &roll_max, &roll_min,
            &roll_max, &roll_min, &default, &pitch_max, &pitch_min, &pitch_max, &pitch_min,
            &default,
        ];
        let mut idx = (runtime / dwell_time) as usize;
        if idx >= sequence.len() {
            idx = sequence.len() - 1;
        }
        *sequence[idx]
    }
    fn finished(&self) -> bool {
        false
    }
}
