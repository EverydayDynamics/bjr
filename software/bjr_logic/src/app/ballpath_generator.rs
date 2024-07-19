use crate::app::control_primitives::KinState;
use crate::app::event_handler::State;
use embedded_time::duration::*;
pub struct BallpathGenerator {

}
impl BallpathGenerator {
    pub fn new() -> Self {
        BallpathGenerator{}
    }
    pub fn new_state(&mut self, state: State, change_time: Microseconds) {
        todo!()
    }
    pub fn get_setpoint(&mut self, time: Microseconds) -> [KinState;2]{
        todo!()
    }
}



