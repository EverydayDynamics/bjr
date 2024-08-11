use core::fmt::{Display, Formatter};
use bsp_traits::{StepperMotorController, TouchSensor, TouchSensorError};
use embedded_time::duration::Microseconds;
use crate::app::control_primitives::KinState;
use crate::app::motor_handler::{ControlMode, MotorHandler, MotorHandlerError, MotorStatus};
use crate::app::touch_handler::TouchHandler;

pub struct Inputs {
    pub measured_plate_angle: [KinState;2],
    pub measured_ball_state: Option<[KinState;2]>,
    pub measured_motors_state: [(KinState, MotorStatus);3],
}
#[derive(Default)]
pub struct Outputs {
    pub piston_state: [(KinState, ControlMode);3],
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
            IOManagerError::MotorInput(e) => {write!(f, "IOMananger Motor input error: {}", e)}
            IOManagerError::MotorOutput(e) => {write!(f, "IOMananger Motor output error: {}", e)}
            IOManagerError::BallSensor(e) => {write!(f, "IOMananger Ball sensor error: {}", e)}
        }
    }
}
pub struct DefaultIOManager<MA,MB,MC, TS>
    where
        MA: StepperMotorController,
        MB: StepperMotorController,
        MC: StepperMotorController,
        TS: TouchSensor,
{
    motor_handler: MotorHandler<MA,MB,MC>,
    touch_handler: TouchHandler<TS>,
    last_call_time: Microseconds<u64>
}
pub trait IOManager {
    fn read_all_inputs(&mut self, call_time: Microseconds<u64>) -> Result<Inputs, IOManagerError>;
    fn read_motor_inputs(&mut self) -> Result<[(KinState, MotorStatus);3], IOManagerError>;
    fn write_motor_outputs(&mut self, output: [Option<(KinState, ControlMode)>;3]) -> Result<(), IOManagerError>;
    fn reset_motor_pos(&mut self, motor_idx:usize) -> Result<(), IOManagerError>;
    fn write_all_outputs(&mut self, outputs: Outputs) -> Result<(), IOManagerError>;
}
impl<MA,MB,MC,TS> DefaultIOManager<MA,MB,MC,TS>
    where
        MA: StepperMotorController,
        MB: StepperMotorController,
        MC: StepperMotorController,
        TS: TouchSensor,
{
    pub fn new(motor_handler: MotorHandler<MA,MB,MC>, touch_handler: TouchHandler<TS>) -> DefaultIOManager<MA,MB,MC,TS> {
        DefaultIOManager {
            motor_handler,
            touch_handler,
            last_call_time: Microseconds::<u64>::new(0),
        }
    }
}
impl<MA,MB,MC,TS> IOManager for DefaultIOManager<MA,MB,MC,TS>
    where
        MA: StepperMotorController,
        MB: StepperMotorController,
        MC: StepperMotorController,
        TS: TouchSensor,
{
    fn read_all_inputs(&mut self, call_time: Microseconds<u64>) -> Result<Inputs, IOManagerError>{
        let measured_motors_state =self.read_motor_inputs()?;
        let measured_ball_state = self.touch_handler.get_ball_state(call_time).map_err(|e|IOManagerError::BallSensor(e))?;

        Ok(Inputs{
            measured_plate_angle: Default::default(),
            measured_ball_state,
            measured_motors_state,
        })
    }

    fn read_motor_inputs(&mut self) -> Result<[(KinState, MotorStatus); 3], IOManagerError> {
        self.motor_handler.get_motor_state().map_err(|e| IOManagerError::MotorInput(e))
    }

    fn write_motor_outputs(&mut self, output: [Option<(KinState, ControlMode)>;3]) -> Result<(), IOManagerError> {
        self.motor_handler.maybe_set_motor_input(output).map_err(|e|IOManagerError::MotorOutput(e))
    }

    fn reset_motor_pos(&mut self, motor_idx: usize) -> Result<(), IOManagerError> {
        self.motor_handler.zero_motor_pos(motor_idx).map_err(|e|IOManagerError::MotorOutput(e))
    }

    fn write_all_outputs(&mut self, outputs: Outputs) -> Result<(), IOManagerError>{
        self.motor_handler.set_motor_input(outputs.piston_state).map_err(|e|IOManagerError::MotorOutput(e))

    }
}