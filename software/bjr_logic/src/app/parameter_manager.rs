use atomic_float::AtomicF32;
use core::f32::consts::{FRAC_PI_2, PI};
use core::fmt::Write;
use core::str::FromStr;
use core::sync::atomic::{AtomicI32, AtomicU32};
use strum_macros::{Display, EnumIter, EnumString};

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
       #[allow(non_snake_case)]
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

const _BASE_CIRCLE_RADIUS: f32 = 40.0;
const _HINGE_OFFSET: f32 = 6.5;
const _JOINT_CIRCLE_RADIUS: f32 = 17.5; //
generate_parameter_types!(
    (AtomicF32, f32, MotorM2Ustep, 8e5),
    (AtomicU32, u32, LongPressThresholdMs, 1000),
    (AtomicU32, u32, DoublePressThresholdMs, 500),
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
    (AtomicU32, u32, LogicRunnerPeriodUs, 3000),
    //Touch sensor parameters
    (AtomicI32, i32, TouchCenterOffsetX, -2000),
    (AtomicI32, i32, TouchCenterOffsetY, -2000),
    //straight
    //(AtomicF32, f32, TouchScaleXX, 3.036437e-5),
    //(AtomicF32, f32, TouchScaleXY, 0.0),
    //(AtomicF32, f32, TouchScaleYX, 0.0),
    //(AtomicF32, f32, TouchScaleYY, -3.881987e-5),
    //angled
    (AtomicF32, f32, TouchScaleXX, -1.51821850e-05),
    (AtomicF32, f32, TouchScaleXY, -2.62963158e-05),
    (AtomicF32, f32, TouchScaleYX, -2.62963158e-05),
    (AtomicF32, f32, TouchScaleYY, 1.51821850e-05),

    //(AtomicI32, i32, TouchCenterOffsetX, 0),
    //(AtomicI32, i32, TouchCenterOffsetY, 0),
    //(AtomicF32, f32, TouchScaleXX, 1.0),
    //(AtomicF32, f32, TouchScaleXY, 0.0),
    //(AtomicF32, f32, TouchScaleYX, 0.0),
    //(AtomicF32, f32, TouchScaleYY, 1.0),
    //No ball parameters
    (AtomicF32, f32, NoBallTargetHeight, 10e-3),
    //Kinematics parameters
    (AtomicF32, f32, KinBaseCircRad, 40.0e-3),
    (AtomicF32, f32, KinHingeOffset, 6.5e-3),
    (AtomicF32, f32, KinJointCircRad, 23.351e-3),
    (AtomicF32, f32, KinMinHeight, 89.902e-3),
    (AtomicF32, f32, KinSurface2JointCircDist, 27.96261e-3),
    //piston parameters
    (AtomicF32, f32, PistonBodyLen, 91.2e-3),
    //feedforward mode parameters
    (AtomicF32, f32, FFDefaultLinSpeed, 0.0),
    (AtomicF32, f32, FFDefaultLinAccel, 20e-3),
    (AtomicF32, f32, FFDefaultAngSpeed, 0.0),
    (AtomicF32, f32, FFDefaultAngAccel, 0.0),
    //MotorController
    (AtomicF32, f32, MCKp, 2.75e1),
    (AtomicF32, f32, MCAccel, 5e-3),
    (AtomicF32, f32, MCVelMax, 40e-3),
    (AtomicF32, f32, MCVelMin, -40e-3),
    (AtomicF32, f32, MCPosDeadBand, 1e-5),
    //PlatePDController
    (AtomicF32, f32, PlatePDCtrlKp, 12.0),
    (AtomicF32, f32, PlatePDCtrlKd, 5.0),
    //trimming parameters
    (AtomicF32, f32, TrimPlateAngleA, 0.0),
    (AtomicF32, f32, TrimPlateAngleB, 0.0),
    //circling setpoint generator
    (AtomicF32, f32, SPCirclingRadius, 20e-3),
    (AtomicF32, f32, SPCirclingTime, 3.0),
    //2Point setpoint generator
    (AtomicF32, f32, SP2Point1X, 0.02),
    (AtomicF32, f32, SP2Point1Y, 0.0),
    (AtomicF32, f32, SP2Point2X, -0.02),
    (AtomicF32, f32, SP2Point2Y, 0.0),
    (AtomicF32, f32, SP2PointPathDuration, 1.0),
    (AtomicF32, f32, SP2PointDwellTime, 2.0),
    //3Point setpoint generator
    (AtomicF32, f32, SP3Point1X, 0.00),
    (AtomicF32, f32, SP3Point1Y, -0.02),
    (AtomicF32, f32, SP3Point2X, 0.02*(0.86602540378)/*libm::cosf(-PI/6.0)*/),
    (AtomicF32, f32, SP3Point2Y, -0.02*(-0.5)/*libm::sinf(-PI/6.0)*/),
    (AtomicF32, f32, SP3Point3X, -0.02*(0.86602540378)/*libm::cosf(-PI/6.0)*/),
    (AtomicF32, f32, SP3Point3Y, -0.02*(-0.5)/*libm::sinf(-PI/6.0)*/),
    (AtomicF32, f32, SP3PointPathDuration, 1.0),
    (AtomicF32, f32, SP3PointDwellTime, 2.0),
);
// Parameter manager

pub struct ParameterManager {
    storage: ParameterStorage,
}

impl Default for ParameterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterManager {
    pub const fn new() -> Self {
        Self {
            storage: ParameterStorage::default(),
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
