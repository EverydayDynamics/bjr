use bsp_traits::Logger;
use core::fmt::{Display, Write};
use rtt_target;
use rtt_target::UpChannel;

pub struct RttLogger {
    up_channel: UpChannel,
}
impl RttLogger {
    pub fn new(up_channel: UpChannel) -> RttLogger {
        RttLogger { up_channel }
    }
}
impl Logger for RttLogger {
    fn trace(&mut self, message: &dyn Display) {
        writeln!(&mut self.up_channel, "Trace: {}", message).unwrap();
    }

    fn debug(&mut self, message: &dyn Display) {
        writeln!(&mut self.up_channel, "Debug: {}", message).unwrap();
    }

    fn info(&mut self, message: &dyn Display) {
        writeln!(&mut self.up_channel, "Info: {}", message).unwrap();
    }

    fn warn(&mut self, message: &dyn Display) {
        writeln!(&mut self.up_channel, "Warning: {}", message).unwrap();
    }

    fn error(&mut self, message: &dyn Display) {
        writeln!(&mut self.up_channel, "Error: {}", message).unwrap();
    }
}
