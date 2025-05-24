
pub mod trimming {
    use core::fmt::{Display, Formatter};
    use crate::app::parameter_manager::{parameter_manager, ParameterType, TrimPlateAngleA, TrimPlateAngleB};
    use heapless::Deque;
    use crate::app::control_primitives::{KinState, PlateDelta, PlateState};

    #[derive(PartialEq, Copy, Clone)]
    pub enum TrimmerError {
        BufferLowError,
    }
    impl Display for TrimmerError {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            match self {
                TrimmerError::BufferLowError => {write!(f, "TrimmerBuffer not full")}
            }
        }
    }
    pub struct Trimmer<T: ParameterType, const N: usize>
    where T::ReturnType: Into<f32> + From<f32>
    {
        buffer: Deque<f32, N>,
        _marker: core::marker::PhantomData<T>,
    }

    impl<T: ParameterType, const N: usize> Trimmer<T, N>
    where T::ReturnType: Into<f32> + From<f32>
    {

        /// Creates a new `Trimmer` with an empty buffer and default trimming value.
        pub const fn new() -> Self {
            Self {
                buffer: Deque::new(),
                _marker: core::marker::PhantomData,
            }
        }

        /// Returns the current trimming value.
        pub fn get_trimming(&self) -> f32 {
            parameter_manager().get::<T>().into()
        }

        /// Adds a value to the buffer, overwriting the oldest if the buffer is full.
        pub fn load_value(&mut self, value: f32) {
            if self.buffer.is_full() {
                self.buffer.pop_front();
            }
            self.buffer.push_back(value).ok();
        }
        pub fn save_trimming(&mut self) -> Result<(), TrimmerError> {
            if self.buffer.is_full() {
                let sum:f32 = self.buffer.iter().sum();
                let trim:f32 =  sum/ (N as f32);
                parameter_manager().set::<T>(trim.into());
                Ok(())
            } else {
                Err(TrimmerError::BufferLowError)
            }
        }

        /// Clears the buffer.
        pub fn flush_buffer(&mut self) {
            self.buffer.clear();
        }
    }
    pub struct PlateTrimmer<const N: usize> {
        plate_a_trim: Trimmer<TrimPlateAngleA, N>,
        plate_b_trim: Trimmer<TrimPlateAngleB, N>
    }
    impl<const N: usize> PlateTrimmer<N>
    {

        /// Creates a new `Trimmer` with an empty buffer and default trimming value.
        pub const fn new() -> Self {
            Self {
                plate_a_trim: Trimmer::new(),
                plate_b_trim: Trimmer::new(),
            }
        }

        /// Returns the current trimming value.
        pub fn get(&self) ->PlateDelta  {
            let mut plate_trim = PlateDelta::default();
            plate_trim.angle[0].pos = self.plate_a_trim.get_trimming();
            plate_trim.angle[1].pos = self.plate_b_trim.get_trimming();
            plate_trim
        }

        pub fn load(&mut self, value: PlateState) {
            self.plate_a_trim.load_value(value.angle[0].pos);
            self.plate_b_trim.load_value(value.angle[1].pos);
        }
        pub fn save(&mut self) -> Result<(), TrimmerError> {
           self.plate_a_trim.save_trimming()?;
           self.plate_b_trim.save_trimming()

        }

        /// Clears the buffer.
        pub fn flush_buffer(&mut self) {
            self.plate_a_trim.flush_buffer();
            self.plate_b_trim.flush_buffer();
        }
    }
    // ---- TESTS ----
    #[cfg(test)]
    mod tests {
        use libm::fabsf;
        use crate::app::parameter_manager::{parameter_manager, TrimPlateAngleA};
        use crate::app::trim::trimming::{Trimmer, TrimmerError};

        #[test]
        fn test_load_and_flush() {
            let expected_trim_value = 1.0f32;
            parameter_manager().set::<TrimPlateAngleA>(expected_trim_value);
            let mut test_trimmer = Trimmer::<TrimPlateAngleA, 2>::new();
            assert_eq!(test_trimmer.get_trimming(), expected_trim_value);
            test_trimmer.load_value(10.0);
            assert!(test_trimmer.save_trimming() == Err(TrimmerError::BufferLowError));
            test_trimmer.flush_buffer();
            test_trimmer.load_value(20.0);
            assert!(test_trimmer.save_trimming() == Err(TrimmerError::BufferLowError));
            test_trimmer.load_value(10.0);
            test_trimmer.load_value(20.0);
            assert!(test_trimmer.save_trimming() == Ok(()));
            assert!(fabsf(test_trimmer.get_trimming()-15.0) < 1e-5f32);

        }
    }
}