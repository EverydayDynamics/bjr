extern crate uom;

use core::sync::atomic::Ordering;
use uom::si::f32::*;
use uom::si::frequency::hertz;
use uom::si::ratio::ratio;
use crate::captive_linear_stepper::{ LinearStepper};
use crate::stepper_state::{StepperState};
use heapless::spsc::Producer;
use uom::si::frequency_drift::hertz_per_second;
use crate::stepper_driver::StepperDriver;
use crate::motor_state::MotorState;
pub trait Motor {
    fn get_state(&self) -> MotorState;
    fn update(&mut self, accel: f32);

}

pub trait StepperMotor {
    fn update_state(&mut self, step_state: &StepperState);
}
pub struct LinearStepperMotor<'a, LSTP, STPD>
where LSTP: LinearStepper,
      STPD: StepperDriver
{
    linear_stepper: LSTP,
    motor_state: Option<MotorState>,
    accel_sender: Producer<'a,f32,2>,
    stepper_driver: STPD,
}
impl<LSTP, STPD> LinearStepperMotor<'_,LSTP, STPD>
    where LSTP: LinearStepper,
          STPD: StepperDriver
{
    pub fn new(linear_stepper: LSTP, accel_sender: Producer<f32,2>, stepper_driver: STPD) -> LinearStepperMotor<LSTP, STPD> {
        LinearStepperMotor{ linear_stepper, motor_state: None,  accel_sender, stepper_driver}
    }
}
impl<LSTP, STPD> Motor for LinearStepperMotor<'_,LSTP, STPD>
    where LSTP: LinearStepper,
          STPD: StepperDriver
{
    fn get_state(&self) -> MotorState {
       self.motor_state.unwrap()
    }

    fn update(&mut self, accel: f32) {
        let accel =  accel / self.linear_stepper.distance_per_step() * self.stepper_driver.get_microstepping();
        let _ =self.accel_sender.enqueue(accel);
    }
}
impl<LSTP, STPD> StepperMotor for LinearStepperMotor<'_,LSTP, STPD>
    where LSTP: LinearStepper,
          STPD: StepperDriver
{
    fn update_state(&mut self, step_state: &StepperState) {
        let pos = step_state.pos.load(Ordering::Relaxed);
        let vel = step_state.vel.load(Ordering::Relaxed);
        self.motor_state = Some(MotorState {
            vel: vel as f32 * self.linear_stepper.distance_per_step() / self.stepper_driver.get_microstepping(),
            pos:  pos as f32 * self.linear_stepper.distance_per_step() / self.stepper_driver.get_microstepping()+self.linear_stepper.actuator_length(),
            accel: 0.0,
        });
    }

}
