use core::fmt::{Display, Formatter};

use bsp_traits::Logger;
use bsp_traits::MotorEnabler;
use embedded_time::duration::Microseconds;
use crate::app::control_primitives::KinState;
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::motor_handler::ControlMode;
use crate::app::parameter_manager::{NoBallTargetAccel, NoBallTargetPos, NoBallTargetVel, parameter_manager};
use crate::app::state_runner::{RunnableState, StateRunnerError};
use crate::str_to_display;
use crate::utils::DisplayStr;

#[derive(Default)]
enum NoBallState {
    #[default]
    Default,
    MovingToPosition,
    PositionReached,
    HasBall,
}
impl Display for NoBallState {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            NoBallState::Default => {write!(f,"Default")}
            NoBallState::MovingToPosition => {write!(f,"MovingToPosition")}
            NoBallState::PositionReached => {write!(f,"PositionReached")}
            NoBallState::HasBall => {write!(f,"HasBall")}
        }
    }
}
#[derive(Default)]
pub struct NoBallStateRunner {
    state: NoBallState,
}
impl RunnableState for NoBallStateRunner {
    fn entry(&mut self, _call_time: Microseconds<u64>, _motor_enabler: &mut dyn MotorEnabler, logger: & dyn Logger) {
        self.state = NoBallState::Default;

    }
    fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue, logger: & dyn Logger) -> Result<(),StateRunnerError> {
        let mut maybe_next_state: Option<NoBallState> = None;
        match self.state {
            NoBallState::Default => {
                let target_pos = parameter_manager().get::<NoBallTargetPos>();
                let target_vel = parameter_manager().get::<NoBallTargetVel>();
                let target_accel = parameter_manager().get::<NoBallTargetAccel>();
                let target_kinstate = KinState {
                    pos: target_pos,
                    speed: target_vel,
                    accel: target_accel,
                };
                let target_output = [Some((target_kinstate, ControlMode::Position)); 3];
                iomanager.write_motor_outputs(target_output).map_err(|e| StateRunnerError::IOError(e))?;
                maybe_next_state = Some(NoBallState::MovingToPosition);
            }
            _ => {}
        }

        let inputs = iomanager.read_all_inputs(call_time).map_err(|e|StateRunnerError::IOError(e))?;
        match self.state{
            NoBallState::HasBall => {}
            _ => {
                if inputs.measured_ball_state != None {
                    event_queue.enqueue(GlobEvent::BallFound).map_err(|e|StateRunnerError::QueueFull(e))?;
                    maybe_next_state = Some(NoBallState::HasBall);
                }
                let mut position_reached = true;
                for (_, motor_status) in inputs.measured_motors_state {
                    if !motor_status.position_reached {
                        position_reached = false;
                    }
                }
                if position_reached {
                    maybe_next_state = Some(NoBallState::PositionReached);
                }
            }
        }
        if let Some(next_state) = maybe_next_state {
            //logger.debug(&str_to_display!("changing noball state from ({}) to ({})", self.state, next_state));
            self.state = next_state;
        }
        Ok(())
    }
    fn exit(&mut self, _call_time: Microseconds<u64>, logger: & dyn Logger) {}
}
#[cfg(test)]
mod tests {
    use heapless::mpmc::Q8;
    use super::*;
    use mockall::{mock, predicate};
    use crate::app::io_manager::Inputs;
    use crate::app::motor_handler::MotorStatus;
    use crate::app::state_runner::RunnableState;
    use crate::utils::test_helper::bsp_mocks::{MockTestIOManager,MockTestLogger,MockTestMotorEnabler};
    static TEST_EVENT_QUEUE_1: Q8<GlobEvent> = Q8::new();
    #[test]
    fn test_regular_op() {

        let mut test_no_ball_state_runner = NoBallStateRunner::default();
        let mut mock_iomanager = MockTestIOManager::new();
        let mut mock_motor_enabler = MockTestMotorEnabler::new();
        let mut mock_logger = MockTestLogger::new();
        test_no_ball_state_runner.entry(Microseconds::default(), &mut mock_motor_enabler, &mock_logger);
        //first run, expect motion initialization
        mock_logger.expect_debug().returning(|_|());
        mock_iomanager.expect_write_motor_outputs().times(1).returning(|_|Ok(()));
        mock_iomanager.expect_read_all_inputs().times(1).returning(|_|Ok(Inputs{
            measured_plate_angle: [KinState::default();2],
            measured_ball_state: None,
            measured_motors_state: [(KinState::default(), MotorStatus{
                limit_reached: false,
                position_reached: false,
                velocity_reached: false,
                standstill: false,
            });3],
        }));
        let result_1 = test_no_ball_state_runner.update( &mut mock_iomanager, Microseconds::default(), &TEST_EVENT_QUEUE_1, &mock_logger);
        assert!(result_1 == Ok(()));
        assert!(TEST_EVENT_QUEUE_1.dequeue() == None);
        mock_iomanager.checkpoint();
        mock_iomanager.expect_read_all_inputs().times(1).returning(|_|Ok(Inputs{
            measured_plate_angle: [KinState::default();2],
            measured_ball_state: Some([KinState::default();2]),
            measured_motors_state: [(KinState::default(), MotorStatus{
                limit_reached: false,
                position_reached: false,
                velocity_reached: false,
                standstill: false,
            });3],
        }));
        let result_2 = test_no_ball_state_runner.update( &mut mock_iomanager, Microseconds::default(), &TEST_EVENT_QUEUE_1, &mock_logger);
        assert!(result_2 == Ok(()));
        assert!(TEST_EVENT_QUEUE_1.dequeue() == Some(GlobEvent::BallFound));
    }
}
