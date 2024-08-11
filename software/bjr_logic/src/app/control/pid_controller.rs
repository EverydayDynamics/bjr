use embedded_time::duration::Microseconds;
use crate::app::control_primitives::{ControlInputs, Controller, ControlOutputs};

pub struct PIDController {

}
impl Controller for PIDController {
    fn reset(&mut self, call_time: Microseconds<u64>) {
        todo!()
    }

    fn update(&mut self, call_time: Microseconds<u64>, _inputs: ControlInputs) -> ControlOutputs {
        todo!()
    }
}
