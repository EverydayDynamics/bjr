use crate::stepper_state::StepperState;
use log;
use crate::actuator_num::NUM_ACTUATOR;
use crate::motor;
use crate::motor::{Motor, StepperMotor};
use uom::si::velocity::millimeter_per_second;
use uom::si::length::millimeter;
use uom::si::f32::*;
use uom::si::acceleration::millimeter_per_second_squared;
use uom::fmt::DisplayStyle::Abbreviation;
pub struct ControllerTask<MOT>
    where MOT:Motor+StepperMotor
{
    motor: [MOT;NUM_ACTUATOR],
    next_runtime: u64,
}
impl<MOT> ControllerTask<MOT>
    where MOT:Motor+StepperMotor
{
    pub fn new(motors: [MOT;NUM_ACTUATOR], ) -> ControllerTask<MOT> {
        ControllerTask{
            motor: motors,
            next_runtime: 10,
        }
    }
    pub fn update_stepper_state(&mut self, stepper_states: &[StepperState; NUM_ACTUATOR]){
        for it in self.motor.iter_mut().zip(stepper_states.iter()) {
            let (motor, step_state) = it;
            motor.update_state(step_state);
        }

    }
    pub fn run(&mut self) {
        log::debug!("p:{},v:{}",
            self.motor[0].get_state().pos.into_format_args(millimeter, Abbreviation),
            self.motor[0].get_state().vel.into_format_args(millimeter_per_second, Abbreviation));
        self.motor[0].update(uom::si::f32::Acceleration::new::<millimeter_per_second_squared>(10.0))
    }
    pub fn next_run(&mut self) -> u64{
        self.next_runtime += 10;
        self.next_runtime
    }
}