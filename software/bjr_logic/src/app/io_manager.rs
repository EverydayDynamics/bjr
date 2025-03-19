use crate::app::control_primitives::KinState;
use crate::app::motor_handler::{ControlMode, MotorHandler, MotorHandlerError, MotorStatus};
use crate::app::touch_handler::{Differentiator, TouchHandler};
use device_traits::{StepperMotorController, TouchSensor, TouchSensorError};
use core::fmt::{Display, Formatter};
use crate::app::telemetry_handler::TelemetryBuilder;

pub struct Inputs {
    pub measured_plate_angle: [KinState; 2],
    pub measured_ball_state: Option<[KinState; 2]>,
    pub measured_motors_state: [(KinState, MotorStatus); 3],
}
#[derive(Default)]
pub struct Outputs {
    pub piston_state: [(KinState, ControlMode); 3],
}
#[derive(PartialEq, Copy, Clone)]
pub enum IOManagerError {
    MotorInput(MotorHandlerError),
    MotorOutput(MotorHandlerError),
    BallSensor(TouchSensorError),
}
impl Display for IOManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            IOManagerError::MotorInput(e) => {
                write!(f, "IOMananger Motor input error: {}", e)
            }
            IOManagerError::MotorOutput(e) => {
                write!(f, "IOMananger Motor output error: {}", e)
            }
            IOManagerError::BallSensor(e) => {
                write!(f, "IOMananger Ball sensor error: {}", e)
            }
        }
    }
}
pub struct DefaultIOManager<MA, MB, MC, TS, DIFF>
where
    MA: StepperMotorController,
    MB: StepperMotorController,
    MC: StepperMotorController,
    TS: TouchSensor,
    DIFF: Differentiator,
{
    motor_handler: MotorHandler<MA, MB, MC>,
    touch_handler: TouchHandler<TS, DIFF>,
}
pub trait IOManager {
    fn read_all_inputs(&mut self, telemetry_builder: &mut TelemetryBuilder) -> Result<Inputs, IOManagerError>;
    fn read_motor_inputs(&mut self, telemetry_builder: &mut TelemetryBuilder ) -> Result<[(KinState, MotorStatus); 3], IOManagerError>;
    fn write_motor_outputs(
        &mut self,
        output: [Option<(KinState, ControlMode)>; 3],
        telemetry_builder: &mut TelemetryBuilder,
    ) -> Result<(), IOManagerError>;
    fn reset_motor_pos(&mut self, motor_idx: usize) -> Result<(), IOManagerError>;
    fn write_all_outputs(&mut self, outputs: Outputs, telemetry_builder: &mut TelemetryBuilder) -> Result<(), IOManagerError>;
    fn motor_test_motion(&mut self, motor_idx: usize) -> Result<(), IOManagerError>;
}
impl<MA, MB, MC, TS, DIFF> DefaultIOManager<MA, MB, MC, TS, DIFF>
where
    MA: StepperMotorController,
    MB: StepperMotorController,
    MC: StepperMotorController,
    TS: TouchSensor,
    DIFF: Differentiator,
{
    pub fn new(
        motor_handler: MotorHandler<MA, MB, MC>,
        touch_handler: TouchHandler<TS, DIFF>,
    ) -> DefaultIOManager<MA, MB, MC, TS, DIFF> {
        DefaultIOManager {
            motor_handler,
            touch_handler,
        }
    }
}
impl<MA, MB, MC, TS, DIFF> IOManager for DefaultIOManager<MA, MB, MC, TS, DIFF>
where
    MA: StepperMotorController,
    MB: StepperMotorController,
    MC: StepperMotorController,
    TS: TouchSensor,
    DIFF: Differentiator,
{
    fn read_all_inputs(&mut self, telemetry_builder: &mut TelemetryBuilder) -> Result<Inputs, IOManagerError> {
        let measured_motors_state = self.read_motor_inputs(telemetry_builder)?;
        let measured_ball_state = self
            .touch_handler
            .get_ball_state(telemetry_builder)
            .map_err(IOManagerError::BallSensor)?;

        Ok(Inputs {
            measured_plate_angle: Default::default(),
            measured_ball_state,
            measured_motors_state,
        })
    }

    fn read_motor_inputs(&mut self, telemetry_builder: &mut TelemetryBuilder ) -> Result<[(KinState, MotorStatus); 3], IOManagerError> {
        self.motor_handler
            .get_motor_state(telemetry_builder)
            .map_err(IOManagerError::MotorInput)
    }

    fn write_motor_outputs(
        &mut self,
        output: [Option<(KinState, ControlMode)>; 3],
        telemetry_builder: &mut TelemetryBuilder,
    ) -> Result<(), IOManagerError> {
        self.motor_handler
            .maybe_set_motor_input(output, telemetry_builder)
            .map_err(IOManagerError::MotorOutput)
    }

    fn reset_motor_pos(&mut self, motor_idx: usize) -> Result<(), IOManagerError> {
        self.motor_handler
            .zero_motor_pos(motor_idx)
            .map_err(IOManagerError::MotorOutput)
    }

    fn write_all_outputs(&mut self, outputs: Outputs, telemetry_builder: &mut TelemetryBuilder) -> Result<(), IOManagerError> {
        self.motor_handler
            .set_motor_input(outputs.piston_state, telemetry_builder)
            .map_err(IOManagerError::MotorOutput)
    }

    fn motor_test_motion(&mut self, motor_idx: usize) -> Result<(), IOManagerError> {
        self.motor_handler
            .zero_motor_pos(motor_idx)
            .map_err(IOManagerError::MotorOutput)?;
        self.motor_handler.test_motion(motor_idx).map_err(IOManagerError::MotorOutput)
    }
}
