extern crate uom;
use uom::si::f32::*;
use uom::si::frequency::hertz;
use uom::si::time::second;
use uom::num::One;
use uom::si::ratio::ratio;
use crate::captive_linear_stepper::{ LinearStepper};
use crate::stepper_state::{STEP_BASE_FREQ, STEP_THRESHOLD, STEP_VELOCITY_SCALER, StepperState};
use heapless::spsc::Producer;
use crate::stepper_driver::StepperDriver;
#[derive(Clone,Copy)]
pub struct MotorState {
    pub vel: Velocity,
    pub pos: Length,
}
pub trait Motor {
    fn get_state(&self) -> MotorState;
    fn update(&mut self, accel: Acceleration);

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
    accel_sender: Producer<'a,FrequencyDrift,2>,
    stepper_driver: STPD,
}
impl<LSTP, STPD> LinearStepperMotor<'_,LSTP, STPD>
    where LSTP: LinearStepper,
          STPD: StepperDriver
{
    pub fn new(linear_stepper: LSTP, accel_sender: Producer<FrequencyDrift,2>, stepper_driver: STPD) -> LinearStepperMotor<LSTP, STPD> {
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

    fn update(&mut self, accel: Acceleration) {
        let _ =self.accel_sender.enqueue(accel / self.linear_stepper.distance_per_step() * *self.stepper_driver.get_microstepping());
    }
}
impl<LSTP, STPD> StepperMotor for LinearStepperMotor<'_,LSTP, STPD>
    where LSTP: LinearStepper,
          STPD: StepperDriver
{
    fn update_state(&mut self, step_state: &StepperState) {
        self.motor_state = Some(MotorState {
            vel: Frequency::new::<hertz>(step_state.vel) * self.linear_stepper.distance_per_step() / *self.stepper_driver.get_microstepping(),
            pos: Ratio::new::<ratio>(step_state.pos as f32) * self.linear_stepper.distance_per_step() / *self.stepper_driver.get_microstepping()});
    }

}
