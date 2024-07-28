use core::fmt::Display;
use core::fmt::{Formatter};
use bsp_traits::{StepperMotorController, StepperDeviceError, MotorInput, MotorMode};
use crate::app::control_primitives::KinState;
use crate::app::limits::{LimitError, MotorPosLimit, MotorVelLimit, Limit};
use crate::app::parameter_manager::{parameter_manager, MotorM2Ustep};
use crate::app::consts::MOTOR_NUM;

#[derive(PartialEq, Copy, Clone)]
pub enum ControlMode {
    Position,
    Velocity,
}
#[derive(Default, Copy, Clone)]
pub struct MotorStatus {
   pub limit_reached: bool,
    pub position_reached: bool,
    pub velocity_reached: bool,
    pub standstill: bool,
}
#[derive(PartialEq, Copy, Clone)]
pub enum MotorHandlerError {
    MotorError(StepperDeviceError, usize),
    MotorPositionLimitError(LimitError<f32>, usize),
    MotorVelocityLimitError(LimitError<f32>, usize),
    InputOverflow,
}
impl Display for MotorHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            MotorHandlerError::MotorError(e, idx) => {write!(f, "Motor Handler Motor error: {}, motor idx: {}", e, idx)}
            MotorHandlerError::MotorPositionLimitError(e, idx) => {write!(f, "MotorHandler Position limit error: {}, motor idx: {}", e, idx)}
            MotorHandlerError::MotorVelocityLimitError(e, idx) => {write!(f, "MotorHandler Velocity limit error: {}, motor idx: {}", e, idx)}
            MotorHandlerError::InputOverflow => {write!(f, "MotorHandler Input value overflow")}
        }
    }
}

pub struct MotorHandler<'a> {
   motors: [&'a mut dyn StepperMotorController;MOTOR_NUM],
    accel: [f32;MOTOR_NUM],
    poslim: MotorPosLimit,
    vellim: MotorVelLimit,
}
impl MotorHandler<'_> {
    pub fn new(motors: [&mut dyn StepperMotorController;MOTOR_NUM], poslim: MotorPosLimit, vellim: MotorVelLimit) -> MotorHandler{
        MotorHandler{
            motors,
            accel: [0.0f32;3],
            poslim,
            vellim,
        }
    }
    pub fn get_motor_state(&mut self) -> Result<[(KinState, MotorStatus);MOTOR_NUM], MotorHandlerError> {
       let mut state:[(KinState, MotorStatus);MOTOR_NUM] = Default::default();
        let motm2us = parameter_manager().get::<MotorM2Ustep>();
        for motor_idx in 0..MOTOR_NUM {
            let motstate = self.motors[motor_idx].get_state().map_err(|e|MotorHandlerError::MotorError(e, motor_idx))?;
            let mut kinstate:KinState = Default::default();
            kinstate.accel = self.accel[motor_idx];
            kinstate.speed = (motstate.velocity as f64 /motm2us as f64) as f32;
            kinstate.pos = (motstate.position as f64 /motm2us as f64) as f32;
            self.vellim.check(kinstate.speed).map_err(|e|MotorHandlerError::MotorVelocityLimitError(e, motor_idx))?;
            self.poslim.check(kinstate.pos).map_err(|e|MotorHandlerError::MotorPositionLimitError(e, motor_idx))?;
            let status = MotorStatus{
                limit_reached: motstate.limit_reached,
                position_reached: motstate.position_reached,
                velocity_reached: motstate.velocity_reached,
                standstill: motstate.standstill,
            };
            state[motor_idx] = (kinstate, status)
        }
        Ok(state)
    }
    pub fn set_motor_input(&mut self, inputs: [(KinState, ControlMode);MOTOR_NUM]) -> Result<(),MotorHandlerError> {
        let motm2us = parameter_manager().get::<MotorM2Ustep>();
       for motor_idx in 0..MOTOR_NUM {

           let motor_mode = match inputs[motor_idx].1 {
               ControlMode::Position => MotorMode::PositionCtrl,
               ControlMode::Velocity => MotorMode::VelocityCtrl,
           };
           let input = MotorInput{
               velocity: ((inputs[motor_idx].0.speed *motm2us) as i32).try_into().map_err(|_|MotorHandlerError::InputOverflow)?,
               acceleration: ((inputs[motor_idx].0.accel *motm2us)as i32).try_into().map_err(|_|MotorHandlerError::InputOverflow)?,
               position: ((inputs[motor_idx].0.pos *motm2us) as i32).try_into().map_err(|_|MotorHandlerError::InputOverflow)?,
               mode: motor_mode,
           };
            self.motors[motor_idx].set_inputs(input).map_err(|e|MotorHandlerError::MotorError(e, motor_idx))?;
       }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use bsp_traits::{MotorInput, MotorState};
use super::*;
    use mockall::mock;
    use crate::app::event_queue::drain_event_queue;
    use crate::app::parameter_manager::parameter_manager;

    mock! {
        pub StepperMotorController {}
        impl StepperMotorController for StepperMotorController {
            fn set_inputs(&mut self, inputs: MotorInput) -> Result<(), StepperDeviceError>;
            fn get_state(&mut self) -> Result<MotorState, StepperDeviceError>;
        }
    }

    #[test]
    fn test_motors() {

    }
}