use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
use crate::app::button_handler::ButtonHandler;
use crate::app::event_handler::EventHandler;
use crate::app::feedforward_generator::FFGen;
use embedded_time::duration::*;
use crate::app::ballpath_generator::BallpathGenerator;
use crate::app::control_primitives::{ControlInputs, ControlOutputs, KinState};
use crate::app::control_runner::ControlRunner;

struct Inputs {
   measured_plate_angle: [KinState;2],
    measured_ball_state: [KinState;2],
}
#[derive(Default)]
struct Outputs {
    piston_state: [KinState;3],
}
pub struct LogicRunner<'a> {
    button_handler: ButtonHandler<'a>,
    event_queue: &'static Q8<GlobEvent>,
    event_handler: EventHandler,
    ffgen: FFGen,
    ballpath_generator: BallpathGenerator,
    control_runner: ControlRunner,
}

impl<'a> LogicRunner<'a> {
    pub fn new(button_handler: ButtonHandler<'a>, event_queue: &'static Q8<GlobEvent>) -> Self {
        LogicRunner{
            button_handler,
            event_queue,
            event_handler: EventHandler::new(),
            ffgen: FFGen::new(),
            ballpath_generator: BallpathGenerator::new(),
            control_runner: ControlRunner{},
        }
    }
    fn read_inputs(&mut self) -> Inputs{
        todo!()
    }
    fn write_outputs(&mut self, outputs: Outputs){
        todo!()
    }

    fn calculate_logic(&mut self, inputs: Inputs) -> Outputs{
        let time = Microseconds::default();
        let state_change = self.event_handler.handle_events();
        if let Some(new_state) = state_change {
            self.ffgen.new_state(new_state, time);
            self.ballpath_generator.new_state(new_state, time);
            self.control_runner.new_state(new_state);
        }
        let control_inputs = ControlInputs{
            measured_plate_angle: inputs.measured_plate_angle,
            feed_forward: self.ffgen.get_ff(time),
            ball_setpoint: self.ballpath_generator.get_setpoint(time),
            measured_ball_state: inputs.measured_ball_state,
        };
        let control_output = self.control_runner.run(control_inputs);
        Outputs::default()
    }
    pub fn update(&mut self) -> Microseconds{
        let inputs = self.read_inputs();
        let outputs = self.calculate_logic(inputs);
        self.write_outputs(outputs);
        return Microseconds::default();
    }
}