use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
use crate::app::button_handler::ButtonHandler;
use crate::app::event_handler::EventHandler;
use crate::app::feedforward_generator::FFGen;
use embedded_time::duration::*;
use crate::app::ballpath_generator::BallpathGenerator;
use crate::app::control_primitives::{ControlInputs, KinState};
use crate::app::control_runner::ControlRunner;
use crate::app::motor_handler::{MotorHandler, MotorHandlerError};
enum LogicRunnerError {
    MotorInputGathering(MotorHandlerError)
}
struct Inputs {
   measured_plate_angle: [KinState;2],
    measured_ball_state: [KinState;2],
    measured_motors_state: [(KinState, bool);3],
}
#[derive(Default)]
struct Outputs {
    piston_state: [KinState;3],
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
        motor_handler: MotorHandler) -> Self {
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
    fn read_inputs(&mut self, call_time: Microseconds<u64>) -> Result<Inputs,LogicRunnerError>{
        self.button_handler.update(call_time-self.last_call_time);
        let measured_motors_state =self.motor_handler.get_motor_state().map_err(|e|LogicRunnerError::MotorInputGathering(e))?;
        Inputs{
            measured_plate_angle: [],
            measured_ball_state: [],
            measured_motors_state,
        }
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
    pub fn update(&mut self, call_time: Microseconds<u64>) -> Microseconds{
        let inputs = self.read_inputs(call_time);
        let outputs = self.calculate_logic(inputs);
        self.write_outputs(outputs);
        Microseconds::default()
    }
}