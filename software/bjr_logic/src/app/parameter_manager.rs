#![no_std]

use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use atomic_float::AtomicF32;
use std::sync::atomic::AtomicBool;
use std::cmp::PartialOrd;

pub trait ParameterType {
    type AtomicType;
    type ReturnType: PartialOrd;
    fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType;
    fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType;
}
macro_rules! generate_parameter_types {
    ($(($atomic_type:ty, $return_type:ty, $member:ident, $default:expr)),* $(,)?) => {
        $(
            pub struct $member;

            impl ParameterType for $member {
                type AtomicType = $atomic_type;
                type ReturnType = $return_type;

                fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType {
                    &param_storage.$member
                }

                fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType {
                    atomic.load(std::sync::atomic::Ordering::Relaxed)
                }
            }
        )*

        pub struct ParameterStorage {
            $(
                pub $member: $atomic_type,
            )*
        }

        impl ParameterStorage {
            const fn default() -> Self {
                Self {
                    $(
                        $member: <$atomic_type>::new($default),
                    )*
                }
            }
        }
    };
}
generate_parameter_types!(
    (AtomicF32, f32, MotorM2Ustep, 440.0),
    (AtomicU32, u32, LongPressThresholdMs, 1000),
    (AtomicF32, f32, MotorUpperPosLimit, 1e6),
    (AtomicF32, f32, MotorLowerPosLimit, -1e6),
    (AtomicF32, f32, MotorUpperVelLimit, 1e6),
    (AtomicF32, f32, MotorLowerVelLimit, -1e6),
    (AtomicBool, bool, Mute, false)
);
// Parameter manager

pub struct ParameterManager {
    storage: ParameterStorage,
}

impl ParameterManager {
    pub const fn new() -> Self {
        Self {
            storage: ParameterStorage::default()
        }
    }

    pub fn get<T: ParameterType>(&self) -> T::ReturnType {
        let atomic = T::get_atomic(&self.storage);
        T::atomic_load(atomic)
    }

    // Optional: Add a set method if needed
    pub fn set<T: ParameterType>(&self, value: T) {
        let atomic = T::get_atomic(&self.storage);
    }

}

// Create a static instance of the ParameterManager
static PARAMETER_MANAGER: ParameterManager = ParameterManager::new();

// Public function to access the manager
pub fn parameter_manager() -> &'static ParameterManager {
    &PARAMETER_MANAGER
}