#![no_std]

use core::sync::atomic::{AtomicI32, AtomicU32, Atomic,Ordering};
use core::marker::PhantomData;
use core::default;
// Parameter types
pub struct Param1;
pub struct Param2;
pub struct LongPressThresholdMs;
// Add more parameter types as needed

// Trait to associate types with their atomic representations
pub trait ParameterType {
    type AtomicType;
    type ReturnType;
    fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType;
    fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType;
}

// Implement ParameterType for each parameter
impl ParameterType for i32 {
    type AtomicType = AtomicI32;
    type ReturnType = i32;
    fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType {
        &param_storage.param1
    }
    fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType {atomic.load(Ordering::Relaxed)}
}

impl ParameterType for u32 {
    type AtomicType = AtomicU32;
    type ReturnType = u32;
    fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType {
        &param_storage.param2
    }
    fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType {atomic.load(Ordering::Relaxed)}
}
impl ParameterType for LongPressThresholdMs {
    type AtomicType = AtomicU32;
    type ReturnType = u32;
    fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType {
        &param_storage.long_press_threshold_ms
    }
    fn atomic_load(atomic: &Self::AtomicType) -> Self::ReturnType {atomic.load(Ordering::Relaxed)}
}

//impl ParameterType for LongPressThresholdUs {
//    type AtomicType = AtomicU32;
//    fn get_atomic(param_storage: &ParameterStorage) -> &Self::AtomicType {
//        &param_storage.long_press_threshold_us
//    }
//    fn atomic_load(atomic: &Self::AtomicType) -> Self {
//        atomic.load(Ordering::Relaxed)
//    }
//}


// Storage for parameters
#[derive(Default)]
struct ParameterStorage {
    param1: AtomicI32,
    param2: AtomicU32,
    long_press_threshold_ms: AtomicU32
    // Add more parameters as needed
}

// Parameter manager

pub struct ParameterManager {
    storage: ParameterStorage,
}

impl ParameterManager {
    pub const fn new() -> Self {
        Self {
            storage: ParameterStorage{
                param1: AtomicI32::new(0),
                param2: AtomicU32::new(0),
                long_press_threshold_ms: AtomicU32::new(1000),
            }
        }
    }

    pub fn get<T: ParameterType>(&self) -> T::ReturnType {
        let atomic = T::get_atomic(&self.storage);
        T::atomic_load(atomic)
    }

    // Optional: Add a set method if needed
    pub fn set<T: ParameterType>(&self, value: T) {
        let atomic = T::get_atomic(&self.storage);
        todo!()
    }

}

// Create a static instance of the ParameterManager
static PARAMETER_MANAGER: ParameterManager = ParameterManager::new();

// Public function to access the manager
pub fn get_parameter_manager() -> &'static ParameterManager {
    &PARAMETER_MANAGER
}