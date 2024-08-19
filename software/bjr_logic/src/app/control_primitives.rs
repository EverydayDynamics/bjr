use embedded_time::duration::Microseconds;
use core::ops::{Add, Sub, Mul, Div, Neg};
use crate::app::parameter_manager::{FFDefaultAngAccel, FFDefaultAngSpeed, FFDefaultLinAccel, FFDefaultLinSpeed, parameter_manager};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct KinState{
    pub pos: f32,
    pub speed: f32,
    pub accel: f32,
}

impl Add for KinState {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            pos: self.pos + other.pos,
            speed: self.speed + other.speed,
            accel: self.accel + other.accel,
        }
    }
}

impl Sub for KinState {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            pos: self.pos - other.pos,
            speed: self.speed - other.speed,
            accel: self.accel - other.accel,
        }
    }
}
impl Neg for KinState {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            pos: -self.pos,
            speed: -self.speed,
            accel: -self.accel,
        }

    }
}

impl Mul<f32> for KinState {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        Self {
            pos: self.pos * scalar,
            speed: self.speed * scalar,
            accel: self.accel * scalar,
        }
    }
}

impl Div<f32> for KinState {
    type Output = Self;

    fn div(self, scalar: f32) -> Self::Output {
        Self {
            pos: self.pos / scalar,
            speed: self.speed / scalar,
            accel: self.accel / scalar,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PlateState {
    pub height: KinState,
    pub angle: [KinState;2],
}
impl PlateState {
    pub fn new_with_default_sa(height:f32, alpha:f32, beta:f32) -> PlateState{
        let default_lin_speed = parameter_manager().get::<FFDefaultLinSpeed>();
        let default_lin_accel = parameter_manager().get::<FFDefaultLinAccel>();
        let default_ang_speed = parameter_manager().get::<FFDefaultAngSpeed>();
        let default_ang_accel = parameter_manager().get::<FFDefaultAngAccel>();
        PlateState{ height: KinState{
            pos: height,
            speed: default_lin_speed,
            accel: default_lin_accel,
        }, angle: [
            KinState{
                pos: alpha,
                speed: default_ang_speed,
                accel: default_ang_accel,
            },
            KinState{
                pos: beta,
                speed: default_ang_speed,
                accel: default_ang_accel,
            },

        ] }
    }
}

#[derive(Default)]
pub struct PlateDelta {
    pub height: KinState,
    pub angle: [KinState;2],
}
impl Add<PlateDelta> for PlateState {
    type Output = PlateState;

    fn add(self, rhs: PlateDelta) -> Self::Output {
        Self {
            height: self.height + rhs.height,
            angle: [self.angle[0]+rhs.angle[0],self.angle[1]+rhs.angle[1]],
        }

    }
}
pub struct ControlExecInputs{

}
pub struct ControlInputs{
    pub ball_setpoint: [KinState;2],
    pub measured_plate_state: PlateState,
    pub measured_ball_state: [KinState;2],
}

pub struct TelemetryPacket{

}
pub enum BallPattern{
    CenterHold,
    Triangle,
    Circling,
}
pub trait Controller {

    fn reset(&mut self, call_time: Microseconds<u64>);
    fn update(&mut self, call_time: Microseconds<u64>, _inputs: ControlInputs) -> PlateState;
}