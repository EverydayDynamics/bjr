use core::fmt::{Display, Formatter};
use embedded_time::duration::Microseconds;
use crate::app::control_primitives::KinState;
use crate::app::motor_handler::{ControlMode, MotorHandler, MotorHandlerError, MotorStatus};

pub struct Inputs {
    measured_plate_angle: [KinState;2],
    measured_ball_state: [KinState;2],
    measured_motors_state: [(KinState, MotorStatus);3],
}
#[derive(Default)]
pub struct Outputs {
    piston_state: [KinState;3],
}
#[derive(PartialEq, Copy, Clone)]
pub enum IOManagerError {
    MotorInput(MotorHandlerError),
    MotorOutput(MotorHandlerError),
}
impl Display for IOManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            IOManagerError::MotorInput(e) => {write!(f, "IOMananger Motor input error: {}", e)}
            IOManagerError::MotorOutput(e) => {write!(f, "IOMananger Motor output error: {}", e)}
        }
    }
}
pub struct DefaultIOManager<'a> {
    motor_handler: MotorHandler<'a>,
    last_call_time: Microseconds<u64>
}
pub trait IOManager {
    fn read_all_inputs(&mut self, call_time: Microseconds<u64>) -> Result<Inputs, IOManagerError>;
    fn read_motor_inputs(&mut self) -> Result<[(KinState, MotorStatus);3], IOManagerError>;
    fn write_motor_outputs(&mut self, output: [Option<(KinState, ControlMode)>;3]) -> Result<(), IOManagerError>;
    fn reset_motor_pos(&mut self, motor_idx:usize) -> Result<(), IOManagerError>;
    fn write_all_outputs(&mut self, outputs: Outputs) -> Result<(), IOManagerError>;
}
impl<'a> DefaultIOManager<'a> {
    pub fn new(motor_handler: MotorHandler<'a>) -> DefaultIOManager<'a> {
        DefaultIOManager {
            motor_handler,
            last_call_time: Microseconds::<u64>::new(0),
        }
    }
}
impl<'a> IOManager for DefaultIOManager<'a> {
    fn read_all_inputs(&mut self, _call_time: Microseconds<u64>) -> Result<Inputs, IOManagerError>{
        let measured_motors_state =self.motor_handler.get_motor_state().map_err(|e| IOManagerError::MotorInput(e))?;
        Ok(Inputs{
            measured_plate_angle: Default::default(),
            measured_ball_state: Default::default(),
            measured_motors_state,
        })
    }

    fn read_motor_inputs(&mut self) -> Result<[(KinState, MotorStatus); 3], IOManagerError> {
        todo!()
    }

    fn write_motor_outputs(&mut self, _output: [Option<(KinState, ControlMode)>;3]) -> Result<(), IOManagerError> {
        todo!()
    }

    fn reset_motor_pos(&mut self, _motor_idx: usize) -> Result<(), IOManagerError> {
        todo!()
    }

    fn write_all_outputs(&mut self, _outputs: Outputs) -> Result<(), IOManagerError>{
        todo!()
    }
}