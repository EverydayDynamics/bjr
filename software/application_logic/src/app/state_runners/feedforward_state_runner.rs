use crate::app::consts::MOTOR_NUM;
use crate::app::control::feedforward_generator::{
    FFGenContCircle, FFGenMotionDemo, FeedForwardGen,
};
use crate::app::control::inverse_kinematics::inverse_kinematics;
use crate::app::control::motor_controller::MotorController;
use crate::app::control_primitives::{KinState, PlateState};
use crate::app::io_manager::Outputs;
use crate::app::motor_handler::ControlMode;
use crate::app::state_runner::StateRunnerCommand::NoCommand;
use crate::app::state_runner::{
    RunnableState, StateRunnerCommand, StateRunnerContext, StateRunnerError,
};
use core::fmt::{Display, Formatter};
use device_traits::{LoggableMessage, Logger};
use embedded_time::fixed_point::FixedPoint;
use strum_macros::Display;

#[derive(Default, Display, Copy, Clone)]
enum FeedForwardStateRunnerState {
    #[default]
    NoState,
    Circling,
    MovingToPoint(PlateState),
    MotionDemo,
}
#[derive(Default)]
pub struct FeedforwardStateRunner {
    state: FeedForwardStateRunnerState,
    ff_circler: FFGenContCircle,
    ff_motion_demo: FFGenMotionDemo,
    motor_controllers: [MotorController; 3],
}

struct FFDebugMsg(FeedForwardStateRunnerState, u64);

impl LoggableMessage for FFDebugMsg {}
impl Display for FFDebugMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "state: {}, t:{}", self.0, self.1)
    }
}
struct FFCmd(StateRunnerCommand);
impl LoggableMessage for FFCmd {}
impl Display for FFCmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Feed Forward Command Received: {}", self.0)
    }
}
impl RunnableState for FeedforwardStateRunner {
    fn entry<LOG: Logger>(&mut self, ctx: &mut StateRunnerContext<LOG>) {
        ctx.motor_enabler.set_enable(true);
        self.state = FeedForwardStateRunnerState::NoState;
    }
    fn update<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>,
    ) -> Result<(), StateRunnerError> {
        match ctx.command {
            NoCommand => {}
            _ => {
                ctx.logger.info(FFCmd(*ctx.command));
            }
        }
        match ctx.command {
            StateRunnerCommand::NoCommand => {}
            StateRunnerCommand::FeedForwardPlateCommand(ff_plate_state) => {
                self.state = FeedForwardStateRunnerState::MovingToPoint(*ff_plate_state);
            }
            StateRunnerCommand::FeedForwardMotorCommand(motor_command) => {
                ctx.iomanager
                    .write_all_outputs(
                        Outputs {
                            piston_state: motor_command.map(|o| (o, ControlMode::Position)),
                        },
                        ctx.telemetry_builder,
                    )
                    .map_err(StateRunnerError::IOError)?;

                self.state = FeedForwardStateRunnerState::NoState;
            }
            StateRunnerCommand::FeedForwardCircling(circling_params) => {
                self.ff_circler.update_params(circling_params);
                self.state = FeedForwardStateRunnerState::Circling;
            }
            StateRunnerCommand::DebugMotorTest(id) => {
                ctx.motor_enabler.set_enable(true);
                ctx.iomanager
                    .motor_test_motion(*id)
                    .map_err(StateRunnerError::IOError)?;
                ctx.logger
                    .debug(FFDebugMsg(self.state, ctx.call_time.integer()));
            }
            StateRunnerCommand::TrimPlateAngle => {}
            StateRunnerCommand::MotionDemo => {
                self.ff_motion_demo.reset(ctx.call_time);
                self.state = FeedForwardStateRunnerState::MotionDemo;
            }
        }
        let maybe_desired_state = match self.state {
            FeedForwardStateRunnerState::NoState => None,
            FeedForwardStateRunnerState::Circling => {
                Some(PlateState::default() + self.ff_circler.get_ff(ctx.call_time))
            }
            FeedForwardStateRunnerState::MovingToPoint(plate_state) => Some(plate_state),
            FeedForwardStateRunnerState::MotionDemo => {
                Some(self.ff_motion_demo.get_ff(ctx.call_time))
            }
        };
        if let Some(desired_state) = maybe_desired_state {
            ctx.last_plate_setpoint = desired_state;
            let motors_state = ctx
                .iomanager
                .read_motor_inputs(ctx.telemetry_builder)
                .map_err(StateRunnerError::IOError)?;
            let motors_setpoint = inverse_kinematics(&desired_state);
            let mut motor_outputs: [KinState; 3] = Default::default();
            for mot_idx in 0..MOTOR_NUM {
                let tracking_error: f32;
                (motor_outputs[mot_idx], tracking_error) = self.motor_controllers[mot_idx]
                    .calc(motors_setpoint[mot_idx], motors_state[mot_idx].0);
                ctx.telemetry_builder
                    .add_motor_control(mot_idx, tracking_error);
            }
            ctx.iomanager
                .write_all_outputs(
                    Outputs {
                        piston_state: motor_outputs.map(|o| (o, ControlMode::Velocity)),
                    },
                    ctx.telemetry_builder,
                )
                .map_err(StateRunnerError::IOError)?;
        }
        Ok(())
    }
    fn exit<LOG: Logger>(&mut self, _ctx: &mut StateRunnerContext<LOG>) {}
}
