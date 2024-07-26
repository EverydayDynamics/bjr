use embedded_time::duration::Microseconds;
use crate::app::event_handler::State;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerError};


pub enum HomingStateRunnerState {
    Default,
    FastApproach,
    SlowApproach,
    GoingToSafeSpot,
}
pub struct HomingStateRunner {
    state: HomingStateRunnerState

}
impl HomingStateRunner {
    pub fn new() -> Self {
        HomingStateRunner{
            state: HomingStateRunnerState::Default,
        }
    }
}
impl RunnableState for HomingStateRunner {
    fn entry(&mut self, call_time: Microseconds<u64>) {
        self.state = HomingStateRunnerState::Default;
    }

    fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>) -> Result<(), StateRunnerError> {
        match self.state {
            HomingStateRunnerState::Default => {

            }
            HomingStateRunnerState::FastApproach => {}
            HomingStateRunnerState::SlowApproach => {}
            HomingStateRunnerState::GoingToSafeSpot => {}
        }
        Ok(())
    }

    fn exit(&mut self, call_time: Microseconds<u64>) {
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::{mock, predicate};
    use crate::app::io_manager::{Inputs, Outputs, IOManagerError};
    use crate::app::state_runner::RunnableState;
    use crate::app::control_primitives::KinState;
    use crate::app::parameter_manager::{HomingHighVelocity, HomingLowVelocity, HomingSafePosition, parameter_manager};
    use crate::app::motor_handler::ControlMode;

    mock! {
        pub TestIOManager {}
        impl<'a> IOManager for TestIOManager {
            fn read_all_inputs(&mut self, call_time: Microseconds<u64>) -> Result<Inputs, IOManagerError>;
            fn write_all_outputs(&mut self, outputs: Outputs) -> Result<(), IOManagerError>;
            fn read_motor_inputs(&mut self) -> Result<[(KinState, bool); 3], IOManagerError>;
            fn write_motor_outputs(&mut self, output: [(KinState, ControlMode);3]) -> Result<(), IOManagerError>;
        }
    }

    const TEST_HOMING_HIGH_VELOCITY:f32 = 1e3;
    const TEST_HOMING_LOW_VELOCITY:f32 = 1e2;
    const TEST_HOMING_SAFE_POSITION:f32 = 2e-3;
    #[test]

    fn test_homing_runner_homing_from_afar() {
        parameter_manager().set::<HomingHighVelocity>(TEST_HOMING_HIGH_VELOCITY);
        parameter_manager().set::<HomingLowVelocity>(TEST_HOMING_LOW_VELOCITY);
        parameter_manager().set::<HomingSafePosition>(TEST_HOMING_SAFE_POSITION);

        let mut mock_iomanager = MockTestIOManager::new();
        let mut test_homing_state_runner = HomingStateRunner::new();
        parameter_manager().set::<HomingHighVelocity>(TEST_HOMING_HIGH_VELOCITY);
        mock_iomanager.expect_read_motor_inputs()
            .returning(|| Ok([(KinState {
                pos: 0.0,
                speed: 0.0,
                accel: 0.0,
            }, false); 3]));
        mock_iomanager.expect_write_motor_outputs()
            .withf(|input|true)
            .returning(|_| Ok(()));
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(0));
        assert_eq!(result, Ok(()));

        mock_iomanager.expect_read_motor_inputs()
            .returning(|| Ok([(KinState {
                pos: 0.0,
                speed: 0.0,
                accel: 0.0,
            }, true); 3]));
    }
}
