use core::str::FromStr;
use core::sync::atomic::{AtomicI32, AtomicU32};
use atomic_float::AtomicF32;
use strum_macros::{Display, EnumIter, EnumString, EnumTable, VariantNames};
use core::fmt::{Display, Write};

struct ByteWriter<'a>(&'a mut [u8]);

impl<'a> Write for ByteWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let len = bytes.len().min(self.0.len());
        self.0[..len].copy_from_slice(&bytes[..len]);
        Ok(())
    }
}
pub trait ParameterType {
    type AtomicType;
    type ReturnType: PartialOrd;

    fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType;
    fn atomic_store(value: <Self as ParameterType>::ReturnType, atomic: &Self::AtomicType);
    fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType;
}

pub enum ParameterParseError {
    InvalidToken,
    WriteInterfaceError,
}
macro_rules! generate_parameter_types {
    ($(($atomic_type:ty, $return_type:ty, $member:ident, $default:expr)),* $(,)?) => {
        $(
            #[derive(Default)]
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
            impl FromStr for $member {
                type Err= ParameterParseError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    if stringify!($member).to_ascii_lowercase() == s.to_ascii_lowercase() {
                        Ok($member{})
                    } else {
                        Err(ParameterParseError::InvalidToken)
                    }
                }
            }
        )*
        #[derive(EnumString, EnumIter, Display)]
        pub enum ParameterList{
            $(
                #[strum(ascii_case_insensitive)]
                $member,
            )*

        }
        impl ParameterList {
            pub fn write_value<'a>(&self, writer: &mut dyn Write) -> Result<(),ParameterParseError>{
                match self{
                $(
                    ParameterList::$member => {write!(writer,"{}", PARAMETER_MANAGER.get::<$member>()).map_err(|_| ParameterParseError::WriteInterfaceError)?;},
                )*
                }
                Ok(())
            }
            pub fn set_value(&self, value_str: &str) -> Result<(),ParameterParseError>{
                match self{
                $(
                    ParameterList::$member => {PARAMETER_MANAGER.set::<$member>(<$member as ParameterType>::ReturnType::from_str(value_str).map_err(|_| ParameterParseError::InvalidToken)?)},
                )*
                }
                Ok(())
            }


        }
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
    (AtomicF32, f32, MotorM2Ustep, 8e5),
    (AtomicU32, u32, LongPressThresholdMs, 1000),
    (AtomicF32, f32, MotorUpperPosLimit, 1e6),
    (AtomicF32, f32, MotorLowerPosLimit, -1e6),
    (AtomicF32, f32, MotorUpperVelLimit, 1e6),
    (AtomicF32, f32, MotorLowerVelLimit, -1e6),
    //homing params
    (AtomicF32, f32, HomingHighVelocity, 20e-3),
    (AtomicF32, f32, HomingLowVelocity, 10e-3),
    (AtomicF32, f32, HomingSafePosition, 2e-3),
    (AtomicF32, f32, HomingMaxTravel, 20e-3),
    (AtomicF32, f32, HomingAccel, 5e-3),
    //running period
    (AtomicU32, u32, LogicRunnerPeriodUs, 10000),
    //Touch sensor parameters
    (AtomicI32, i32, TouchCenterOffsetX, -2048),
    (AtomicI32, i32, TouchCenterOffsetY, -2048),
    (AtomicF32, f32, TouchScaleY, 0.5180664e-4),
    (AtomicF32, f32, TouchScaleX, 0.388671875e-4),
    //No ball parameters
    (AtomicF32, f32, NoBallTargetHeight, 100e-3),

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