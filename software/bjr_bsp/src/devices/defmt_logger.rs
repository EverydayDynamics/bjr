use device_traits::Logger;
use core::fmt::Display;
use defmt::Format;

pub struct DefmtLogger {}
impl Logger for DefmtLogger {
    fn trace<T: Format>(&mut self, message: T) {
        todo!()
    }

    fn debug<T: Format>(&mut self, message: T) {
        todo!()
    }

    fn info<T: Format>(&mut self, message: T) {
        todo!()
    }

    fn warn<T: Format>(&mut self, message: T) {
        todo!()
    }

    fn error<T: Format>(&mut self, message: T) {
        todo!()
    }
}
