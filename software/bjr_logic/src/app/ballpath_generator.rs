use crate::app::control_primitives::KinState;
use embedded_time::duration::*;
pub struct BallpathGenerator {

}
impl Default for BallpathGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl BallpathGenerator {
    pub fn new() -> Self {
        BallpathGenerator{}
    }
    pub fn get_setpoint(&mut self, _time: Microseconds) -> [KinState;2]{
        todo!()
    }
}



