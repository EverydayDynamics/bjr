use bsp_traits::{Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::control::ControlExecutor;
use crate::app::control::feedforward_generator::FeedForwardGen;
use crate::app::control::setpoint_generator::SetPointGen;
use crate::app::control_primitives::Controller;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerError};

pub struct ControlStateRunner<CTRL, FFG, SPG> {
    executor: ControlExecutor<CTRL, FFG, SPG>,
}
impl<CTRL, FFG, SPG> ControlStateRunner<CTRL, FFG, SPG>
where CTRL: Controller,
      FFG: FeedForwardGen,
      SPG: SetPointGen,
{
    pub fn new(executor: ControlExecutor<CTRL, FFG, SPG>) -> ControlStateRunner<CTRL, FFG, SPG> {
        ControlStateRunner{ executor }
    }
}
impl<CTRL, FFG, SPG> RunnableState for ControlStateRunner<CTRL, FFG, SPG>
where CTRL: Controller,
      FFG: FeedForwardGen,
      SPG: SetPointGen,
{
    fn entry(&mut self, call_time: Microseconds<u64>, _motor_enabler: &mut dyn MotorEnabler, logger: & dyn Logger) {
        self.executor.reset(call_time);
    }
    fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, _event_queue: EventQueue, logger: & dyn Logger) -> Result<(),StateRunnerError> {
        let inputs = iomanager.read_all_inputs(call_time).map_err(|e|StateRunnerError::IOError(e))?;
        if inputs.measured_ball_state == None {
            //No ball found

        } else {
            // We have the ball

        }
        Ok(())
    }
    fn exit(&mut self, _call_time: Microseconds<u64>, logger: & dyn Logger) {}
}
