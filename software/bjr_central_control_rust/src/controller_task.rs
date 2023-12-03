use crate::stepper_state::StepperState;
use crate::actuator_num::NUM_ACTUATOR;
use crate::motor::{Motor, StepperMotor};
use crate::motor_controller::{MotorController, PDPosCtrl};
use uom::si::velocity::millimeter_per_second;
use uom::si::length::millimeter;
use uom::si::f32::*;
use uom::si::acceleration::millimeter_per_second_squared;
use uom::si::frequency::hertz;
use uom::si::frequency_drift::hertz_per_second;
use uom::fmt::DisplayStyle::Abbreviation;
#[allow(unused_imports)]
use num_traits::real::Real;

use core::f32::consts::PI;
use crate::kinematics;
use crate::plate_state::PlateState;
use crate::motor_state::MotorState;

const phase:f32 =PI/3.0*2.0;
const AMPLITUDE:f32 =PI/180.0*5.0;
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
                PDPosCtrl::new( ( 10.0), (10.0)),
                PDPosCtrl::new( ( 10.0), ( 10.0)),
                PDPosCtrl::new( ( 10.0), ( 10.0)),
            ],
            next_runtime: 10000,
        }
    }
    pub fn update_stepper_state(&mut self, stepper_states: &[StepperState; NUM_ACTUATOR]){
        for it in self.motor.iter_mut().zip(stepper_states.iter()) {
            let (motor, step_state) = it;
            motor.update_state(step_state);
        }

    }
    pub fn run(&mut self) {
        let speed = 4.000f32;
        let time:f32 = (self.next_runtime/10000) as f32 /100.0;
        let amplitude:f32 = 5.0;
        let pstate = PlateState{
            angle_a: (time * speed).cos() *AMPLITUDE,
            angle_b: (time * speed).sin() *AMPLITUDE,
            height: 130.0-18.464-10.0,
            angle_a_speed: -(time * speed).sin() *AMPLITUDE*speed,
            angle_b_speed: (time * speed).cos() *AMPLITUDE*speed,
            height_speed: 0.0,
            angle_a_accel: -(time * speed).cos() *AMPLITUDE*speed*speed,
            angle_b_accel: -(time * speed).sin() *AMPLITUDE*speed*speed,
            height_accel: 0.0,
        };
        let motor_setpoints = kinematics::inverse(pstate);
        for i in 0..3 {
            let ctrl_output = self.motor_controllers[i].run(&motor_setpoints[i], &self.motor[i].get_state());
            self.motor[i].update(ctrl_output);
        }
    }
    pub fn next_run(&mut self) -> u64{
        self.next_runtime += 10000;
        self.next_runtime
    }
}