use core::fmt::{Display, Formatter};
use crate::app::control::inverse_kinematics::inverse_kinematics;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::{IOManager, Outputs};
use crate::app::motor_handler::ControlMode;
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerError};
use bsp_traits::{LoggableMessage, Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use embedded_time::fixed_point::FixedPoint;
use strum_macros::Display;
use crate::app::consts::MOTOR_NUM;
use crate::app::control::feedforward_generator::{FeedForwardGen, FFGenContCircle};
use crate::app::control::MotorController::MotorController;
use crate::app::control_primitives::{KinState, PlateState};

#[derive(Default, Display, Copy, Clone)]
enum FeedForwardStateRunnerState {
    #[default]
    NoState,
    Circling,
}
#[derive(Default)]
pub struct FeedforwardStateRunner {
    state: FeedForwardStateRunnerState,
    ff_circler: FFGenContCircle,
    motor_controllers: [MotorController;3],
}

struct FFDebugMsg (FeedForwardStateRunnerState, u64);

impl LoggableMessage for FFDebugMsg {}
impl Display for FFDebugMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
       writeln!(f,"state: {}, t:{}", self.0, self.1)
    }
}
impl RunnableState for FeedforwardStateRunner {
    fn entry<LOG: Logger>(
        &mut self,
        _call_time: Microseconds<u64>,
        _motor_enabler: &mut dyn MotorEnabler,
        _logger: &mut LOG,
    ) {
        //self.state = FeedForwardStateRunnerState::NoState;
    }
    fn update<LOG: Logger>(
        &mut self,
        iomanager: &mut dyn IOManager,
        call_time: Microseconds<u64>,
        _event_queue: EventQueue,
        logger: &mut LOG,
        command: &StateRunnerCommand,
    ) -> Result<(), StateRunnerError> {
        match command {
            StateRunnerCommand::FeedForwardPlateCommand(ff_plate_state) => {
                let motor_outputs = inverse_kinematics(ff_plate_state);
                iomanager
                    .write_all_outputs(Outputs {
                        piston_state: motor_outputs.map(|o| (o, ControlMode::Position)),
                    })
                    .map_err(StateRunnerError::IOError)?;

                self.state = FeedForwardStateRunnerState::NoState;
            }
            StateRunnerCommand::NoCommand => {}
            StateRunnerCommand::FeedForwardMotorCommand(motor_command) => {
                iomanager.write_all_outputs(Outputs {
                    piston_state: motor_command.map(|o| (o, ControlMode::Position)),
                })
                    .map_err(StateRunnerError::IOError)?;

                self.state = FeedForwardStateRunnerState::NoState;
            }
            StateRunnerCommand::FeedForwardCircling(circling_params) => {
                self.ff_circler.update_params(circling_params);
                self.state = FeedForwardStateRunnerState::Circling;
            }
        }
        logger.debug(FFDebugMsg(self.state, call_time.integer()));
        match self.state {
            FeedForwardStateRunnerState::NoState => {}
            FeedForwardStateRunnerState::Circling => {
                let motors_state = iomanager.read_motor_inputs().map_err(|e|StateRunnerError::IOError(e))?;
                let desired_state = PlateState::default() + self.ff_circler.get_ff(call_time);
                let mut motors_setpoint = inverse_kinematics(&desired_state);
                let mut motor_outputs: [KinState;3] = Default::default();
                for mot_idx in 0..MOTOR_NUM {
                    motor_outputs[mot_idx] = self.motor_controllers[mot_idx].calc(motors_setpoint[mot_idx], motors_state[mot_idx].0);
                }
                iomanager
                    .write_all_outputs(Outputs {
                        piston_state: motor_outputs.map(|o| (o, ControlMode::Velocity)),
                    })
                    .map_err(StateRunnerError::IOError)?;
            }
        }
        Ok(())
    }
    fn exit<LOG: Logger>(&mut self, _call_time: Microseconds<u64>, _logger: &mut LOG) {}
}
