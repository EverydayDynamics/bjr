use core::sync::atomic::{AtomicI32, AtomicU32};
use atomic_float::AtomicF32;

pub trait ParameterType {
    type AtomicType;
    type ReturnType: PartialOrd;

    fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType;
    fn atomic_store(value: <Self as ParameterType>::ReturnType, atomic: &Self::AtomicType);
    fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType;
}
macro_rules! generate_parameter_types {
    ($(($atomic_type:ty, $return_type:ty, $member:ident, $default:expr)),* $(,)?) => {
        $(
            pub struct $member;

            impl ParameterType for $member {
                type AtomicType = $atomic_type;
                type ReturnType = $return_type;


                fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType{
                    &param_storage.$member
                }
                fn atomic_store(value: <Self as ParameterType>::ReturnType, atomic: &Self::AtomicType) {
                    atomic.store(value, core::sync::atomic::Ordering::Relaxed);
                }

                fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType {
                    atomic.load(core::sync::atomic::Ordering::Relaxed)
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
    //homing params
    (AtomicF32, f32, HomingHighVelocity, 1e1),
    (AtomicF32, f32, HomingLowVelocity, 1e0),
    (AtomicF32, f32, HomingSafePosition, 1e2),
    (AtomicF32, f32, HomingMaxTravel, 2e4),

    (AtomicU32, u32, LogicRunnerPeriodUs, 10000),
    (AtomicF32, f32, HomingAccel, 2e3),
    (AtomicI32, i32, TouchCenterOffsetX, -1024),
    (AtomicI32, i32, TouchCenterOffsetY, -1024),
    (AtomicF32, f32, TouchScaleY, 0.10361328e-3),
    (AtomicF32, f32, TouchScaleX, 0.77734375e-4),
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
    pub fn set<T: ParameterType>(&self, value: T::ReturnType) {
        let atomic = T::get_atomic(&self.storage);
        T::atomic_store(value, atomic)
    }

}

// Create a static instance of the ParameterManager
static PARAMETER_MANAGER: ParameterManager = ParameterManager::new();

// Public function to access the manager
pub fn parameter_manager() -> &'static ParameterManager {
    &PARAMETER_MANAGER
}