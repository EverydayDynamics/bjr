use core::cell::RefCell;
use core::fmt::{Debug, Formatter};
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{ErrorKind,Operation, SpiBus, SpiDevice};

pub struct Spidev<'a, SPI, CSPIN> {
    guarded_spi: &'a Mutex<RefCell<Option<SPI>>>,
    cs_executor: CsExecutor<CSPIN>,
}
struct CsExecutor<CSPIN> {
    cs_pin: CSPIN,
}
impl<CSPIN> CsExecutor<CSPIN>
where
    CSPIN: OutputPin,
{
    pub fn selected<F, R>(&mut self, f: F) -> Result<R, CSPIN::Error>
    where
        F: FnOnce() -> R,
    {
        self.cs_pin.set_low()?;
        let r = f();
        self.cs_pin.set_high()?;
        Ok(r)
    }
}
impl<SPI, CSPIN> Spidev<'_, SPI, CSPIN> {
    pub fn new(guarded_spi: &Mutex<RefCell<Option<SPI>>>, cs_pin: CSPIN) -> Spidev<SPI, CSPIN> {
        Spidev {
            guarded_spi,
            cs_executor: CsExecutor { cs_pin },
        }
    }
}
pub enum SpiDevError<SPI, CSPIN>
where
    SPI: SpiBus,
    CSPIN: OutputPin,
{
    SPIError(SPI::Error),
    CSPinError(CSPIN::Error),
    MutexError,
    NotImplemented,
}
impl<SPI, CSPIN> Debug for SpiDevError<SPI, CSPIN>
where
    SPI: SpiBus,
    CSPIN: OutputPin,
{
    fn fmt(&self, _f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
impl<SPI, CSPIN> embedded_hal::spi::Error for SpiDevError<SPI, CSPIN>
where
    SPI: SpiBus,
    CSPIN: OutputPin,
{
    fn kind(&self) -> ErrorKind {
        match self {
            SpiDevError::SPIError(_spi_error) => {
                //Todo: make the error propagation proper here
                embedded_hal::spi::ErrorKind::Overrun
            }
            SpiDevError::CSPinError(_) => embedded_hal::spi::ErrorKind::ChipSelectFault,
            SpiDevError::NotImplemented => embedded_hal::spi::ErrorKind::Other,
            SpiDevError::MutexError => embedded_hal::spi::ErrorKind::Other,
        }
    }
}

impl<SPI, CSPIN> embedded_hal::spi::ErrorType for Spidev<'_, SPI, CSPIN>
where
    SPI: SpiBus,
    CSPIN: OutputPin,
{
    type Error = SpiDevError<SPI, CSPIN>;
}

impl<SPI, CSPIN> SpiDevice for Spidev<'_, SPI, CSPIN>
where
    SPI: SpiBus,
    CSPIN: OutputPin,
{
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        let mut spi =
            interrupt::free(|cs| self.guarded_spi.borrow(cs).replace(None)).expect("aaaaaa");
        let result: Result<(), SpiDevError<SPI, CSPIN>> = self
            .cs_executor
            .selected(|| {
                for operation in operations {
                    match operation {
                        Operation::Read(inbuf) => spi
                            .read(inbuf)
                            .map_err(|e| SpiDevError::<SPI, CSPIN>::SPIError(e))?,
                        Operation::Write(outbuf) => spi
                            .write(outbuf)
                            .map_err(|e| SpiDevError::<SPI, CSPIN>::SPIError(e))?,
                        Operation::Transfer(inbuf, outbuf) => spi
                            .transfer(inbuf, outbuf)
                            .map_err(|e| SpiDevError::<SPI, CSPIN>::SPIError(e))?,
                        Operation::TransferInPlace(inoutbuf) => spi
                            .transfer_in_place(inoutbuf)
                            .map_err(|e| SpiDevError::<SPI, CSPIN>::SPIError(e))?,
                        Operation::DelayNs(_) => Err(SpiDevError::<SPI, CSPIN>::NotImplemented)?,
                    }
                }
                Ok(())
            })
            .map_err(|e| SpiDevError::<SPI, CSPIN>::CSPinError(e))?;
        result?;
        interrupt::free(|cs| {
            self.guarded_spi.borrow(cs).replace(Some(spi));
        });
        Ok(())
    }

    /// spi write implementation managing the CS pin
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let mut ops: [Operation<'_, u8>; 1] = [Operation::Write(data)];
        self.transaction(&mut ops)
    }

    fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        let mut ops: [Operation<'_, u8>; 1] = [Operation::TransferInPlace(data)];
        self.transaction(&mut ops)
    }
}
