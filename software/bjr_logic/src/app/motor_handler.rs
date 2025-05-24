use crate::app::consts::MOTOR_NUM;
use crate::app::control_primitives::KinState;
use crate::app::limits::{Limit, LimitError, MotorPosLimit, MotorVelLimit};
use crate::app::parameter_manager::{parameter_manager, MotorM2Ustep};
use device_traits::{MotorInput, MotorMode, MotorState, StepperDeviceError, StepperMotorController};
use core::fmt::Display;
use core::fmt::Formatter;
use crate::app::telemetry_handler::TelemetryBuilder;

#[derive(PartialEq, Copy, Clone, Default)]
pub enum ControlMode {
    #[default]
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
#[cfg(not(feature = "defmt"))]
impl Display for MotorHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            MotorHandlerError::MotorError(e, idx) => {
                write!(f, "Motor Handler Motor error: {}, motor idx: {}", e, idx)
            }
            MotorHandlerError::MotorPositionLimitError(e, idx) => {
                write!(
                    f,
                    "MotorHandler Position limit error: {}, motor idx: {}",
                    e, idx
                )
            }
            MotorHandlerError::MotorVelocityLimitError(e, idx) => {
                write!(
                    f,
                    "MotorHandler Velocity limit error: {}, motor idx: {}",
                    e, idx
                )
            }
            MotorHandlerError::InputOverflow => {
                write!(f, "MotorHandler Input value overflow")
            }
            MotorHandlerError::MotorIdxOutOfRange(idx) => {
                write!(f, "MotorHandler Motor index ({}) out of range", idx)
            }
        }
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for MotorHandlerError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            MotorHandlerError::MotorError(e, idx) => {
                defmt::write!(f, "Motor Handler Motor error: {}, motor idx: {}", e, idx)
            }
            MotorHandlerError::MotorPositionLimitError(e, idx) => {
                defmt::write!(
                    f,
                    "MotorHandler Position limit error: {}, motor idx: {}",
                    e,
                    idx
                )
            }
            MotorHandlerError::MotorVelocityLimitError(e, idx) => {
                defmt::write!(
                    f,
                    "MotorHandler Velocity limit error: {}, motor idx: {}",
                    e,
                    idx
                )
            }
            MotorHandlerError::InputOverflow => {
                defmt::write!(f, "MotorHandler Input value overflow")
            }
            MotorHandlerError::MotorIdxOutOfRange(idx) => {
                defmt::write!(f, "MotorHandler Motor index ({}) out of range", idx)
            }
        }
    }
}
pub enum GenericMotor<MA, MB, MC>
where
    MA: StepperMotorController,
    MB: StepperMotorController,
    MC: StepperMotorController,
{
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
            GenericMotor::MotorA(ma) => ma.set_inputs(inputs),
            GenericMotor::MotorB(mb) => mb.set_inputs(inputs),
            GenericMotor::MotorC(mc) => mc.set_inputs(inputs),
        }
    }

    fn get_state(&mut self) -> Result<MotorState, StepperDeviceError> {
        match self {
            GenericMotor::MotorA(ma) => ma.get_state(),
            GenericMotor::MotorB(mb) => mb.get_state(),
            GenericMotor::MotorC(mc) => mc.get_state(),
        }
    }

    fn set_position(&mut self, new_position: i32) -> Result<(), StepperDeviceError> {
        match self {
            GenericMotor::MotorA(ma) => ma.set_position(new_position),
            GenericMotor::MotorB(mb) => mb.set_position(new_position),
            GenericMotor::MotorC(mc) => mc.set_position(new_position),
        }
    }
    fn test_motion(&mut self) -> Result<(), StepperDeviceError> {
        match self {
            GenericMotor::MotorA(ma) => ma.test_motion(),
            GenericMotor::MotorB(mb) => mb.test_motion(),
            GenericMotor::MotorC(mc) => mc.test_motion(),
        }
    }
}
pub struct MotorHandler<MA, MB, MC>
where
    MA: StepperMotorController,
    MB: StepperMotorController,
    MC: StepperMotorController,
{
    motors: [GenericMotor<MA, MB, MC>; MOTOR_NUM],
    accel: [f32; MOTOR_NUM],
    poslim: MotorPosLimit,
    vellim: MotorVelLimit,
}
impl<MA, MB, MC> MotorHandler<MA, MB, MC>
where
    MA: StepperMotorController,
    MB: StepperMotorController,
    MC: StepperMotorController,
{
    pub fn new(
        motor_a: MA,
        motor_b: MB,
        motor_c: MC,
        poslim: MotorPosLimit,
        vellim: MotorVelLimit,
    ) -> MotorHandler<MA, MB, MC> {
        MotorHandler {
            motors: [
                GenericMotor::MotorA(motor_a),
                GenericMotor::MotorB(motor_b),
                GenericMotor::MotorC(motor_c),
            ],
            accel: [0.0f32; 3],
            poslim,
            vellim,
        }
    }
    pub fn get_motor_state(
        &mut self, telemetry_builder: &mut TelemetryBuilder
    ) -> Result<[(KinState, MotorStatus); MOTOR_NUM], MotorHandlerError> {
        let mut state: [(KinState, MotorStatus); MOTOR_NUM] = Default::default();
        let motm2us = parameter_manager().get::<MotorM2Ustep>();
        for motor_idx in 0..MOTOR_NUM {
            let motstate = self.motors[motor_idx]
                .get_state()
                .map_err(|e| MotorHandlerError::MotorError(e, motor_idx))?;
            let mut kinstate: KinState = Default::default();
            kinstate.accel = self.accel[motor_idx];
            kinstate.speed = (motstate.velocity as f64 / motm2us as f64) as f32;
            kinstate.pos = (motstate.position as f64 / motm2us as f64) as f32;
            self.vellim
                .check(&mut kinstate.speed)
                .map_err(|e| MotorHandlerError::MotorVelocityLimitError(e, motor_idx))?;
            self.poslim
                .check(&mut kinstate.pos)
                .map_err(|e| MotorHandlerError::MotorPositionLimitError(e, motor_idx))?;
            let status = MotorStatus {
                limit_reached: motstate.limit_reached,
                position_reached: motstate.position_reached,
                velocity_reached: motstate.velocity_reached,
                standstill: motstate.standstill,
            };
            telemetry_builder.add_motor_state(motor_idx, &kinstate);
            state[motor_idx] = (kinstate, status)
        }
        Ok(state)
    }
    pub fn maybe_set_motor_input(
        &mut self,
        inputs: [Option<(KinState, ControlMode)>; MOTOR_NUM],
        telemetry_builder: &mut TelemetryBuilder
    ) -> Result<(), MotorHandlerError> {
        let motm2us = parameter_manager().get::<MotorM2Ustep>();
        for motor_idx in 0..MOTOR_NUM {
            if let Some((state, mode)) = inputs[motor_idx] {
                telemetry_builder.add_motor_target(motor_idx, &state);
                let motor_mode = match mode {
                    ControlMode::Position => MotorMode::PositionCtrl,
                    ControlMode::Velocity => MotorMode::VelocityCtrl,
                };
                let input = MotorInput {
                    velocity: ((state.speed * motm2us) as i32)
                        .try_into()
                        .map_err(|_| MotorHandlerError::InputOverflow)?,
                    acceleration: ((state.accel * motm2us) as i32)
                        .try_into()
                        .map_err(|_| MotorHandlerError::InputOverflow)?,
                    position: ((state.pos * motm2us) as i32)
                        .try_into()
                        .map_err(|_| MotorHandlerError::InputOverflow)?,
                    mode: motor_mode,
                };
                self.motors[motor_idx]
                    .set_inputs(input)
                    .map_err(|e| MotorHandlerError::MotorError(e, motor_idx))?;
            }
        }
        Ok(())
    }
    pub fn set_motor_input(
        &mut self,
        inputs: [(KinState, ControlMode); MOTOR_NUM],
        telemetry_builder: &mut TelemetryBuilder,
    ) -> Result<(), MotorHandlerError> {
        let motm2us = parameter_manager().get::<MotorM2Ustep>();
        for motor_idx in 0..MOTOR_NUM {
            let (state, mode) = inputs[motor_idx];
            telemetry_builder.add_motor_target(motor_idx, &state);
            let motor_mode = match mode {
                ControlMode::Position => MotorMode::PositionCtrl,
                ControlMode::Velocity => MotorMode::VelocityCtrl,
            };
            let input = MotorInput {
                velocity: ((state.speed * motm2us) as i32)
                    .try_into()
                    .map_err(|_| MotorHandlerError::InputOverflow)?,
                acceleration: ((state.accel * motm2us) as i32)
                    .try_into()
                    .map_err(|_| MotorHandlerError::InputOverflow)?,
                position: ((state.pos * motm2us) as i32)
                    .try_into()
                    .map_err(|_| MotorHandlerError::InputOverflow)?,
                mode: motor_mode,
            };

            self.motors[motor_idx]
                .set_inputs(input)
                .map_err(|e| MotorHandlerError::MotorError(e, motor_idx))?;
        }
        Ok(())
    }
    pub fn zero_motor_pos(&mut self, motor_idx: usize) -> Result<(), MotorHandlerError> {
        if motor_idx < MOTOR_NUM {
            self.motors[motor_idx]
                .set_position(0)
                .map_err(|e| MotorHandlerError::MotorError(e, motor_idx))
        } else {
            Err(MotorHandlerError::MotorIdxOutOfRange(motor_idx))
        }
    }
    pub fn test_motion(&mut self, motor_idx: usize) -> Result<(), MotorHandlerError> {
        if motor_idx < MOTOR_NUM {
            self.motors[motor_idx]
                .test_motion()
                .map_err(|e| MotorHandlerError::MotorError(e, motor_idx))
        } else {
            Err(MotorHandlerError::MotorIdxOutOfRange(motor_idx))
        }
    }
}