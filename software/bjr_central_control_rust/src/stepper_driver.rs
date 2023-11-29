use uom::si::f32::*;
use uom::si::ratio::ratio;

pub trait StepperDriver {
    fn get_microstepping(&self) -> &Ratio;
}
pub struct SilentStepStick {
    microstepping: Ratio,
}
impl SilentStepStick {
   pub fn new() -> SilentStepStick{
       SilentStepStick {
           microstepping:Ratio::new::<ratio>(16.0),
       }
   }
}
impl StepperDriver for SilentStepStick {
    fn get_microstepping(&self) -> &Ratio {
        &self.microstepping
    }
}