#[cfg(test)]
pub mod bsp_mocks {
    use bsp_traits::{TouchSensor, Point, TouchSensorError};
    use mockall::mock;

    mock! {
        pub TouchSensor {}
        impl TouchSensor for TouchSensor {
            fn get_touch(&mut self) -> Result<Option<Point>, TouchSensorError>;
        }
    }
}
