use uom::si::f32::*;
use uom::si::ratio::ratio;

pub trait StepperDriver {
    fn get_microstepping(&self) -> f32;
}
pub struct SilentStepStick {
    microstepping: f32,
}
impl SilentStepStick {
   pub fn new() -> SilentStepStick{
       SilentStepStick {
           microstepping:8.0,
       }
   }
}
impl StepperDriver for SilentStepStick {
    fn get_microstepping(&self) -> f32 {
        self.microstepping
    }
}