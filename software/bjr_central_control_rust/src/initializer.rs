use crate::stepper_state::StepperState;
use crate::actuator_num::NUM_ACTUATOR;
use crate::motor::{Motor, StepperMotor};
use crate::motor_controller::{MotorController, PDPosCtrl};
use crate::motor_state::MotorState;

#[derive(Clone,Copy)]
enum HomingState {
    Unknown,
    ApproachFast,
    Retract,
    ApproachSlow,
    Homed,
}
pub struct Initializer
{
    counter: u64,
    motors_homing_tools: [(HomingState, PDPosCtrl);NUM_ACTUATOR],
}
impl Initializer
{
    pub fn new() -> Initializer {
        Initializer{ counter:0, motors_homing_tools: [(HomingState::Unknown,PDPosCtrl::new(0.0, 20.0));3] }
    }
    fn run_speed_ctrlr<MOT>(speed: f32, motor: &mut MOT, controller: &mut PDPosCtrl)
        where MOT: Motor+StepperMotor,
    {
        let setpoint = MotorState{
            pos: 0.0,
            vel: speed,
            accel: 0.0,
        };
        let state = motor.get_state();
        let accel = controller.run(&setpoint, &state);
        motor.update(accel);
    }
    pub fn run<MOT>(&mut self, motors: &mut [MOT; NUM_ACTUATOR] ) -> bool
        where MOT: Motor+StepperMotor,
    {
        let mut homed_axes = 0;
        for iter in motors.iter_mut().zip(self.motors_homing_tools.iter_mut()) {
            let (motor, (state, controller)) = iter;
            match state {
                HomingState::Unknown => {
                    if motor.get_home_state() {
                        *state = HomingState::Retract;
                    } else {
                        *state = HomingState::ApproachFast;
                    }
                }
                HomingState::ApproachFast => {
                    if motor.get_home_state() {
                        *state = HomingState::Retract;
                    } else {
                        Initializer::run_speed_ctrlr(-5.0, motor, controller);
                    }
                }
                HomingState::Retract => {
                    if !motor.get_home_state() {
                        *state = HomingState::ApproachSlow;
                    } else {
                        Initializer::run_speed_ctrlr(5.0, motor, controller);
                    }
                }
                HomingState::ApproachSlow => {
                    if motor.get_home_state() {
                        motor.reset_pos();
                        *state = HomingState::Homed;
                    } else {
                        Initializer::run_speed_ctrlr(-0.5, motor, controller);
                    }

                }
                HomingState::Homed => {
                    homed_axes +=1;
                    Initializer::run_speed_ctrlr(0.0, motor, controller);

                }
            }

        }
        homed_axes == NUM_ACTUATOR
    }

    pub fn next_run(&mut self) -> u64{
        10000
    }
}