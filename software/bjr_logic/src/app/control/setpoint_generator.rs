use embedded_time::duration::Microseconds;
use crate::app::control_primitives::PlateDelta;

pub struct SetPointGenCH {

}
impl SetPointGen for SetPointGenCH {
    fn reset(&mut self, time: Microseconds<u64>) {
        todo!()
    }

    fn get_sp(&mut self, time: Microseconds<u64>) -> PlateDelta {
        todo!()
    }
}
pub trait SetPointGen{
    fn reset(&mut self, time: Microseconds<u64>);
    fn get_sp(&mut self, time: Microseconds<u64>) -> PlateDelta;
}
