use crate::app::control_primitives::PlateDelta;
use crate::app::event_handler::State;
use embedded_time::duration::*;
fn default(_time: Microseconds) -> PlateDelta {
    todo!()
}
pub struct FFGen {
    active_anim_func:fn(Microseconds) -> PlateDelta,
}
impl FFGen {
    pub fn new() -> Self {
        FFGen{active_anim_func: default}
    }
    pub fn new_state(&mut self, _state: State, _change_time: Microseconds) {
        self.active_anim_func = default
    }
    pub fn get_ff(&mut self, time: Microseconds) -> PlateDelta{
        (self.active_anim_func)(time)
    }
}