use core::default::Default;
use core::fmt::{Display, Formatter};
use crate::app::control::feedforward_generator::FeedForwardGen;
use crate::app::control::inverse_kinematics::inverse_kinematics;
use crate::app::control::setpoint_generator::SetPointGen;
use crate::app::control_primitives::{ControlInputs, Controller, KinState, PlateState};
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::{Inputs, IOManager, Outputs};
use crate::app::motor_handler::ControlMode;
use crate::app::parameter_manager::{parameter_manager, NoBallTargetHeight};
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerError};
use bsp_traits::{LoggableMessage, Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::consts::MOTOR_NUM;
use crate::app::control::MotorController::MotorController;

pub struct ControlStateRunner<CTRL, FFG, SPG> {
    controller: CTRL,
    ff_generator: FFG,
    sp_generator: SPG,
    counter: usize,
    motor_controllers: [MotorController;MOTOR_NUM],
}
struct CtrlDebugMsg<'a> (&'a PlateState);
impl<'a> LoggableMessage for CtrlDebugMsg<'a> {}
impl<'a> Display for CtrlDebugMsg<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "bst: x:{} \n y:{}", self.0.angle[0], self.0.angle[1])
    }
}
impl<CTRL, FFG, SPG> ControlStateRunner<CTRL, FFG, SPG>
where
    CTRL: Controller,
    FFG: FeedForwardGen,
    SPG: SetPointGen,
{
    pub fn new(
        controller: CTRL,
        ff_generator: FFG,
        sp_generator: SPG,
    ) -> ControlStateRunner<CTRL, FFG, SPG> {
        ControlStateRunner {
            controller,
            ff_generator,
            sp_generator,
            counter:0,
            motor_controllers: Default::default(),
        }
    }
}
impl<CTRL, FFG, SPG> RunnableState for ControlStateRunner<CTRL, FFG, SPG>
where
    CTRL: Controller,
    FFG: FeedForwardGen,
    SPG: SetPointGen,
{
    fn entry<LOG: Logger>(
        &mut self,
        call_time: Microseconds<u64>,
        _motor_enabler: &mut dyn MotorEnabler,
        _logger: &mut LOG,
    ) {
        self.controller.reset(call_time);
        self.ff_generator.reset(call_time);
        self.sp_generator.reset(call_time);
        self.counter=0;
    }
    fn update<LOG: Logger>(
        &mut self,
        iomanager: &mut dyn IOManager,
        call_time: Microseconds<u64>,
        _event_queue: EventQueue,
        logger: &mut LOG,
        _command: &StateRunnerCommand,
    ) -> Result<(), StateRunnerError> {
        let target_height_pos = parameter_manager().get::<NoBallTargetHeight>();
        let inputs = iomanager
            .read_all_inputs(call_time)
            .map_err(StateRunnerError::IOError)?;
        let feed_forward = self.ff_generator.get_ff(call_time);
        let target_plate_state = if let Some(ball_state) = inputs.measured_ball_state {
            // We have the ball, let's do some control
            let setpoint = self.sp_generator.get_sp(call_time);
            self.controller.update(
                call_time,
                ControlInputs {
                    ball_setpoint: setpoint,
                    measured_plate_state: PlateState {
                        height: Default::default(),
                        angle: [KinState::default(); 2],
                    },
                    measured_ball_state: ball_state,
                },
            )
        } else {
            //No ball found, send the plate to noball
            PlateState::new_with_null_sa(target_height_pos, 0.0, 0.0)
        };

        let final_target = target_plate_state + feed_forward;
        if self.counter >100 {
            logger.debug(CtrlDebugMsg(&(final_target)));
            self.counter =0;
        } else {
            self.counter +=1;
        }
        let motor_setpoints = inverse_kinematics(&final_target);
        let mut motor_outputs: [KinState;MOTOR_NUM] = Default::default();
        for mot_idx in 0..MOTOR_NUM {
            motor_outputs[mot_idx] = self.motor_controllers[mot_idx].calc(motor_setpoints[mot_idx], inputs.measured_motors_state[mot_idx].0);
        }

        iomanager
            .write_all_outputs(Outputs {
                piston_state: motor_outputs.map(|o| (o, ControlMode::Velocity)),
            })
            .map_err(StateRunnerError::IOError)?;
        Ok(())
    }
    fn exit<LOG: Logger>(&mut self, _call_time: Microseconds<u64>, _logger: &mut LOG) {}
}
