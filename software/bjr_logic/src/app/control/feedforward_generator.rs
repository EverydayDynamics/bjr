use core::f32::consts::PI;
use crate::app::control_primitives::{KinState, PlateDelta};
use embedded_time::duration::*;
use fast_math;
use libm::{cosf, sinf, sqrtf, fabs, fabsf};
use crate::app::state_runner::CirclingParams;
use crate::utils::usec2sec;

pub trait FeedForwardGen {
    fn reset(&mut self, time: Microseconds<u64>);
    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateDelta;
}

pub struct FFGenCH {}
impl FeedForwardGen for FFGenCH {
    fn reset(&mut self, _time: Microseconds<u64>) {}

    fn get_ff(&mut self, _time: Microseconds<u64>) -> PlateDelta {
        PlateDelta {
            height: Default::default(),
            angle: [KinState::default(); 2],
        }
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
    pub fn new(angulation_angle: f32, angulation_time: f32, height:f32) -> FFGenContCircle {
        FFGenContCircle{
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

    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateDelta {
        let runtime = usec2sec((time - self.start_time).integer());
        let angle_alpha = sinf(2.0*PI*runtime/self.angulation_time)*self.angulation_angle;
        let rate_alpha =2.0*PI/self.angulation_time *cosf(2.0*PI*runtime/self.angulation_time)*self.angulation_angle;
        let accel_alpha = -4.0*PI*PI/(self.angulation_time*self.angulation_time)*sinf(2.0*PI*runtime/self.angulation_time)*self.angulation_angle;

        let angle_beta = cosf(2.0*PI*runtime/self.angulation_time)*self.angulation_angle;
        let rate_beta =-2.0*PI/self.angulation_time *sinf(2.0*PI*runtime/self.angulation_time)*self.angulation_angle;
        let accel_beta = -4.0*PI*PI/(self.angulation_time*self.angulation_time)*cosf(2.0*PI*runtime/self.angulation_time)*self.angulation_angle;
        PlateDelta {
            height: KinState{
                pos: self.height,
                speed: 0.0,
                accel: 0.0,
            },
            angle: [KinState{
                pos: angle_alpha,
                speed: fabsf(rate_alpha),
                accel: fabsf(accel_alpha),
            },KinState{
                pos: angle_beta,
                speed: fabsf(rate_beta),
                accel: fabsf(accel_beta),
            }],
        }
    }
}
