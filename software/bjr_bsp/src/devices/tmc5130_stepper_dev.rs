use bsp_traits::{MotorState, StepperMotorController, StepperDeviceError, MotorInput, MotorMode};
use tmc5130::{reg, Tmc5130};
use tmc5130::reg::IOIN;
use tmc5130::reg::XACTUAL;
use crate::utils::error_wrapper::ErrorWrapper;

pub struct TMC5130StepperDev<SPI> {
    dev_driver: Tmc5130<SPI>,
    map: reg::Map,
}

fn get_error_from_spistatus(status: reg::SPISTATUS) -> Result<(),StepperDeviceError>{
    //defmt::println!("spistatus:{}", status.0);
    if status.driver_error() {
        //Err(StepperDeviceError::DriverError)
        Ok(())
    } else if status.reset_flag() {
        //Err(StepperDeviceError::UnexpectedReset)
        Ok(())
    } else {
        Ok(())
    }
}

impl<SPI> TMC5130StepperDev<SPI>
where StepperDeviceError: From<ErrorWrapper<<SPI as embedded_hal::spi::ErrorType>::Error>>,
      SPI: embedded_hal::spi::SpiDevice
{
    fn clear_status_flags(&mut self)-> Result<(),StepperDeviceError>{
        let (_,gstat) = self.dev_driver.read_register::<reg::GSTAT>().map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        //defmt::println!("reset:{}, drv:{}, uv:{}", gstat.reset(), gstat.drv_err(), gstat.uv_cp());
        let (_,gstat) = self.dev_driver.read_register::<reg::GSTAT>().map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        //defmt::println!("reset:{}, drv:{}, uv:{}", gstat.reset(), gstat.drv_err(), gstat.uv_cp());
        Ok(())
    }
    fn self_test(&mut self)-> Result<(),StepperDeviceError>{
        const EXPECTED_IOIN_VERSION: u16 = 17;
        let (status,read_ioin) = self.dev_driver.read_register::<IOIN>().map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        get_error_from_spistatus(status)?;
        *self.map.ioin_mut() = read_ioin;
        defmt::println!("Ioin version:{}", read_ioin.version());
        if self.map.ioin().version() != EXPECTED_IOIN_VERSION {
            Err(StepperDeviceError::SelfTestVersionMismatch)
        }else {
            Ok(())
        }

    }
    fn initial_register_set(&mut self)-> Result<(),StepperDeviceError> {
        self.map.chopconf_mut().set_toff(3);
        self.map.chopconf_mut().set_hstrt(4);
        self.map.chopconf_mut().set_hend(1);
        self.map.chopconf_mut().set_tbl(2);
        self.map.chopconf_mut().set_chm(false);
        self.map.ihold_irun_mut().set_ihold(1);
        self.map.ihold_irun_mut().set_irun(4);
        self.map.ihold_irun_mut().set_ihold_delay(6);
        self.map.pwmconf_mut().set_pwm_autoscale(true);
        self.map.pwmconf_mut().set_pwm_ampl(200);
        self.map.pwmconf_mut().set_pwm_grad(1);
        self.map.pwmconf_mut().set_pwm_freq(0);
        self.map.gconf_mut().set_en_pwm_mode(true);
        *self.map.tpowerdown_mut() = reg::TPOWERDOWN::from(10);
        *self.map.tpwmthrs_mut() = reg::TPWMTHRS::from(1000);
        *self.map.a1_mut() = reg::A1::from(0);
        *self.map.v1_mut() = reg::V1::from(0);
        *self.map.amax_mut() = reg::AMAX::from(500);
        *self.map.vmax_mut() = reg::VMAX::from(200000);
        *self.map.dmax_mut() = reg::DMAX::from(700);
        *self.map.d1_mut() = reg::D1::from(0);
        *self.map.vstop_mut() = reg::VSTOP::from(10);
        *self.map.rampmode_mut() = reg::RAMPMODE::from(0);
        *self.map.xtarget_mut() = reg::XTARGET::from(0);
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
            tmc5130::Action::write(self.map.state(reg::Address::RAMPMODE)),
            tmc5130::Action::write(self.map.state(reg::Address::XTARGET)),
        ];

        //defmt::info!("Starting bulk action");
        let status = self.dev_driver.bulk_register_action(&mut actions).map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        //get_error_from_spistatus(status)?;
        Ok(())

    }
    pub fn new(spi:SPI)->Result<Self, StepperDeviceError> {
        let dev_driver= Tmc5130::new(spi);
        let mut stepper_device = TMC5130StepperDev{
            dev_driver,
            map: reg::Map::default(),
        };
        stepper_device.self_test()?;
        stepper_device.initial_register_set()?;
        stepper_device.clear_status_flags()?;
        Ok(stepper_device)
    }
}
const RAMPMODE_POS_MODE: u8 = 0;
const RAMPMODE_VEL_MODE_POS: u8 = 1;
const RAMPMODE_VEL_MODE_NEG: u8 = 2;
const RAMPMODE_HOLD: u8 = 3;

impl<SPI> StepperMotorController for TMC5130StepperDev<SPI>
where StepperDeviceError: From<ErrorWrapper<<SPI as embedded_hal::spi::ErrorType>::Error>>,
      SPI: embedded_hal::spi::SpiDevice
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
                if inputs.velocity <0 {
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
        let status = self.dev_driver.bulk_register_action(&mut actions).map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;
        get_error_from_spistatus(status)?;
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
        let status = self.dev_driver.bulk_register_action(&mut actions).map_err(|e| StepperDeviceError::from(ErrorWrapper(e)))?;

        get_error_from_spistatus(status)?;
        *self.map.vactual_mut() = *vactual_binding.reg().unwrap();
        *self.map.xactual_mut() = *xactual_binding.reg().unwrap();
        Ok(MotorState{
            velocity: self.map.vactual().get(),
            position: self.map.xactual().get(),
            limit_reached: status.status_stop_l(),
            velocity_reached: status.velocity_reached(),
            position_reached: status.position_reached(),
            standstill: status.standstill(),
        })
    }

    fn set_position(&mut self, new_position: i32) -> Result<(), StepperDeviceError> {
        self.map.xactual_mut().set(new_position);
        let status = self.dev_driver.write_register(*self.map.xactual()).map_err(|e|StepperDeviceError::from(ErrorWrapper(e)))?;
        get_error_from_spistatus(status)?;
        Ok(())
    }
}