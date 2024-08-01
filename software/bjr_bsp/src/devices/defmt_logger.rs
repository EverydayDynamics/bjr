use core::fmt::Display;
use bsp_traits::Logger;

pub struct DefmtLogger {

}
impl Logger for DefmtLogger {
    fn trace(&self, message: &dyn Display) {
        defmt::trace!("{}", defmt::Display2Format(message));
    }

    fn debug(&self, message: &dyn Display) {
        defmt::debug!("{}", defmt::Display2Format(message));
    }

    fn info(&self, message: &dyn Display) {
        defmt::info!("{}", defmt::Display2Format(message));
    }

    fn warn(&self, message: &dyn Display) {
        defmt::warn!("{}", defmt::Display2Format(message));
    }

    fn error(&self, message: &dyn Display) {
        defmt::error!("{}", defmt::Display2Format(message));
    }
}