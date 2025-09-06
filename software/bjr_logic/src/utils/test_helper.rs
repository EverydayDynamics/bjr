#[cfg(test)]
pub mod bsp_mocks {
    use crate::app::control_primitives::KinState;
    use crate::app::io_manager::{IOManager, IOManagerError, Inputs, Outputs};
    use crate::app::motor_handler::{ControlMode, MotorStatus};
    use crate::app::telemetry_handler::TelemetryBuilder;
    use device_traits::{MotorEnabler, TouchPoint, TouchSensor, TouchSensorError};
    use mockall::mock;

    mock! {
        pub TouchSensor {}
        impl TouchSensor for TouchSensor {
            fn get_touch(&mut self) -> Result<Option<TouchPoint>, TouchSensorError>;
        }
    }

    mock! {
        pub TestMotorEnabler {}
        impl<'a> MotorEnabler for TestMotorEnabler {
            fn set_enable(&mut self, enable: bool);
        }
    }
    mock! {
        pub TestIOManager {}
        impl<'a> IOManager for TestIOManager {
            fn read_all_inputs(&mut self, telemetry_builder: &mut TelemetryBuilder) -> Result<Inputs, IOManagerError>;
            fn read_motor_inputs(&mut self, telemetry_builder: &mut TelemetryBuilder ) -> Result<[(KinState, MotorStatus); 3], IOManagerError>;
            fn write_motor_outputs(
                &mut self,
                output: [Option<(KinState, ControlMode)>; 3],
                telemetry_builder: &mut TelemetryBuilder,
            ) -> Result<(), IOManagerError>;
            fn reset_motor_pos(&mut self, motor_idx: usize) -> Result<(), IOManagerError>;
            fn write_all_outputs(&mut self, outputs: Outputs, telemetry_builder: &mut TelemetryBuilder) -> Result<(), IOManagerError>;
            fn motor_test_motion(&mut self, motor_idx: usize) -> Result<(), IOManagerError>;

        }
    }
}
