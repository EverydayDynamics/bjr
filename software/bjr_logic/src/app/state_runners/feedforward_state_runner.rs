use bsp_traits::{Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::control::inverse_kinematics::inverse_kinematics;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::{IOManager, Outputs};
use crate::app::motor_handler::ControlMode;
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerError};

#[derive(Default)]
pub struct FeedforwardStateRunner {
}
impl RunnableState for FeedforwardStateRunner {
    fn entry(&mut self, _call_time: Microseconds<u64>, _motor_enabler: &mut dyn MotorEnabler, logger: & dyn Logger) {}
    fn update(&mut self, iomanager: &mut dyn IOManager, _call_time: Microseconds<u64>, _event_queue: EventQueue, logger: &mut dyn Logger, command: &StateRunnerCommand) -> Result<(),StateRunnerError> {
        match command {
            StateRunnerCommand::FeedForwardCommand(ff_plate_state) => {
                let motor_outputs = inverse_kinematics(ff_plate_state);
                iomanager.write_all_outputs(Outputs{ piston_state: motor_outputs.map(|o|(o, ControlMode::Position)) }).map_err(|e| StateRunnerError::IOError(e))?;
            }
            StateRunnerCommand::NoCommand => {}
        }
        Ok(())
    }
    fn exit(&mut self, _call_time: Microseconds<u64>, logger: & dyn Logger) {}
}
