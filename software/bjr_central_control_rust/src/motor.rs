extern crate uom;

use core::sync::atomic::Ordering;
use uom::si::f32::*;
use uom::si::frequency::hertz;
use uom::si::ratio::ratio;
use crate::captive_linear_stepper::{ LinearStepper};
use crate::stepper_state::{StepperState};
use heapless::spsc::Producer;
use uom::si::frequency_drift::hertz_per_second;
use crate::actuator_num::NUM_ACTUATOR;
use crate::stepper_driver::StepperDriver;
use crate::motor_state::MotorState;
pub trait Motor {
    fn get_state(&self) -> MotorState;
    fn update(&mut self, accel: f32);

}

pub trait StepperMotor {
    fn get_home_state(&self) -> bool;
    fn reset_pos(&mut self);
}
pub struct LinearStepperMotor<'a, LSTP, STPD>
where LSTP: LinearStepper,
      STPD: StepperDriver,
{
    linear_stepper: LSTP,
    accel_sender: Producer<'a,f32,2>,
    stepper_state: &'a StepperState,
    stepper_driver: STPD,
}
impl<LSTP, STPD> LinearStepperMotor<'_,LSTP, STPD>
    where LSTP: LinearStepper,
          STPD: StepperDriver,
{
    pub fn new<'a>(linear_stepper: LSTP, accel_sender: Producer<'a,f32,2>, stepper_driver: STPD, stepper_state: &'a StepperState) -> LinearStepperMotor<'a, LSTP, STPD> {
        LinearStepperMotor{ linear_stepper, accel_sender, stepper_driver, stepper_state}
    }
}
impl<LSTP, STPD> Motor for LinearStepperMotor<'_,LSTP, STPD>
    where LSTP: LinearStepper,
          STPD: StepperDriver,
{
    fn get_state(&self) -> MotorState {
        let pos = self.stepper_state.pos.load(Ordering::Relaxed);
        let vel = self.stepper_state.vel.load(Ordering::Relaxed);
        MotorState {
            vel: vel as f32 * self.linear_stepper.distance_per_step() / self.stepper_driver.get_microstepping(),
            pos:  pos as f32 * self.linear_stepper.distance_per_step() / self.stepper_driver.get_microstepping()+self.linear_stepper.actuator_length(),
            accel: 0.0,
        }
    }

    fn update(&mut self, accel: f32) {
        let accel =  accel / self.linear_stepper.distance_per_step() * self.stepper_driver.get_microstepping();
        let _ =self.accel_sender.enqueue(accel);
    }
}
impl<LSTP, STPD> StepperMotor for LinearStepperMotor<'_,LSTP, STPD>
    where LSTP: LinearStepper,
          STPD: StepperDriver,
{
    fn reset_pos(&mut self) {
        self.stepper_state.pos.store(0, Ordering::Relaxed);
    }
    fn get_home_state(&self) -> bool {
        self.stepper_driver.get_home_state()
    }

}
pub enum MotorEnum <MOTA,MOTB,MOTC>
where MOTA: Motor+StepperMotor,
      MOTB: Motor+StepperMotor,
      MOTC: Motor+StepperMotor,
{
    MotorA(MOTA),
    MotorB(MOTB),
    MotorC(MOTC),
}
impl<MOTA,MOTB,MOTC> Motor for MotorEnum<MOTA, MOTB, MOTC>
    where MOTA: Motor+StepperMotor,
          MOTB: Motor+StepperMotor,
          MOTC: Motor+StepperMotor,
{
    fn get_state(&self) -> MotorState {
        match self {
            MotorEnum::MotorA(mota) => mota.get_state(),
            MotorEnum::MotorB(motb) => motb.get_state(),
            MotorEnum::MotorC(motc) => motc.get_state(),
        }
    }

    fn update(&mut self, accel: f32) {
        match self {
            MotorEnum::MotorA(mota) => mota.update(accel),
            MotorEnum::MotorB(motb) => motb.update(accel),
            MotorEnum::MotorC(motc) => motc.update(accel),
        }
    }
}
impl<MOTA,MOTB,MOTC> StepperMotor for MotorEnum<MOTA, MOTB, MOTC>
    where MOTA: Motor+StepperMotor,
          MOTB: Motor+StepperMotor,
          MOTC: Motor+StepperMotor,
{

    fn get_home_state(&self) -> bool {
        match self {
            MotorEnum::MotorA(mota) => mota.get_home_state(),
            MotorEnum::MotorB(motb) => motb.get_home_state(),
            MotorEnum::MotorC(motc) => motc.get_home_state(),
        }
    }

    fn reset_pos(&mut self) {
        match self {
            MotorEnum::MotorA(mota) => mota.reset_pos(),
            MotorEnum::MotorB(motb) => motb.reset_pos(),
            MotorEnum::MotorC(motc) => motc.reset_pos(),
        }
    }
}
