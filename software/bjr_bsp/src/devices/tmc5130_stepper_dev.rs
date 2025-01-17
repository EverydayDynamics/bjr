use crate::utils::error_wrapper::ErrorWrapper;
use device_traits::{
    MotorInput, MotorMode, MotorState, StepperDeviceError, StepperMotorController,
    StepperMotorPhase,
};
use tmc5130::reg::XACTUAL;
use tmc5130::reg::{DRV_STATUS, IOIN};
use tmc5130::{reg, Tmc5130};

pub struct TMC5130StepperDev<SPI, PIN> {
    dev_driver: Tmc5130<SPI>,
    limit_pin: PIN,
    map: reg::Map,
}

impl<SPI, PIN> TMC5130StepperDev<SPI, PIN>
where
    StepperDeviceError: From<ErrorWrapper<<SPI as embedded_hal::spi::ErrorType>::Error>>,
    SPI: embedded_hal::spi::SpiDevice,
    PIN: embedded_hal::digital::InputPin,
{
    fn get_error_from_drv_status(&mut self) -> Result<(), StepperDeviceError> {
        let (status, read_drv_status) = self
            .dev_driver
            .read_register::<DRV_STATUS>()
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        if read_drv_status.s2ga() {
            Err(StepperDeviceError::ShortToGround(StepperMotorPhase::A))
        } else if read_drv_status.s2gb() {
            Err(StepperDeviceError::ShortToGround(StepperMotorPhase::B))
        } else if read_drv_status.ot() {
            Err(StepperDeviceError::OverTemperatureShutdown)
        }else if read_drv_status.olb() {
            Err(StepperDeviceError::OpenLoad(StepperMotorPhase::B))
        } else if read_drv_status.ola() {
            Err(StepperDeviceError::OpenLoad(StepperMotorPhase::A))
        }else {
            //Err(StepperDeviceError::OpenLoad(StepperMotorPhase::A))
            Ok(())
        }
    }

    fn get_error_from_spistatus(
        &mut self,
        status: reg::SPISTATUS,
    ) -> Result<(), StepperDeviceError> {
        if status.driver_error() {
            let (status, read_drv_status) = self
                .dev_driver
                .read_register::<DRV_STATUS>()
                .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
            if read_drv_status.s2ga() {
                Err(StepperDeviceError::ShortToGround(StepperMotorPhase::A))
            } else if read_drv_status.s2gb() {
                Err(StepperDeviceError::ShortToGround(StepperMotorPhase::B))
            } else if read_drv_status.ot() {
                Err(StepperDeviceError::OverTemperatureShutdown)
            } else if read_drv_status.ola() {
                Err(StepperDeviceError::OpenLoad(StepperMotorPhase::A))
            }else if read_drv_status.olb() {
                Err(StepperDeviceError::OpenLoad(StepperMotorPhase::B))
            }else {
                Ok(())
            }
        } else if status.reset_flag() {
            Ok(())
        } else {
            Ok(())
        }
    }
    fn clear_status_flags(&mut self) -> Result<(), StepperDeviceError> {
        let (_, gstat) = self
            .dev_driver
            .read_register::<reg::GSTAT>()
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        let (_, gstat) = self
            .dev_driver
            .read_register::<reg::GSTAT>()
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        Ok(())
    }
    fn self_test(&mut self) -> Result<(), StepperDeviceError> {
        const EXPECTED_IOIN_VERSION: u16 = 17;

        let (status, read_ioin) = self
            .dev_driver
            .read_register::<IOIN>()
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        self.get_error_from_spistatus(status)?;
        *self.map.ioin_mut() = read_ioin;
        if self.map.ioin().drv_enn_cfg6() {
            Err(StepperDeviceError::EnableError)
        }else if self.map.ioin().version() != EXPECTED_IOIN_VERSION {
            Err(StepperDeviceError::SelfTestVersionMismatch)
        } else {
            Ok(())
        }
    }
    fn initial_register_set(&mut self) -> Result<(), StepperDeviceError> {
        self.map.chopconf_mut().set_toff(3);
        self.map.chopconf_mut().set_hstrt(4);
        self.map.chopconf_mut().set_hend(1);
        self.map.chopconf_mut().set_tbl(2);
        self.map.chopconf_mut().set_chm(false);
        self.map.chopconf_mut().set_vsense(false);
        //set microstep resolution to 8 usteps
        self.map.chopconf_mut().set_mres(5);
        self.map.ihold_irun_mut().set_ihold(3);
        self.map.ihold_irun_mut().set_irun(20);
        self.map.ihold_irun_mut().set_ihold_delay(6);
        self.map.pwmconf_mut().set_pwm_autoscale(true);
        self.map.pwmconf_mut().set_pwm_ampl(200);
        self.map.pwmconf_mut().set_pwm_grad(1);
        self.map.pwmconf_mut().set_pwm_freq(0);
        self.map.gconf_mut().set_en_pwm_mode(true);
        self.map.gconf_mut().set_shaft(true);
        *self.map.tpowerdown_mut() = reg::TPOWERDOWN::from(10);
        *self.map.tpwmthrs_mut() = reg::TPWMTHRS::from(1000);
        *self.map.a1_mut() = reg::A1::from(0);
        *self.map.v1_mut() = reg::V1::from(0);
        *self.map.amax_mut() = reg::AMAX::from(500);
        *self.map.vmax_mut() = reg::VMAX::from(200000);
        *self.map.dmax_mut() = reg::DMAX::from(700);
        *self.map.d1_mut() = reg::D1::from(0);
        *self.map.vstop_mut() = reg::VSTOP::from(10);
        *self.map.vstart_mut() = reg::VSTART::from(0);
        *self.map.rampmode_mut() = reg::RAMPMODE::from(0);
        *self.map.xactual_mut() = reg::XACTUAL::from(0);
        *self.map.vactual_mut() = reg::VACTUAL::default();

        let mut actions = [
            tmc5130::Action::write(self.map.state(reg::Address::CHOPCONF)),
            tmc5130::Action::write(self.map.state(reg::Address::IHOLD_IRUN)),
            tmc5130::Action::write(self.map.state(reg::Address::TPOWERDOWN)),
            tmc5130::Action::write(self.map.state(reg::Address::GCONF)),
            tmc5130::Action::write(self.map.state(reg::Address::TPWMTHRS)),
            tmc5130::Action::write(self.map.state(reg::Address::PWMCONF)),
            tmc5130::Action::write(self.map.state(reg::Address::A1)),
            tmc5130::Action::write(self.map.state(reg::Address::V1)),
            tmc5130::Action::write(self.map.state(reg::Address::AMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::VMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::DMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::D1)),
            tmc5130::Action::write(self.map.state(reg::Address::VSTOP)),
            tmc5130::Action::write(self.map.state(reg::Address::VSTART)),
            tmc5130::Action::write(self.map.state(reg::Address::RAMPMODE)),
            tmc5130::Action::write(self.map.state(reg::Address::XACTUAL)),
        ];

        let status = self
            .dev_driver
            .bulk_register_action(&mut actions)
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        self.get_error_from_spistatus(status)
        //Ok(())
    }
    pub fn new(spi: SPI, limit_pin: PIN) -> Result<Self, StepperDeviceError> {
        let dev_driver = Tmc5130::new(spi);
        let mut stepper_device = TMC5130StepperDev {
            dev_driver,
            limit_pin,
            map: reg::Map::default(),
        };
        stepper_device.self_test()?;
        stepper_device.get_error_from_drv_status()?;
        stepper_device.initial_register_set()?;
        stepper_device.clear_status_flags()?;
        Ok(stepper_device)
    }
}
const RAMPMODE_POS_MODE: u8 = 0;
const RAMPMODE_VEL_MODE_POS: u8 = 1;
const RAMPMODE_VEL_MODE_NEG: u8 = 2;
const RAMPMODE_HOLD: u8 = 3;

impl<SPI, PIN> StepperMotorController for TMC5130StepperDev<SPI, PIN>
where
    StepperDeviceError: From<ErrorWrapper<<SPI as embedded_hal::spi::ErrorType>::Error>>,
    SPI: embedded_hal::spi::SpiDevice,
    PIN: embedded_hal::digital::InputPin,
{
    fn set_inputs(&mut self, inputs: MotorInput) -> Result<(), StepperDeviceError> {
        let vel_abs = inputs.velocity.abs() as u32;
        let acc_abs = inputs.acceleration.abs() as u16;
        self.map.vmax_mut().set(vel_abs);
        self.map.v1_mut().set(vel_abs);
        self.map.d1_mut().set(acc_abs);
        self.map.a1_mut().set(acc_abs);
        self.map.amax_mut().set(acc_abs);
        self.map.dmax_mut().set(acc_abs);
        self.map.xtarget_mut().set(inputs.position);
        match inputs.mode {
            MotorMode::PositionCtrl => {
                self.map.rampmode_mut().set(RAMPMODE_POS_MODE);
            }
            MotorMode::VelocityCtrl => {
                if inputs.velocity < 0 {
                    self.map.rampmode_mut().set(RAMPMODE_VEL_MODE_NEG);
                } else {
                    self.map.rampmode_mut().set(RAMPMODE_VEL_MODE_POS);
                }
            }
        }

        let mut drv_status_binding = tmc5130::reg::State::from(*self.map.drv_status_mut());
        let mut actions = [
            tmc5130::Action::read(&mut drv_status_binding),
            tmc5130::Action::write(self.map.state(reg::Address::RAMPMODE)),
            tmc5130::Action::write(self.map.state(reg::Address::AMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::DMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::VMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::V1)),
            tmc5130::Action::write(self.map.state(reg::Address::D1)),
            tmc5130::Action::write(self.map.state(reg::Address::A1)),
            tmc5130::Action::write(self.map.state(reg::Address::XTARGET)),
        ];
        let status = self
            .dev_driver
            .bulk_register_action(&mut actions)
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        self.get_error_from_spistatus(status)?;
        self.map.drv_status_mut().0 = drv_status_binding.into();
        Ok(())
    }

    fn get_state(&mut self) -> Result<MotorState, StepperDeviceError> {
        let mut vactual_binding = reg::State::VACTUAL(Default::default());
        let mut xactual_binding = reg::State::XACTUAL(Default::default());
        let mut actions = [
            tmc5130::Action::read(&mut xactual_binding),
            tmc5130::Action::read(&mut vactual_binding),
        ];
        let status = self
            .dev_driver
            .bulk_register_action(&mut actions)
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;

        self.get_error_from_spistatus(status)?;
        *self.map.vactual_mut() = *vactual_binding.reg().unwrap();
        *self.map.xactual_mut() = *xactual_binding.reg().unwrap();
        Ok(MotorState {
            velocity: self.map.vactual().get(),
            position: self.map.xactual().get(),
            limit_reached: self.limit_pin.is_low().unwrap(),
            velocity_reached: status.velocity_reached(),
            position_reached: status.position_reached(),
            standstill: status.standstill(),
        })
    }

    fn set_position(&mut self, new_position: i32) -> Result<(), StepperDeviceError> {
        self.map.xactual_mut().set(new_position);
        let status = self
            .dev_driver
            .write_register(*self.map.xactual())
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        self.get_error_from_spistatus(status)?;
        Ok(())
    }

    fn test_motion(&mut self) -> Result<(), StepperDeviceError> {
        self.map.chopconf_mut().set_toff(3);
        self.map.chopconf_mut().set_hstrt(4);
        self.map.chopconf_mut().set_hend(1);
        self.map.chopconf_mut().set_tbl(2);
        self.map.chopconf_mut().set_chm(false);
        //set microstep resolution to 8 usteps
        self.map.ihold_irun_mut().set_ihold(10);
        self.map.ihold_irun_mut().set_irun(31);
        self.map.ihold_irun_mut().set_ihold_delay(6);
        self.map.tpowerdown_mut().set(10);
        self.map.tpwmthrs_mut().set(500);
        *self.map.a1_mut() = reg::A1::from(1000);
        *self.map.v1_mut() = reg::V1::from(50000);
        *self.map.amax_mut() = reg::AMAX::from(500);
        *self.map.vmax_mut() = reg::VMAX::from(200000);
        *self.map.dmax_mut() = reg::DMAX::from(700);
        *self.map.d1_mut() = reg::D1::from(1400);
        *self.map.vstop_mut() = reg::VSTOP::from(10);
        self.map.rampmode_mut().set(0);
        self.map.xtarget_mut().set(-151200);
        let mut actions = [
            tmc5130::Action::write(self.map.state(reg::Address::CHOPCONF)),
            tmc5130::Action::write(self.map.state(reg::Address::IHOLD_IRUN)),
            tmc5130::Action::write(self.map.state(reg::Address::TPOWERDOWN)),
            tmc5130::Action::write(self.map.state(reg::Address::TPWMTHRS)),
            tmc5130::Action::write(self.map.state(reg::Address::A1)),
            tmc5130::Action::write(self.map.state(reg::Address::V1)),
            tmc5130::Action::write(self.map.state(reg::Address::AMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::VMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::DMAX)),
            tmc5130::Action::write(self.map.state(reg::Address::D1)),
            tmc5130::Action::write(self.map.state(reg::Address::VSTOP)),
            tmc5130::Action::write(self.map.state(reg::Address::RAMPMODE)),
            tmc5130::Action::write(self.map.state(reg::Address::XTARGET)),
        ];
        let status = self
            .dev_driver
            .bulk_register_action(&mut actions)
            .map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        self.get_error_from_spistatus(status)
    }
}
