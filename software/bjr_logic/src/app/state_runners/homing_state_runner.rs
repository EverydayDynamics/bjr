use embedded_time::duration::Microseconds;
use crate::app::consts::MOTOR_NUM;
use crate::app::control_primitives::KinState;
use crate::app::event_handler::State;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::motor_handler::ControlMode;
use crate::app::parameter_manager::{HomingAccel, HomingHighVelocity, HomingLowVelocity, HomingMaxTravel, HomingSafePosition, parameter_manager};
use crate::app::state_runner::{RunnableState, StateRunnerError};


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
pub struct HomingStateRunner {
    states: [HomingStateRunnerState;MOTOR_NUM]

}
impl HomingStateRunner {
    pub fn new() -> Self {
        HomingStateRunner{
            states: [HomingStateRunnerState::Default;MOTOR_NUM],
        }
    }
}
impl RunnableState for HomingStateRunner {
    fn entry(&mut self, call_time: Microseconds<u64>) {
        self.states = [HomingStateRunnerState::Default;MOTOR_NUM];
    }

    fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue) -> Result<(), StateRunnerError> {

        let homing_high_velocity = parameter_manager().get::<HomingHighVelocity>();
        let homing_low_velocity = parameter_manager().get::<HomingLowVelocity>();
        let homing_safe_position = parameter_manager().get::<HomingSafePosition>();
        let homing_max_travel = parameter_manager().get::<HomingMaxTravel>();
        let homing_accel = parameter_manager().get::<HomingAccel>();

        let mut retval = Ok(());
        let inputs = iomanager.read_motor_inputs().map_err(|e|StateRunnerError::HomingIOError(e))?;
        let mut outputs: [Option<(KinState, ControlMode)>; 3] = [None;MOTOR_NUM];
        let mut motor_idx = 0;
        let mut done_counter = 0;
        for ((state, input), output) in self.states.iter_mut().zip(inputs.iter()).zip(outputs.iter_mut()) {
            let mut maybe_next_state:Option<HomingStateRunnerState> = None;
            if motor_idx == 0 {
                println!("");
            }
            match state {
                HomingStateRunnerState::Default => {
                    if input.1.limit_reached {
                        maybe_next_state.replace(HomingStateRunnerState::FirstGoingToSafeSpot);
                    } else {
                        maybe_next_state.replace(HomingStateRunnerState::FastApproach);
                        output.replace((KinState{
                            pos: -homing_max_travel,
                            speed: homing_high_velocity,
                            accel: homing_accel,
                        }, ControlMode::Position));
                    }
                }
                HomingStateRunnerState::FastApproach => {
                    if input.1.limit_reached {
                        //TODO implement position reset here
                        maybe_next_state.replace(HomingStateRunnerState::StoppingAfterFastApproach);
                        output.replace((KinState{
                            pos: 0.0,
                            speed: 0.0,
                            accel: homing_accel,
                        }, ControlMode::Velocity));

                    } else if input.1.position_reached {
                        // Finished the motion without hitting the limit.
                        maybe_next_state.replace(HomingStateRunnerState::Error);
                        retval =  Err(StateRunnerError::HomingOverrun);

                    }

                }

                HomingStateRunnerState::StoppingAfterFastApproach => {
                    if input.1.standstill {
                        maybe_next_state.replace(HomingStateRunnerState::FirstGoingToSafeSpot);
                        output.replace((KinState{
                            pos: homing_safe_position,
                            speed: homing_high_velocity,
                            accel: homing_accel,
                        }, ControlMode::Position));

                    }
                    //TODO find a way to timeout the stopping.
                }
                HomingStateRunnerState::SlowApproach => {
                    if input.1.limit_reached {
                        //TODO implement position reset here
                        maybe_next_state.replace(HomingStateRunnerState::StoppingAfterSlowApproach);
                        output.replace((KinState{
                            pos: 0.0,
                            speed: 0.0,
                            accel: homing_accel,
                        }, ControlMode::Velocity));

                    } else if input.1.position_reached {
                        // Finished the motion without hitting the limit.
                        maybe_next_state.replace(HomingStateRunnerState::Error);
                        retval =  Err(StateRunnerError::HomingOverrun);

                    }

                }
                HomingStateRunnerState::StoppingAfterSlowApproach => {
                    if input.1.standstill {
                        maybe_next_state.replace(HomingStateRunnerState::FinalGoingToSafeSpot);
                        output.replace((KinState{
                            pos: homing_safe_position,
                            speed: homing_high_velocity,
                            accel: homing_accel,
                        }, ControlMode::Position));

                    }
                    //TODO find a way to timeout the stopping.

                }
                HomingStateRunnerState::FirstGoingToSafeSpot => {
                    if input.1.position_reached {
                        if input.1.limit_reached {
                            // Limit is stuck on at safe spot.
                            maybe_next_state.replace(HomingStateRunnerState::Error);
                            retval =  Err(StateRunnerError::HomingLimistSWStuckAtSafePos);
                        } else {
                            maybe_next_state.replace(HomingStateRunnerState::SlowApproach);
                            output.replace((KinState{
                                pos: -homing_safe_position,
                                speed: homing_low_velocity,
                                accel: homing_accel,
                            }, ControlMode::Position));

                        }

                    } else if input.1.standstill {
                        maybe_next_state.replace(HomingStateRunnerState::Error);
                        retval =  Err(StateRunnerError::HomingUnexpectedStopGoingToSafePos);
                    }

                }
                HomingStateRunnerState::FinalGoingToSafeSpot => {
                    if input.1.position_reached {
                        if input.1.limit_reached {
                            // Limit is stuck on at safe spot.
                            maybe_next_state.replace(HomingStateRunnerState::Error);
                            retval =  Err(StateRunnerError::HomingLimistSWStuckAtSafePos);
                        } else {
                            maybe_next_state.replace(HomingStateRunnerState::Done);
                        }

                    } else if input.1.standstill {
                        maybe_next_state.replace(HomingStateRunnerState::Error);
                        retval =  Err(StateRunnerError::HomingUnexpectedStopGoingToSafePos);
                    }

                }
                HomingStateRunnerState::Done => {
                    done_counter+=1;

                }
                HomingStateRunnerState::Error => {

                    retval =  Err(StateRunnerError::HomingUnexpectedStopGoingToSafePos);
                }
            }
            if let Some(next_state) = maybe_next_state {
                *state = next_state;

            }
            motor_idx += 1;
        }
        iomanager.write_motor_outputs(outputs).map_err(|e|StateRunnerError::HomingIOError(e))?;
        if done_counter == MOTOR_NUM {
            // All motors homed.
        }
        retval
    }

    fn exit(&mut self, call_time: Microseconds<u64>) {
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use heapless::mpmc::Q8;
    use super::*;
    use mockall::predicate::*;
    use mockall::{mock, predicate};
    use crate::app::consts::MOTOR_NUM;
    use crate::app::io_manager::{Inputs, Outputs, IOManagerError};
    use crate::app::state_runner::RunnableState;
    use crate::app::control_primitives::KinState;
    use crate::app::event::GlobEvent;
    use crate::app::parameter_manager::{HomingAccel, HomingHighVelocity, HomingLowVelocity, HomingSafePosition, parameter_manager};
    use crate::app::motor_handler::{ControlMode, MotorStatus};

    mock! {
        pub TestIOManager {}
        impl<'a> IOManager for TestIOManager {
            fn read_all_inputs(&mut self, call_time: Microseconds<u64>) -> Result<Inputs, IOManagerError>;
            fn write_all_outputs(&mut self, outputs: Outputs) -> Result<(), IOManagerError>;
            fn read_motor_inputs(&mut self) -> Result<[(KinState, MotorStatus); 3], IOManagerError>;
            fn write_motor_outputs(&mut self, output: [Option<(KinState, ControlMode)>;3]) -> Result<(), IOManagerError>;
        }
    }

    const TEST_HOMING_HIGH_VELOCITY:f32 = 1e-2;
    const TEST_HOMING_LOW_VELOCITY:f32 = 1e-3;
    const TEST_HOMING_SAFE_POSITION:f32 = 2e-3;
    const TEST_HOMING_MAX_TRAVEL:f32 = 2e-2;
    const TEST_HOMING_ACCEL:f32 = 2e-3;
    fn expect_motor_output_safe_pos(mock_iomanager: &mut MockTestIOManager) {
        let expected_motor_output_safe_pos = [Some((KinState{
            pos: TEST_HOMING_SAFE_POSITION,
            speed: TEST_HOMING_HIGH_VELOCITY,
            accel: TEST_HOMING_ACCEL,
        }, ControlMode::Position));MOTOR_NUM];
        mock_iomanager.expect_write_motor_outputs()
            .with(eq(expected_motor_output_safe_pos))
            .times(1)
            .returning(|_| Ok(()));
    }

    fn expect_motor_output_stop(mock_iomanager: &mut MockTestIOManager) {
        let expected_motor_output_safe_pos = [Some((KinState{
            pos: 0.0,
            speed: 0.0,
            accel: TEST_HOMING_ACCEL,
        }, ControlMode::Velocity));MOTOR_NUM];
        mock_iomanager.expect_write_motor_outputs()
            .with(eq(expected_motor_output_safe_pos))
            .times(1)
            .returning(|_| Ok(()));
    }
    fn expect_no_motor_output(mock_iomanager: &mut MockTestIOManager) {
        let expected_motor_output_safe_pos = [None;MOTOR_NUM];
        mock_iomanager.expect_write_motor_outputs()
            .with(eq(expected_motor_output_safe_pos))
            .times(1)
            .returning(|_| Ok(()));
    }

    fn expect_motor_output_fast_approach(mock_iomanager: &mut MockTestIOManager) {
        let expected_motor_output_safe_pos = [Some((KinState{
            pos: -TEST_HOMING_MAX_TRAVEL,
            speed: TEST_HOMING_HIGH_VELOCITY,
            accel: TEST_HOMING_SAFE_POSITION,
        }, ControlMode::Position));MOTOR_NUM];
        mock_iomanager.expect_write_motor_outputs()
            .with(eq(expected_motor_output_safe_pos))
            .times(1)
            .returning(|_| Ok(()));
    }

    fn expect_motor_output_slow_approach(mock_iomanager: &mut MockTestIOManager) {
        let expected_motor_output_safe_pos = [Some((KinState{
            pos: -TEST_HOMING_SAFE_POSITION,
            speed: TEST_HOMING_LOW_VELOCITY,
            accel: TEST_HOMING_SAFE_POSITION,
        }, ControlMode::Position));MOTOR_NUM];
        mock_iomanager.expect_write_motor_outputs()
            .with(eq(expected_motor_output_safe_pos))
            .times(1)
            .returning(|_| Ok(()));
    }
    fn expect_motor_input(mock_iomanager: &mut MockTestIOManager, status: MotorStatus) {
        let expected_motor_input_none = [
            (KinState{
                pos: 0.0,
                speed: 0.0,
                accel: 0.0,
            }, status);MOTOR_NUM];
        let a = mock_iomanager.expect_read_motor_inputs()
            .with()
            .times(1)
            .returning(move ||
            Ok(expected_motor_input_none));
    }
    #[test]
    fn test_homing_runner_homing_from_afar() {
        static TEST_EVENT_QUEUE:Q8<GlobEvent> = Q8::new();
        parameter_manager().set::<HomingHighVelocity>(TEST_HOMING_HIGH_VELOCITY);
        parameter_manager().set::<HomingLowVelocity>(TEST_HOMING_LOW_VELOCITY);
        parameter_manager().set::<HomingSafePosition>(TEST_HOMING_SAFE_POSITION);
        parameter_manager().set::<HomingMaxTravel>(TEST_HOMING_MAX_TRAVEL);
        parameter_manager().set::<HomingAccel>(TEST_HOMING_ACCEL);


        let mut mock_iomanager = MockTestIOManager::new();
        let mut test_homing_state_runner = HomingStateRunner::new();
        test_homing_state_runner.entry(Microseconds(0));
        //update (1000us)
        //Check limit status -> not at limit
        // Start moving to limit fast
        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: false,
            velocity_reached: false,
            standstill: true,
        });
        expect_motor_output_fast_approach(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(10000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (2000us)
        //Check limit status -> not at limit

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(20000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (3000us)
        //Check limit status -> not at limit

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(30000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (4000us)
        //Check limit status -> at limit
        //Send stop signal
        //Send position reset

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: true,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_motor_output_stop(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(40000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));


        //update (5000us)
        //Check standstill status -> no standstill

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: true,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(50000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));


        //update (6000us)
        //Check standstill status -> standstill
        //Send moving to safe position

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: false,
            velocity_reached: false,
            standstill: true,
        });
        expect_motor_output_safe_pos(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(60000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (7000us)
        //Check position reached status -> no reached

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(70000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (8000us)
        //Check position reached status -> reached
        //Check limit -> no limit
        // --- slow approach --
        //Send slow approach

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: true,
            velocity_reached: false,
            standstill: false,
        });
        expect_motor_output_slow_approach(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(80000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (9000us)
        //Check limit -> no limit

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(90000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (10000us)
        //Check limit -> no limit

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(100000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (11000us)
        //Check limit -> limit reached
        //send stop signal

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: true,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_motor_output_stop(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(110000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (12000us)
        //Check standstill -> no standstill

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: true,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(120000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (13000us)
        //Check standstill -> standstill
        //send safe pos travel signal
        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: true,
            position_reached: false,
            velocity_reached: false,
            standstill: true,
        });
        expect_motor_output_safe_pos(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(130000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (14000us)
       //Check position reached -> not reached

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: false,
            velocity_reached: false,
            standstill: false,
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(140000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));

        //update (15000us)
        //Check position reached -> reached

        expect_motor_input(&mut mock_iomanager, MotorStatus{
            limit_reached: false,
            position_reached: true,
            velocity_reached: false,
            standstill: false
        });
        expect_no_motor_output(&mut mock_iomanager);
        let result = test_homing_state_runner.update(&mut mock_iomanager, Microseconds::new(150000),&TEST_EVENT_QUEUE);
        mock_iomanager.checkpoint();
        assert_eq!(result, Ok(()));
    }
}
