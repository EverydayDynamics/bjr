use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
use crate::bsp::traits::Button;
use crate::app::button_handler::ButtonHandler;

struct LogicRunner<'a> {
    button_handler: ButtonHandler<'a>,
    event_queue: &'static Q8<GlobEvent>
}
impl<'a> LogicRunner<'a> {
    pub fn new(button_handler: ButtonHandler<'a>, event_queue: &'static Q8<GlobEvent>) -> Self {
        LogicRunner{ button_handler , event_queue}
    }
    pub fn update() {

    }
}