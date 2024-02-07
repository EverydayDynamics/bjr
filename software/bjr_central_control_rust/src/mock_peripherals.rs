use mockall::*;
use embedded_hal::digital::v2::OutputPin;
mock! {
    pub Output {}     // Name of the mock struct, less the "Mock" prefix
    impl OutputPin for Output {   // specification of the trait to mock
        type Error = i32;
        fn set_high(&mut self) -> Result<(), <Self as embedded_hal::digital::v2::OutputPin>::Error> { todo!() }
        fn set_low(&mut self) -> Result<(), <Self as embedded_hal::digital::v2::OutputPin>::Error> { todo!() }
    }
}
