use crate::app::parameter_manager::{parameter_manager, MotorLowerPosLimit, MotorLowerVelLimit, MotorUpperPosLimit, MotorUpperVelLimit, ParameterType, MCVelMax, MCVelMin};
use core::fmt::{Display, Formatter};
use crate::app::severity_trait::ErrorSeverity::Report;

#[derive(PartialEq, Copy, Clone)]
pub enum LimitLevel {
    Ignore,
    Warn,
    Clamp,
    Error,
}
pub struct LimitChecker {}
#[derive(PartialEq, Copy, Clone)]
pub struct LimitReport<T> {
    limit: T,
    value: T,
}
impl<T: Display> Display for LimitReport<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Value {} out of limit {}", self.value, self.limit)
    }
}
#[derive(PartialEq, Copy, Clone)]
pub struct LimitError<T>(LimitReport<T>);
impl<T: Display> Display for LimitError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct LimitWarning<T>(LimitReport<T>);
impl<T: Display> Display for LimitWarning<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Default for LimitChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl LimitChecker {
    pub fn new() -> LimitChecker {
        LimitChecker {}
    }
}
pub trait Limit<T>
where
    T: PartialOrd + Display + Copy,
{
    type UpperParam: ParameterType<ReturnType = T>;
    type LowerParam: ParameterType<ReturnType = T>;
    fn get_level(&self) -> LimitLevel;
    fn set_level(&mut self, level: LimitLevel);
    fn check(&self, value: &mut T) -> Result<Option<LimitWarning<T>>, LimitError<T>> {
        let upper_limit = parameter_manager().get::<Self::UpperParam>();
        let lower_limit = parameter_manager().get::<Self::LowerParam>();
        let report :Option<LimitReport<T>> =
        if *value > upper_limit {
            if self.get_level() == LimitLevel::Clamp {
                *value = upper_limit;
            }
            Some(LimitReport{ limit: upper_limit, value: *value })
        } else if *value < lower_limit {
            if self.get_level() == LimitLevel::Clamp {
                *value = lower_limit;
            }
            Some(LimitReport{ limit: upper_limit, value: *value })
        } else {
            None
        };
        match self.get_level() {
            LimitLevel::Ignore => {Ok(None)}
            LimitLevel::Warn => {Ok(report.map(|res|LimitWarning(res)))}
            LimitLevel::Clamp => {Ok(None)}
            LimitLevel::Error => { report.map_or(Ok(None), |res|Err(LimitError(res)))}
        }
    }
}

pub struct MotorPosLimit {
    level: LimitLevel,
}
impl MotorPosLimit {
    pub fn new(level: LimitLevel) -> MotorPosLimit {
        MotorPosLimit { level }
    }
}
impl Limit<f32> for MotorPosLimit {
    type UpperParam = MotorUpperPosLimit;
    type LowerParam = MotorLowerPosLimit;

    fn get_level(&self) -> LimitLevel {self.level}

    fn set_level(&mut self, level: LimitLevel) {self.level = level;}
}
pub struct MotorVelLimit {
    level: LimitLevel,
}
impl MotorVelLimit {
    pub fn new(level: LimitLevel) -> MotorVelLimit {
        MotorVelLimit { level }
    }
}
impl Limit<f32> for MotorVelLimit {
    type UpperParam = MotorUpperVelLimit;
    type LowerParam = MotorLowerVelLimit;

    fn get_level(&self) -> LimitLevel { self.level}

    fn set_level(&mut self, level: LimitLevel) {self.level = level;}
}
pub struct MotorControlVelLimit{
    level: LimitLevel,
}
impl MotorControlVelLimit {
    pub fn new(level: LimitLevel) -> MotorControlVelLimit {
        MotorControlVelLimit { level }
    }
}
impl Limit<f32> for MotorControlVelLimit {
    type UpperParam = MCVelMax;
    type LowerParam = MCVelMin;

    fn get_level(&self) -> LimitLevel { self.level}

    fn set_level(&mut self, level: LimitLevel) {self.level = level;}
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_within() {
        let test_motor_vel_limit = MotorVelLimit::new(true);
        let mut test_value = 10f32;
        assert!(test_motor_vel_limit.check(&mut test_value) == Ok(()));
    }
    #[test]
    fn test_limit_above() {
        let test_motor_vel_limit = MotorVelLimit::new(true);
        let mut test_value = 1e7;
        assert!(
            test_motor_vel_limit.check(&mut test_value)
                == Err(LimitError::OverLimit(LimitReport {
                    limit: 1e6,
                    value: 1e7
                }))
        );
    }
    #[test]
    fn test_limit_below() {
        let test_motor_vel_limit = MotorVelLimit::new(true);
        let mut test_value = -1e7;
        assert!(
            test_motor_vel_limit.check(&mut test_value)
                == Err(LimitError::UnderLimit(LimitReport {
                    limit: -1e6,
                    value: -1e7
                }))
        );
    }
    #[test]
    fn test_limit_above_disabled() {
        let test_motor_vel_limit = MotorVelLimit::new(false);
        assert!(test_motor_vel_limit.check(1e7) == Ok(()));
    }
}
