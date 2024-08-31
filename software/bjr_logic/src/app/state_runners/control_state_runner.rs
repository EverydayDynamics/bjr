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
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerContext, StateRunnerError};
use bsp_traits::{LoggableMessage, Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::consts::MOTOR_NUM;
use crate::app::control::MotorController::MotorController;
use crate::app::telemetry_handler::TelemetryBuilder;

pub struct ControlStateRunner<CTRL, FFG, SPG> {
    controller: CTRL,
    ff_generator: FFG,
    sp_generator: SPG,
    counter: usize,
    motor_controllers: [MotorController;MOTOR_NUM],
    last_plate_state: PlateState,
}
struct CtrlDebugMsg<'a> (&'a [KinState;2]);
impl<'a> LoggableMessage for CtrlDebugMsg<'a> {}
impl<'a> Display for CtrlDebugMsg<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "sp:\n x:{} \n y:{}\n t:{}", self.0[0].pos, self.0[1].pos, self.0[0].accel)
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
            last_plate_state:  PlateState::new_with_null_sa(0.010, 0.0, 0.0)



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
        ctx: &mut StateRunnerContext<LOG>
    ) {
        self.controller.reset(ctx.call_time);
        self.ff_generator.reset(ctx.call_time);
        self.sp_generator.reset(ctx.call_time);
        self.counter=0;
    }
    fn update<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) -> Result<(), StateRunnerError> {
        let target_height_pos = parameter_manager().get::<NoBallTargetHeight>();
        let inputs = ctx.iomanager
            .read_all_inputs(ctx.call_time, ctx.telemetry_builder)
            .map_err(StateRunnerError::IOError)?;
        let feed_forward = self.ff_generator.get_ff(ctx.call_time);
        let target_plate_state = if let Some(ball_state) = inputs.measured_ball_state {
            if self.counter >0 {
                self.counter =0;
                let setpoint = self.sp_generator.get_sp(ctx.call_time);
                ctx.logger.debug(CtrlDebugMsg(&setpoint));
                self.last_plate_state = self.controller.update(
                    ctx.call_time,
                    ControlInputs {
                        ball_setpoint: setpoint,
                        measured_plate_state: PlateState {
                            height: Default::default(),
                            angle: [KinState::default(); 2],
                        },
                        measured_ball_state: ball_state,
                    },
                );
            } else {
                self.counter +=1;
            }
            self.last_plate_state
            // We have the ball, let's do some control
        } else {
            //No ball found, send the plate to noball
            PlateState::new_with_null_sa(target_height_pos, 0.0, 0.0)
        };

        let final_target = target_plate_state + feed_forward;
        let motor_setpoints = inverse_kinematics(&final_target);
        let mut motor_outputs: [KinState;MOTOR_NUM] = Default::default();
        for mot_idx in 0..MOTOR_NUM {
            let mut tracking_error:f32 = 0.0;
            (motor_outputs[mot_idx], tracking_error) = self.motor_controllers[mot_idx].calc(motor_setpoints[mot_idx], inputs.measured_motors_state[mot_idx].0);
            ctx.telemetry_builder.add_motor_control(mot_idx, tracking_error);
        }

        ctx.iomanager
            .write_all_outputs(Outputs {
                piston_state: motor_outputs.map(|o| (o, ControlMode::Velocity)),
            }, ctx.telemetry_builder)
            .map_err(StateRunnerError::IOError)?;
        Ok(())
    }
    fn exit<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) {}
}
