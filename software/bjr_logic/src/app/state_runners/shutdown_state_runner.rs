use crate::app::state_runner::{RunnableState, StateRunnerContext, StateRunnerError};
use device_traits::Logger;
use crate::app::control::motor_controller::MotorController;
use crate::app::control_primitives::KinState;
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::Outputs;
use crate::app::motor_handler::ControlMode;
use crate::app::parameter_manager::{parameter_manager, FFDefaultLinAccel, FFDefaultLinSpeed, HomingAccel, HomingHighVelocity, HomingSafePosition};

#[derive(Default)]
pub struct ShutdownStateRunner {
    finished: bool,
    started: bool,
}
impl RunnableState for ShutdownStateRunner {
    fn entry<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) {
        ctx.motor_enabler.set_enable(true);
        self.started = false;
        self.finished = false;
    }
    fn update<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) -> Result<(), StateRunnerError> {
        if !self.started {
            self.started = true;
            let home_loc = parameter_manager().get::<HomingSafePosition>();
            let ff_default_lin_speed = parameter_manager().get::<HomingHighVelocity>();
            let ff_default_lin_accel = parameter_manager().get::<HomingAccel>();
            ctx.iomanager.write_all_outputs(Outputs {
                piston_state: [
                    (KinState{ pos:home_loc,
                        speed: ff_default_lin_speed,
                        accel: ff_default_lin_accel }, ControlMode::Position); 3],
            }, ctx.telemetry_builder)
                .map_err(StateRunnerError::IOError)
        }else {
            if !self.finished {
                let mut finished = true;
                let motors_input = ctx.iomanager.read_motor_inputs(ctx.telemetry_builder).map_err(|e|StateRunnerError::IOError(e))?;
                for motor_input in motors_input {
                    if !motor_input.1.position_reached {
                        finished = false;
                        if motor_input.1.standstill {
                            //Err(StateRunnerError::ShutdownUnexpectedStopGoingToSafePos)?;
                        }
                    } else {
                    }
                }
                self.finished = finished;
                Ok(())
            } else {
                ctx.motor_enabler.set_enable(false);
                ctx.event_queue.enqueue(GlobEvent::DeinitDone).map_err(StateRunnerError::QueueFull)
            }

        }
    }
    fn exit<LOG: Logger>(
        &mut self,
        _ctx: &mut StateRunnerContext<LOG>
    ) {}
}
