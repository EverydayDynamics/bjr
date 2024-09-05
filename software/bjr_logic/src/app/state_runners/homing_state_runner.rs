use crate::app::consts::MOTOR_NUM;
use crate::app::control_primitives::KinState;
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::motor_handler::ControlMode;
use crate::app::parameter_manager::{
    parameter_manager, HomingAccel, HomingHighVelocity, HomingLowVelocity, HomingMaxTravel,
    HomingSafePosition,
};
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerContext, StateRunnerError};
use device_traits::{LoggableMessage, Logger, MotorEnabler};
use core::fmt::{Display, Formatter};
use embedded_time::duration::Microseconds;
use crate::app::telemetry_handler::TelemetryBuilder;

#[derive(Copy, Clone)]
pub enum HomingStateRunnerState {
    Default,
    FastApproach,
    StoppingAfterFastApproach,
    SlowApproach,
    StoppingAfterSlowApproach,
    FirstGoingToSafeSpot,
    FinalGoingToSafeSpot,
    Done,
    Error,
}
#[cfg(not(feature = "defmt"))]
impl Display for HomingStateRunnerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            HomingStateRunnerState::Default => {
                write!(f, "Default")
            }
            HomingStateRunnerState::FastApproach => {
                write!(f, "FastApproach")
            }
            HomingStateRunnerState::StoppingAfterFastApproach => {
                write!(f, "StoppingAfterFastApproach")
            }
            HomingStateRunnerState::SlowApproach => {
                write!(f, "SlowApproach")
            }
            HomingStateRunnerState::StoppingAfterSlowApproach => {
                write!(f, "StoppingAfterSlowApproach")
            }
            HomingStateRunnerState::FirstGoingToSafeSpot => {
                write!(f, "FirstGoingToSafeSpot")
            }
            HomingStateRunnerState::FinalGoingToSafeSpot => {
                write!(f, "FinalGoingToSafeSpot")
            }
            HomingStateRunnerState::Done => {
                write!(f, "Done")
            }
            HomingStateRunnerState::Error => {
                write!(f, "Error")
            }
        }
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for  HomingStateRunnerState{
    fn format(&self, f: defmt::Formatter) {
        match self {
            HomingStateRunnerState::Default => {
                defmt::write!(f, "Default")
            }
            HomingStateRunnerState::FastApproach => {
                defmt::write!(f, "FastApproach")
            }
            HomingStateRunnerState::StoppingAfterFastApproach => {
                defmt::write!(f, "StoppingAfterFastApproach")
            }
            HomingStateRunnerState::SlowApproach => {
                defmt::write!(f, "SlowApproach")
            }
            HomingStateRunnerState::StoppingAfterSlowApproach => {
                defmt::write!(f, "StoppingAfterSlowApproach")
            }
            HomingStateRunnerState::FirstGoingToSafeSpot => {
                defmt::write!(f, "FirstGoingToSafeSpot")
            }
            HomingStateRunnerState::FinalGoingToSafeSpot => {
                defmt::write!(f, "FinalGoingToSafeSpot")
            }
            HomingStateRunnerState::Done => {
                defmt::write!(f, "Done")
            }
            HomingStateRunnerState::Error => {
                defmt::write!(f, "Error")
            }
        }
    }
}
struct HomingStateChangeMessage(HomingStateRunnerState, HomingStateRunnerState, usize);
impl LoggableMessage for HomingStateChangeMessage {}

#[cfg(not(feature = "defmt"))]
impl Display for  HomingStateChangeMessage{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f,"changing homing state from ({}) to ({}) in motor ({})", self.0, self.1, self.2)

    }
}


#[cfg(feature = "defmt")]
impl defmt::Format for  HomingStateChangeMessage{
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f,"changing homing state from ({}) to ({}) in motor ({})", self.0, self.1, self.2);
    }
}
pub struct HomingStateRunner {
    states: [HomingStateRunnerState; MOTOR_NUM],
}
impl Default for HomingStateRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl HomingStateRunner {
    pub fn new() -> Self {
        HomingStateRunner {
            states: [HomingStateRunnerState::Default; MOTOR_NUM],
        }
    }
}
impl RunnableState for HomingStateRunner {
    fn entry<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) {
        ctx.motor_enabler.set_enable(true);
        self.states = [HomingStateRunnerState::Default; MOTOR_NUM];
    }

    fn update<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) -> Result<(), StateRunnerError> {
        let homing_high_velocity = parameter_manager().get::<HomingHighVelocity>();
        let homing_low_velocity = parameter_manager().get::<HomingLowVelocity>();
        let homing_safe_position = parameter_manager().get::<HomingSafePosition>();
        let homing_max_travel = parameter_manager().get::<HomingMaxTravel>();
        let homing_accel = parameter_manager().get::<HomingAccel>();

        let mut retval = Ok(());
        let inputs = ctx.iomanager
            .read_motor_inputs(ctx.telemetry_builder)
            .map_err(StateRunnerError::HomingIOError)?;
        let mut outputs: [Option<(KinState, ControlMode)>; 3] = [None; MOTOR_NUM];
        let mut motor_idx = 0;
        let mut done_counter = 0;
        for ((state, input), output) in self
            .states
            .iter_mut()
            .zip(inputs.iter())
            .zip(outputs.iter_mut())
        {
            let mut maybe_next_state: Option<HomingStateRunnerState> = None;
            match state {
                HomingStateRunnerState::Default => {
                    ctx.iomanager
                        .reset_motor_pos(motor_idx)
                        .map_err(StateRunnerError::HomingIOError)?;
                    if input.1.limit_reached {
                        maybe_next_state.replace(HomingStateRunnerState::FirstGoingToSafeSpot);
                        output.replace((
                            KinState {
                                pos: homing_safe_position,
                                speed: homing_high_velocity,
                                accel: homing_accel,
                            },
                            ControlMode::Position,
                        ));
                    } else {
                        maybe_next_state.replace(HomingStateRunnerState::FastApproach);
                        output.replace((
                            KinState {
                                pos: -homing_max_travel,
                                speed: homing_high_velocity,
                                accel: homing_accel,
                            },
                            ControlMode::Position,
                        ));
                    }
                }
                HomingStateRunnerState::FastApproach => {
                    if input.1.limit_reached {
                        ctx.iomanager
                            .reset_motor_pos(motor_idx)
                            .map_err(StateRunnerError::HomingIOError)?;
                        maybe_next_state.replace(HomingStateRunnerState::StoppingAfterFastApproach);
                        output.replace((
                            KinState {
                                pos: 0.0,
                                speed: 0.0,
                                accel: homing_accel,
                            },
                            ControlMode::Velocity,
                        ));
                    } else if input.1.position_reached {
                        // Finished the motion without hitting the limit.
                        maybe_next_state.replace(HomingStateRunnerState::Error);
                        retval = Err(StateRunnerError::HomingOverrun);
                    }
                }

                HomingStateRunnerState::StoppingAfterFastApproach => {
                    if input.1.standstill {
                        maybe_next_state.replace(HomingStateRunnerState::FirstGoingToSafeSpot);
                        output.replace((
                            KinState {
                                pos: homing_safe_position,
                                speed: homing_high_velocity,
                                accel: homing_accel,
                            },
                            ControlMode::Position,
                        ));
                    }
                    //TODO find a way to timeout the stopping.
                }
                HomingStateRunnerState::SlowApproach => {
                    if input.1.limit_reached {
                        ctx.iomanager
                            .reset_motor_pos(motor_idx)
                            .map_err(StateRunnerError::HomingIOError)?;
                        maybe_next_state.replace(HomingStateRunnerState::StoppingAfterSlowApproach);
                        output.replace((
                            KinState {
                                pos: 0.0,
                                speed: 0.0,
                                accel: homing_accel,
                            },
                            ControlMode::Velocity,
                        ));
                    } else if input.1.position_reached {
                        // Finished the motion without hitting the limit.
                        maybe_next_state.replace(HomingStateRunnerState::Error);
                        retval = Err(StateRunnerError::HomingOverrun);
                    }
                }
                HomingStateRunnerState::StoppingAfterSlowApproach => {
                    if input.1.standstill {
                        maybe_next_state.replace(HomingStateRunnerState::FinalGoingToSafeSpot);
                        output.replace((
                            KinState {
                                pos: homing_safe_position,
                                speed: homing_high_velocity,
                                accel: homing_accel,
                            },
                            ControlMode::Position,
                        ));
                    }
                    //TODO find a way to timeout the stopping.
                }
                HomingStateRunnerState::FirstGoingToSafeSpot => {
                    if input.1.position_reached {
                        if input.1.limit_reached {
                            // Limit is stuck on at safe spot.
                            maybe_next_state.replace(HomingStateRunnerState::Error);
                            retval = Err(StateRunnerError::HomingLimistSWStuckAtSafePos);
                        } else {
                            maybe_next_state.replace(HomingStateRunnerState::SlowApproach);
                            output.replace((
                                KinState {
                                    pos: -homing_safe_position,
                                    speed: homing_low_velocity,
                                    accel: homing_accel,
                                },
                                ControlMode::Position,
                            ));
                        }
                    } else if input.1.standstill {
                        maybe_next_state.replace(HomingStateRunnerState::Error);
                        retval = Err(StateRunnerError::HomingUnexpectedStopGoingToSafePos);
                    }
                }
                HomingStateRunnerState::FinalGoingToSafeSpot => {
                    if input.1.position_reached {
                        if input.1.limit_reached {
                            // Limit is stuck on at safe spot.
                            maybe_next_state.replace(HomingStateRunnerState::Error);
                            retval = Err(StateRunnerError::HomingLimistSWStuckAtSafePos);
                        } else {
                            maybe_next_state.replace(HomingStateRunnerState::Done);
                        }
                    } else if input.1.standstill {
                        maybe_next_state.replace(HomingStateRunnerState::Error);
                        retval = Err(StateRunnerError::HomingUnexpectedStopGoingToSafePos);
                    }
                }
                HomingStateRunnerState::Done => {
                    done_counter += 1;
                }
                HomingStateRunnerState::Error => {
                    retval = Err(StateRunnerError::HomingUnexpectedStopGoingToSafePos);
                }
            }
            if let Some(next_state) = maybe_next_state {
                ctx.logger.debug(HomingStateChangeMessage(*state, next_state, motor_idx));
                *state = next_state;
            }
            motor_idx += 1;
        }
        ctx.iomanager
            .write_motor_outputs(outputs, ctx.telemetry_builder)
            .map_err(StateRunnerError::HomingIOError)?;
        if done_counter == MOTOR_NUM {
            // All motors homed.
            ctx.event_queue
                .enqueue(GlobEvent::HomingFinished)
                .map_err(StateRunnerError::QueueFull)?;
        }
        retval
    }

    fn exit<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) {}
}
