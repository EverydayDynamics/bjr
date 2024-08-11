use embedded_time::duration::Microseconds;
use crate::app::control::control_primitives::{BallPattern, Controller};
use crate::app::control::feedforward_generator::FeedForwardGen;
use crate::app::control::setpoint_generator::SetPointGen;
use crate::app::motor_handler::ControlMode;

pub mod feedforward_generator;
pub mod pid_controller;
pub mod control_primitives;
pub mod inverse_kinematics;
mod setpoint_generator;

pub struct ControlExecutor<CTRL,FFG,SPG> {
    controller: CTRL,
    feedforward_gen: FFG,
    setpoint_gen: SPG,

}

impl<CTRL, FFG, SPG> ControlExecutor<CTRL, FFG, SPG>
where CTRL: Controller,
    FFG: FeedForwardGen,
    SPG: SetPointGen,
{
    pub fn new(controller: CTRL, feedforward_gen: FFG, setpoint_gen: SPG) -> ControlExecutor<CTRL, FFG, SPG> {
        ControlExecutor{
            controller,
            feedforward_gen,
            setpoint_gen,
        }
    }
    pub fn reset(&mut self, call_time: Microseconds<u64>){
        self.setpoint_gen.reset(call_time);
        self.feedforward_gen.reset(call_time);
    }
    pub fn update(&mut self, call_time: Microseconds<u64>, ){



    }
}