use embedded_time::duration::Microseconds;
use crate::app::control_primitives::{KinState, PlateDelta};

pub struct SetPointGenCH {

}
impl SetPointGen for SetPointGenCH {
    fn reset(&mut self, _time: Microseconds<u64>) {
    }

    fn get_sp(&mut self, _time: Microseconds<u64>) -> [KinState;2] {
        [KinState{
            pos: 0.0,
            speed: 0.0,
            accel: 0.0,
        };2]
    }
}
pub trait SetPointGen{
    fn reset(&mut self, time: Microseconds<u64>);
    fn get_sp(&mut self, time: Microseconds<u64>) -> [KinState;2];
}
