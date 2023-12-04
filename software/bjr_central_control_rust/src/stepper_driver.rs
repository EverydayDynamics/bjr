use uom::si::f32::*;
use uom::si::ratio::ratio;
use embedded_hal::digital::v2::InputPin;

pub trait StepperDriver {
    fn get_microstepping(&self) -> f32;
    fn get_home_state(&self) -> bool;
}
pub struct SilentStepStick<DPIN> 
where DPIN: InputPin,
{
    microstepping: f32,
    diag_pin: DPIN
}
impl<DPIN> SilentStepStick<DPIN>
where DPIN: InputPin,
{
   pub fn new(diag_pin: DPIN) -> SilentStepStick<DPIN> {
       SilentStepStick {
           microstepping:8.0,
           diag_pin,
       }
   }
}
impl<DPIN> StepperDriver for SilentStepStick<DPIN> 
where DPIN: InputPin,
{
    fn get_microstepping(&self) -> f32 {
        self.microstepping
    }

    fn get_home_state(&self) -> bool {
        self.diag_pin.is_high().ok().unwrap()
    }
}