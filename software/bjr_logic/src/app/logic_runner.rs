use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
use crate::app::button_handler::ButtonHandler;
use crate::app::event_handler::EventHandler;
use crate::app::feedforward_generator::FFGen;
use embedded_time::duration::*;
use crate::app::ballpath_generator::BallpathGenerator;
use crate::app::control_runner::ControlRunner;
use crate::app::motor_handler::{MotorHandler, MotorHandlerError};
#[derive(Debug)]
enum LogicRunnerError {
    MotorInputGathering(MotorHandlerError)
}
pub struct LogicRunner<'a> {
    last_call_time: Microseconds<u64>,
    button_handler: ButtonHandler<'a>,
    event_queue: &'static Q8<GlobEvent>,
    event_handler: EventHandler,
    ffgen: FFGen,
    ballpath_generator: BallpathGenerator,
    control_runner: ControlRunner,
    motor_handler: MotorHandler<'a>,

}

impl<'a> LogicRunner<'a> {
    pub fn new(
        button_handler: ButtonHandler<'a>,
        event_queue: &'static Q8<GlobEvent>,
        motor_handler: MotorHandler<'a>) -> Self {
        LogicRunner{
            last_call_time: Default::default(),
            button_handler,
            event_queue,
            event_handler: EventHandler::new(),
            ffgen: FFGen::new(),
            ballpath_generator: BallpathGenerator::new(),
            control_runner: ControlRunner{},
            motor_handler,
        }
    }

    pub fn update(&mut self, call_time: Microseconds<u64>) -> Microseconds{
        self.button_handler.update(call_time);
        let state = self.event_handler.handle_events();

        Microseconds::default()
    }
}