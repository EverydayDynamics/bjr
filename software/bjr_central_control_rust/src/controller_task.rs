use crate::stepper_state::StepperState;
use log;
use crate::actuator_num::NUM_ACTUATOR;
use crate::motor;
use crate::motor::{Motor, MotorState, StepperMotor};
use crate::motor_controller::{MotorController, PDPosCtrl};
use uom::si::velocity::millimeter_per_second;
use uom::si::length::millimeter;
use uom::si::f32::*;
use uom::si::acceleration::millimeter_per_second_squared;
use uom::fmt::DisplayStyle::Abbreviation;
use uom::si::frequency::hertz;
use uom::si::frequency_drift::hertz_per_second;
#[allow(unused_imports)]
use num_traits::real::Real;
pub struct ControllerTask<MOT>
    where MOT:Motor+StepperMotor
{
    motor: [MOT;NUM_ACTUATOR],
    motor_controllers: [PDPosCtrl;NUM_ACTUATOR],
    next_runtime: u64,
}
impl<MOT> ControllerTask<MOT>
    where MOT:Motor+StepperMotor
{
    pub fn new(motors: [MOT;NUM_ACTUATOR], ) -> ControllerTask<MOT> {
        ControllerTask{
            motor: motors,
            motor_controllers: [
                PDPosCtrl::new( FrequencyDrift::new::<hertz_per_second>( 20.0), Frequency::new::<hertz>(20.0)),
                PDPosCtrl::new( FrequencyDrift::new::<hertz_per_second>( 0.0), Frequency::new::<hertz>( 0.0)),
                PDPosCtrl::new( FrequencyDrift::new::<hertz_per_second>( 0.0), Frequency::new::<hertz>( 0.0)),
            ],
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
        let setpoint: MotorState = MotorState{ vel: Velocity::new::<millimeter_per_second>((self.next_runtime as f32 / 500.0f32).cos() *5.0*2.0), pos: Length::new::<millimeter>((self.next_runtime as f32 / 500.0f32).sin() *5.0) };
        //let setpoint: MotorState = MotorState{ vel: Velocity::new::<millimeter_per_second>(10.0), pos: Length::new::<millimeter>(0.0) };
        let ctrl_output = self.motor_controllers[0].run(&setpoint,
                                      &self.motor[0].get_state(),
                                      &Acceleration::new::<millimeter_per_second_squared>(0.0));
        //log::debug!("p:{},v:{}, a:{}",
        //    self.motor[0].get_state().pos.into_format_args(millimeter, Abbreviation),
        //    self.motor[0].get_state().vel.into_format_args(millimeter_per_second, Abbreviation),
        //    ctrl_output.into_format_args(millimeter_per_second_squared, Abbreviation));
        self.motor[0].update(ctrl_output);
    }
    pub fn next_run(&mut self) -> u64{
        self.next_runtime += 10;
        self.next_runtime
    }
}