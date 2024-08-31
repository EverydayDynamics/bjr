use bsp_traits::{LoggableMessage, Logger};
use core::fmt::{Display, Write};
use rtt_target;
use rtt_target::UpChannel;
use ufmt;
use ufmt::uwriteln;

pub struct RttLogger {
    up_channel: UpChannel,
}
impl RttLogger {
    pub fn new(up_channel: UpChannel) -> RttLogger {
        RttLogger { up_channel }
    }
}
impl Logger for RttLogger {
    fn trace<MSG: LoggableMessage>(&mut self, message: MSG) {
        writeln!(&mut self.up_channel, "Trace: {}", message).unwrap();
    }

    fn debug<MSG: LoggableMessage>(&mut self, message: MSG) {
        writeln!(&mut self.up_channel, "Debug: {}", message).unwrap();
    }

    fn info<MSG: LoggableMessage>(&mut self, message: MSG) {
        writeln!(&mut self.up_channel, "info: {}", message).unwrap();
    }

    fn warn<MSG: LoggableMessage>(&mut self, message: MSG) {
        writeln!(&mut self.up_channel, "Warn: {}", message).unwrap();
    }

    fn error<MSG: LoggableMessage>(&mut self, message: MSG) {
        writeln!(&mut self.up_channel, "Error: {}", message).unwrap();
    }
}