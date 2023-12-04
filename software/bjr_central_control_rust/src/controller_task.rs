use crate::stepper_state::StepperState;
use crate::actuator_num::NUM_ACTUATOR;
use crate::motor::{Motor, StepperMotor};
use crate::motor_controller::{MotorController, PDPosCtrl};
#[allow(unused_imports)]
use num_traits::real::Real;

use core::f32::consts::PI;
use crate::kinematics;
use crate::plate_state::PlateState;
use crate::motor_state::MotorState;

const phase:f32 =PI/3.0*2.0;
const AMPLITUDE:f32 =PI/180.0*7.0;
pub struct ControllerTask
{
    motor_controllers: [PDPosCtrl;NUM_ACTUATOR],
    next_runtime: u64,
}
impl ControllerTask
{
    pub fn new() -> ControllerTask {
        ControllerTask{
            motor_controllers: [
                PDPosCtrl::new( ( 10.0), (10.0)),
                PDPosCtrl::new( ( 10.0), ( 10.0)),
                PDPosCtrl::new( ( 10.0), ( 10.0)),
            ],
            next_runtime: 10000,
        }
    }
    pub fn run<MOT>(&mut self, motors: &mut [MOT; NUM_ACTUATOR])
        where MOT:Motor+StepperMotor
    {
        let speed = 8.000f32;
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
            let ctrl_output = self.motor_controllers[i].run(&motor_setpoints[i], &motors[i].get_state());
            motors[i].update(ctrl_output);
        }
    }
    pub fn next_run(&mut self) -> u64{
        self.next_runtime+=10000;
        10000
    }
}