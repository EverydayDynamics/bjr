use embedded_time::duration::Microseconds;
use crate::app::io_manager::IOManager;

#[derive(PartialEq, Debug)]
pub enum StateRunnerError {
}

pub trait RunnableState {
    fn entry(&mut self, call_time: Microseconds<u64>);
    fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>) -> Result<(),StateRunnerError>;
    fn exit(&mut self, call_time: Microseconds<u64>);
}
