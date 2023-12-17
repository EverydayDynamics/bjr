use crate::stepper_state::StepperState;
use crate::actuator_num::NUM_ACTUATOR;
use crate::motor::{Motor, StepperMotor};
use crate::motor_controller::{MotorController, PDPosCtrl};
#[allow(unused_imports)]
use num_traits::real::Real;

use core::f32::consts::PI;
use core::fmt::Debug;
use embedded_hal::blocking::i2c::{Write, WriteRead};
use crate::kinematics;
use crate::plate_state::PlateState;
use crate::motor_state::MotorState;
use crate::plate_angle_sensor::PlateAngleSensor;

const PHASE:f32 =PI/3.0*2.0;
const AMPLITUDE:f32 =PI/180.0*7.0;

const PERIOD_MICROS:u64= 10000;
pub struct ControllerTask
{
    motor_controllers: [PDPosCtrl;NUM_ACTUATOR],
    next_runtime: u64,
    roll: f32,
    pitch: f32,
}
impl ControllerTask
{
    pub fn new() -> ControllerTask {
        ControllerTask{
            motor_controllers: [
                PDPosCtrl::new( ( 10.0), (20.0)),
                PDPosCtrl::new( ( 10.0), ( 20.0)),
                PDPosCtrl::new( ( 10.0), ( 20.0)),
            ],
            next_runtime: 10000,
            roll: 0.0,
            pitch:0.0,
        }
    }
    pub fn run<MOT, I2C, E>(&mut self, motors: &mut [MOT; NUM_ACTUATOR], plate_sensor: &mut PlateAngleSensor<I2C>, time_tracker: &mut u64)
        where MOT:Motor+StepperMotor,
              I2C: Write<Error = E> + WriteRead<Error = E>, E: Debug,
    {
        if let Ok((angles, rates)) = plate_sensor.update(*time_tracker) {
            self.roll = angles.x;
            self.pitch = angles.y;
        }
        log::info!("roll:{}, pitch:{}", self.roll, self.pitch);

        const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;
        let speed = 4.000f32;
        let time:f32 = (self.next_runtime/10000) as f32 /100.0;
        let amplitude:f32 = 5.0;
        let pstate = PlateState{
            angle_a:-self.pitch*20.0 * DEG_TO_RAD, //(time * speed).cos() *AMPLITUDE,
            angle_b:-self.roll*20.0 * DEG_TO_RAD, // (time * speed).sin() *AMPLITUDE,
            height: 130.0-18.464-10.0,
            angle_a_speed: 0.0, //-(time * speed).sin() *AMPLITUDE*speed,
            angle_b_speed: 0.0, //(time * speed).cos() *AMPLITUDE*speed,
            height_speed: 0.0,
            angle_a_accel: 0.0, //-(time * speed).cos() *AMPLITUDE*speed*speed,
            angle_b_accel: 0.0, //-(time * speed).sin() *AMPLITUDE*speed*speed,
            height_accel: 0.0,
        };
        let motor_setpoints = kinematics::inverse(pstate);
        for i in 0..3 {
            let ctrl_output = self.motor_controllers[i].run(&motor_setpoints[i], &motors[i].get_state());
            motors[i].update(ctrl_output);
        }
    }
    pub fn next_run(&mut self) -> u64{
        self.next_runtime+=PERIOD_MICROS;
        PERIOD_MICROS
    }
}