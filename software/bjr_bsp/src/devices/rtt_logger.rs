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
    fn trace<T: Display>(&mut self, message: T) {
        writeln!(&mut self.up_channel, "Trace: {}", message).unwrap();
    }

    fn debug<T: Display>(&mut self, message: T) {
        writeln!(&mut self.up_channel, "Debug: {}", message).unwrap();
    }

    fn info<T: Display>(&mut self, message: T) {
        writeln!(&mut self.up_channel, "info: {}", message).unwrap();
    }

    fn warn<T: Display>(&mut self, message: T) {
        writeln!(&mut self.up_channel, "Warn: {}", message).unwrap();
    }

    fn error<T: Display>(&mut self, message: T) {
        writeln!(&mut self.up_channel, "Error: {}", message).unwrap();
    }
}
