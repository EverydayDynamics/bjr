use crate::app::consts::MOTOR_NUM;
use crate::app::control::feedforward_generator::FeedForwardGen;
use crate::app::control::inverse_kinematics::inverse_kinematics;
use crate::app::control::motor_controller::MotorController;
use crate::app::control::setpoint_generator::SetPointGen;
use crate::app::control_primitives::{ControlInputs, Controller, KinState, PlateState};
use crate::app::io_manager::Outputs;
use crate::app::motor_handler::ControlMode;
use crate::app::parameter_manager::{parameter_manager, NoBallTargetHeight};
use crate::app::state_runner::{
    RunnableState, StateRunnerCommand, StateRunnerContext, StateRunnerError,
};
use crate::app::trim::trimming::PlateTrimmer;
use core::default::Default;
use device_traits::Logger;

pub struct ControlStateRunner<CTRL, FFG, SPG> {
    controller: CTRL,
    ff_generator: FFG,
    sp_generator: SPG,
    motor_controllers: [MotorController; MOTOR_NUM],
    last_plate_state: PlateState,
    plate_trimmer: PlateTrimmer<5>,
    setpoint: [KinState; 2],
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
            motor_controllers: Default::default(),
            last_plate_state: PlateState::new_with_null_sa(0.010, 0.0, 0.0),
            plate_trimmer: PlateTrimmer::new(),
            setpoint: [KinState::default(); 2],
        }
    }
}
impl<CTRL, FFG, SPG> RunnableState for ControlStateRunner<CTRL, FFG, SPG>
where
    CTRL: Controller,
    FFG: FeedForwardGen,
    SPG: SetPointGen,
{
    fn entry<LOG: Logger>(&mut self, ctx: &mut StateRunnerContext<LOG>) {
        self.controller.reset(ctx.call_time);
        self.ff_generator.reset(ctx.call_time);
        self.sp_generator.reset(ctx.call_time);
        self.plate_trimmer.flush_buffer();
    }
    fn update<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>,
    ) -> Result<(), StateRunnerError> {
        let target_height_pos = parameter_manager().get::<NoBallTargetHeight>();
        let inputs = ctx
            .iomanager
            .read_all_inputs(ctx.telemetry_builder)
            .map_err(StateRunnerError::IOError)?;
        let feed_forward = self.ff_generator.get_ff(ctx.call_time);
        let target_plate_state = if let Some(ball_state) = inputs.measured_ball_state {
            self.setpoint = self.sp_generator.get_sp(ctx.call_time);
            ctx.telemetry_builder.add_setpoint_telemetry(&self.setpoint);
            self.last_plate_state = self.controller.update(
                ctx.call_time,
                ControlInputs {
                    ball_setpoint: self.setpoint.clone(),
                    measured_plate_state: PlateState {
                        height: Default::default(),
                        angle: [KinState::default(); 2],
                    },
                    measured_ball_state: ball_state,
                },
            );
            self.plate_trimmer.load(self.last_plate_state);
            self.last_plate_state
            // We have the ball, let's do some control
        } else {
            //No ball found, send the plate to noball
            PlateState::new_with_null_sa(target_height_pos, 0.0, 0.0)
        };

        let final_target = target_plate_state + feed_forward + self.plate_trimmer.get();
        ctx.last_plate_setpoint = final_target;
        let motor_setpoints = inverse_kinematics(&final_target);
        let mut motor_outputs: [KinState; MOTOR_NUM] = Default::default();
        for mot_idx in 0..MOTOR_NUM {
            let tracking_error: f32;
            (motor_outputs[mot_idx], tracking_error) = self.motor_controllers[mot_idx].calc(
                motor_setpoints[mot_idx],
                inputs.measured_motors_state[mot_idx].0,
            );
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
        match ctx.command {
            StateRunnerCommand::TrimPlateAngle => {
                self.plate_trimmer
                    .save()
                    .map_err(|e| StateRunnerError::TrimmingError(e))?;
            }
            _ => {}
        }
        Ok(())
    }
    fn exit<LOG: Logger>(&mut self, _ctx: &mut StateRunnerContext<LOG>) {}
}
