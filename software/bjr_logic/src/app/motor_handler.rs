use core::fmt::Display;
use core::fmt::{Formatter};
use bsp_traits::{StepperMotorController, StepperDeviceError, MotorInput, MotorMode, MotorState};
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
    MotorIdxOutOfRange(usize),
}
impl Display for MotorHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            MotorHandlerError::MotorError(e, idx) => {write!(f, "Motor Handler Motor error: {}, motor idx: {}", e, idx)}
            MotorHandlerError::MotorPositionLimitError(e, idx) => {write!(f, "MotorHandler Position limit error: {}, motor idx: {}", e, idx)}
            MotorHandlerError::MotorVelocityLimitError(e, idx) => {write!(f, "MotorHandler Velocity limit error: {}, motor idx: {}", e, idx)}
            MotorHandlerError::InputOverflow => {write!(f, "MotorHandler Input value overflow")}
            MotorHandlerError::MotorIdxOutOfRange(idx) => {write!(f, "MotorHandler Motor index ({}) out of range", idx)}
        }
    }
}
pub enum GenericMotor<MA, MB, MC> where MA: StepperMotorController, MB: StepperMotorController, MC: StepperMotorController {
    MotorA(MA),
    MotorB(MB),
    MotorC(MC),
}
impl<MA, MB, MC> StepperMotorController for GenericMotor<MA, MB, MC>
where
MA: StepperMotorController,
MB: StepperMotorController,
MC: StepperMotorController,
{
    fn set_inputs(&mut self, inputs: MotorInput) -> Result<(), StepperDeviceError> {
        match self {
            GenericMotor::MotorA(ma) => {ma.set_inputs(inputs)}
            GenericMotor::MotorB(mb) => {mb.set_inputs(inputs)}
            GenericMotor::MotorC(mc) => {mc.set_inputs(inputs)}
        }
    }

    fn get_state(&mut self) -> Result<MotorState, StepperDeviceError> {
        match self {
            GenericMotor::MotorA(ma) => {ma.get_state()}
            GenericMotor::MotorB(mb) => {mb.get_state()}
            GenericMotor::MotorC(mc) => {mc.get_state()}
        }
    }

    fn set_position(&mut self, new_position: i32) -> Result<(), StepperDeviceError> {
        match self {
            GenericMotor::MotorA(ma) => {ma.set_position(new_position)}
            GenericMotor::MotorB(mb) => {mb.set_position(new_position)}
            GenericMotor::MotorC(mc) => {mc.set_position(new_position)}
        }
    }
}
pub struct MotorHandler<MA, MB, MC>
    where
        MA: StepperMotorController,
        MB: StepperMotorController,
        MC: StepperMotorController,
{
   motors: [GenericMotor<MA, MB, MC>;MOTOR_NUM],
    accel: [f32;MOTOR_NUM],
    poslim: MotorPosLimit,
    vellim: MotorVelLimit,
}
impl<MA, MB, MC> MotorHandler<MA, MB, MC>
    where
        MA: StepperMotorController,
        MB: StepperMotorController,
        MC: StepperMotorController,
{
    pub fn new(motor_a: MA, motor_b: MB, motor_c: MC, poslim: MotorPosLimit, vellim: MotorVelLimit) -> MotorHandler<MA, MB, MC> {
        MotorHandler{
            motors: [GenericMotor::MotorA(motor_a), GenericMotor::MotorB(motor_b), GenericMotor::MotorC(motor_c)],
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
    pub fn set_motor_input(&mut self, inputs: [Option<(KinState, ControlMode)>;MOTOR_NUM]) -> Result<(),MotorHandlerError> {
        let motm2us = parameter_manager().get::<MotorM2Ustep>();
       for motor_idx in 0..MOTOR_NUM {
           if let Some((state, mode)) = inputs[motor_idx] {
               let motor_mode = match mode {
                   ControlMode::Position => MotorMode::PositionCtrl,
                   ControlMode::Velocity => MotorMode::VelocityCtrl,
               };
               let input = MotorInput{
                   velocity: ((state.speed *motm2us) as i32).try_into().map_err(|_|MotorHandlerError::InputOverflow)?,
                   acceleration: ((state.accel *motm2us)as i32).try_into().map_err(|_|MotorHandlerError::InputOverflow)?,
                   position: ((state.pos *motm2us) as i32).try_into().map_err(|_|MotorHandlerError::InputOverflow)?,
                   mode: motor_mode,
               };
               self.motors[motor_idx].set_inputs(input).map_err(|e|MotorHandlerError::MotorError(e, motor_idx))?;

           }
       }
        Ok(())
    }
    pub fn zero_motor_pos(&mut self, motor_idx: usize) -> Result<(),MotorHandlerError> {
        if (motor_idx < MOTOR_NUM) {
            self.motors[motor_idx].set_position(0).map_err(|e|MotorHandlerError::MotorError(e, motor_idx))
        }else {
            Err(MotorHandlerError::MotorIdxOutOfRange(motor_idx))
        }
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
            fn set_position(&mut self, new_position: i32) -> Result<(), StepperDeviceError>;
        }
    }

    #[test]
    fn test_motors() {

    }
}