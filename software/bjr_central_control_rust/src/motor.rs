extern crate uom;
use uom::si::f32::*;
use uom::si::frequency::hertz;
use uom::si::time::second;
use uom::num::One;
use uom::si::ratio::ratio;
use crate::captive_linear_stepper::{ LinearStepper};
use crate::stepper_state::{STEP_BASE_FREQ, STEP_THRESHOLD, STEP_VELOCITY_SCALER, StepperState};
use heapless::spsc::Producer;
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
pub struct LinearStepperMotor<'a, LSTP>
where LSTP: LinearStepper 
{
    linear_stepper: LSTP,
    motor_state: Option<MotorState>,
    accel_sender: Producer<'a,i32,2>,
    set_accel_holder: Option<i32>,
}
impl<LSTP> LinearStepperMotor<'_,LSTP>
    where LSTP: LinearStepper
{
    pub fn new(linear_stepper: LSTP, accel_sender: Producer<i32,2>) -> LinearStepperMotor<LSTP> {
        LinearStepperMotor{ linear_stepper, motor_state: None, set_accel_holder: None, accel_sender}
    }
}
impl<LSTP> Motor for LinearStepperMotor<'_,LSTP>
where LSTP: LinearStepper
{
    fn get_state(&self) -> MotorState {
       self.motor_state.unwrap()
    }

    fn update(&mut self, accel: Acceleration) {
        let steps_per_ssq = accel/self.linear_stepper.distance_per_step();
        let dimless_accel = (steps_per_ssq*
            uom::si::f32::Time::new::<second>(STEP_VELOCITY_SCALER as f32) /
            uom::si::f32::Frequency::new::<hertz>(STEP_BASE_FREQ as f32));
        let _ =self.accel_sender.enqueue((dimless_accel.get::<ratio>())as i32);
    }
}
impl<LSTP> StepperMotor for LinearStepperMotor<'_,LSTP>
where LSTP: LinearStepper
{
    fn update_state(&mut self, step_state: &StepperState) {
        self.motor_state = Some(MotorState {
            vel: uom::si::f32::Frequency::new::<hertz>((step_state.vel as f32) / (STEP_VELOCITY_SCALER as f32)) * self.linear_stepper.distance_per_step(),
            pos: step_state.pos as f32 * self.linear_stepper.distance_per_step()});
    }

}
