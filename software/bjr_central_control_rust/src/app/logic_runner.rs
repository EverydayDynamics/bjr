use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
use crate::bsp::traits::Button;
use crate::app::button_handler::ButtonHandler;
use crate::app::event_handler::EventHandler;
use crate::app::feedforward_generator::FFGen;
use embedded_time::duration::*;

struct LogicRunner<'a> {
    button_handler: ButtonHandler<'a>,
    event_queue: &'static Q8<GlobEvent>,
    event_handler: EventHandler,
    ffgen: FFGen,
}
impl<'a> LogicRunner<'a> {
    pub fn new(button_handler: ButtonHandler<'a>, event_queue: &'static Q8<GlobEvent>) -> Self {
        LogicRunner{
            button_handler,
            event_queue,
            event_handler: EventHandler::new(event_queue),
            ffgen: FFGen::new(),
        }
    }
    pub fn update(&mut self, time: Microseconds) {

        let state_change = self.event_handler.handle_events();
        if let Some(new_state) = state_change {
            self.ffgen.new_state(new_state, time);
        }
        let feed_froward = self.ffgen.get_ff(time);

    }
}