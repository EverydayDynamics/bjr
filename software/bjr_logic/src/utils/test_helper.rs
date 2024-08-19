#[cfg(test)]
pub mod bsp_mocks {
    use bsp_traits::{TouchSensor, Point, TouchSensorError, MotorEnabler, Logger};
    use mockall::mock;
    use core::fmt::Display;
    use crate::app::io_manager::{IOManager, IOManagerError, Inputs, Outputs};
    use embedded_time::duration::*;
    use crate::app::motor_handler::{ControlMode, MotorStatus};
    use crate::app::control_primitives::KinState;

    mock! {
        pub TouchSensor {}
        impl TouchSensor for TouchSensor {
            fn get_touch(&mut self) -> Result<Option<Point>, TouchSensorError>;
        }
    }

    mock! {
        pub TestMotorEnabler {}
        impl<'a> MotorEnabler for TestMotorEnabler {
            fn set_enable(&mut self, enable: bool);
        }
    }
    mock! {
        pub TestLogger {}
        impl<'a> Logger for TestLogger {
                fn trace(&mut self, msg: &dyn Display);
                fn debug(&mut self, msg: &dyn Display);
                fn info(&mut self, msg: &dyn Display);
                fn warn(&mut self, msg: &dyn Display);
                fn error(&mut self, msg: &dyn Display);

        }
    }
    mock! {
        pub TestIOManager {}
        impl<'a> IOManager for TestIOManager {
            fn read_all_inputs(&mut self, call_time: Microseconds<u64>) -> Result<Inputs, IOManagerError>;
            fn write_all_outputs(&mut self, outputs: Outputs) -> Result<(), IOManagerError>;
            fn read_motor_inputs(&mut self) -> Result<[(KinState, MotorStatus); 3], IOManagerError>;
            fn write_motor_outputs(&mut self, output: [Option<(KinState, ControlMode)>;3]) -> Result<(), IOManagerError>;
            fn reset_motor_pos(&mut self, motor_idx: usize) -> Result<(), IOManagerError>;
        }
    }
}
