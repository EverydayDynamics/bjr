use bsp_traits::{Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::control::feedforward_generator::FeedForwardGen;
use crate::app::control::inverse_kinematics::inverse_kinematics;
use crate::app::control::setpoint_generator::SetPointGen;
use crate::app::control_primitives::{ControlExecInputs, ControlInputs, Controller, KinState, PlateState};
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::{IOManager, IOManagerError, Outputs};
use crate::app::motor_handler::ControlMode;
use crate::app::parameter_manager::{NoBallTargetHeight, parameter_manager};
use crate::app::state_runner::{RunnableState, StateRunnerError};
use crate::str_to_display;
use crate::utils::DisplayStr;

pub struct ControlStateRunner<CTRL, FFG, SPG> {
    controller: CTRL,
    ff_generator: FFG,
    sp_generator: SPG,
}
impl<CTRL, FFG, SPG> ControlStateRunner<CTRL, FFG, SPG>
where CTRL: Controller,
      FFG: FeedForwardGen,
      SPG: SetPointGen,
{
    pub fn new(controller: CTRL, ff_generator: FFG, sp_generator: SPG) -> ControlStateRunner<CTRL, FFG, SPG> {
        ControlStateRunner{
            controller,
            ff_generator,
            sp_generator,
        }
    }
}
impl<CTRL, FFG, SPG> RunnableState for ControlStateRunner<CTRL, FFG, SPG>
where CTRL: Controller,
      FFG: FeedForwardGen,
      SPG: SetPointGen,
{
    fn entry(&mut self, call_time: Microseconds<u64>, _motor_enabler: &mut dyn MotorEnabler, logger: & dyn Logger) {
        self.controller.reset(call_time);
        self.ff_generator.reset(call_time);
        self.sp_generator.reset(call_time);
    }
    fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, _event_queue: EventQueue, logger: & dyn Logger) -> Result<(),StateRunnerError> {
        let inputs = iomanager.read_all_inputs(call_time).map_err(|e|StateRunnerError::IOError(e))?;
        let feed_forward = self.ff_generator.get_ff(call_time);
        let target_plate_state = if let Some(ball_state) = inputs.measured_ball_state {
            // We have the ball, let's do some control
            let setpoint = self.sp_generator.get_sp(call_time);
            self.controller.update(call_time, ControlInputs {
                ball_setpoint: setpoint,
                measured_plate_state: PlateState { height: Default::default(), angle: [KinState::default();2] },
                measured_ball_state: ball_state,
            })
        } else {
            //No ball found, send the plate to
            let target_height_pos = parameter_manager().get::<NoBallTargetHeight>();
            PlateState{ height: KinState{
                pos: target_height_pos,
                speed: 0.0,
                accel: 0.0,
            }, angle: [KinState::default();2]}
        };
         let motor_outputs = inverse_kinematics(target_plate_state+feed_forward);
        iomanager.write_all_outputs(Outputs{ piston_state: motor_outputs.map(|o|(o, ControlMode::Position)) }).map_err(|e| StateRunnerError::IOError(e))?;
        Ok(())
    }
    fn exit(&mut self, _call_time: Microseconds<u64>, logger: & dyn Logger) {}
}
