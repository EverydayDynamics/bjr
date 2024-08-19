use crate::app::control_primitives::{ControlInputs, Controller, KinState, PlateState};
use embedded_time::duration::Microseconds;

pub struct PIDController {}
impl Controller for PIDController {
    fn reset(&mut self, call_time: Microseconds<u64>) {}

    fn update(&mut self, call_time: Microseconds<u64>, _inputs: ControlInputs) -> PlateState {
        PlateState {
            height: Default::default(),
            angle: [KinState::default(); 2],
        }
    }
}
