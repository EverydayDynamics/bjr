use crate::app::control::control_primitives::{ControlInputs, Controller, ControlOutputs};

pub struct PIDController {

}
impl Controller for PIDController {
    fn reset(&mut self) {
        todo!()
    }

    fn update(&mut self, _inputs: ControlInputs) -> ControlOutputs {
        todo!()
    }
}
