use bsp_traits::Logger;
use core::fmt::Display;

pub struct DefmtLogger {}
impl Logger for DefmtLogger {
    fn trace(&mut self, message: &dyn Display) {}

    fn debug(&mut self, message: &dyn Display) {}

    fn info(&mut self, message: &dyn Display) {}

    fn warn(&mut self, message: &dyn Display) {}

    fn error(&mut self, message: &dyn Display) {}
}
