use core::fmt::{Debug, Formatter};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::{Error, ErrorKind, ErrorType, Operation, SpiBus, SpiDevice};
use core::cell::RefCell;
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use defmt::Format;

pub struct Spidev<'a, SPI, CSPIN> {
    guarded_spi: &'a Mutex<RefCell<Option<SPI>>>,
    cs_executor: CsExecutor<CSPIN>,
}
struct CsExecutor<CSPIN> {
    cs_pin: CSPIN,
}
impl<CSPIN> CsExecutor<CSPIN>
where CSPIN: OutputPin
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
impl<SPI, CSPIN> Spidev<'_, SPI, CSPIN>
{
    pub fn new(guarded_spi: &Mutex<RefCell<Option<SPI>>>, cs_pin: CSPIN) ->Spidev<SPI, CSPIN>{
        Spidev{ guarded_spi, cs_executor: CsExecutor { cs_pin } }
    }
}
pub enum SpiDevError<SPI, CSPIN>
    where
        SPI: SpiBus,
        CSPIN: OutputPin
{
    SPIError(SPI::Error),
    CSPinError(CSPIN::Error),
    MutexError,
    NotImplemented,
}
impl<SPIERR, CSPINERR> Format for SpiDevError<SPIERR, CSPINERR>
    where
        SPIERR: SpiBus,
        CSPINERR: OutputPin
{
    fn format(&self, f: defmt::Formatter) {
        match self {
            SpiDevError::SPIError(spierr) => {
                defmt::write!(f, "Failed to access SPI bus:");
                match spierr.kind()  {
                    ErrorKind::Overrun => defmt::write!(f, "The peripheral receive buffer was overrun"),
                    ErrorKind::ModeFault => defmt::write!(
                        f,
                        "Multiple devices on the SPI bus are trying to drive the slave select pin"
                    ),
                    ErrorKind::FrameFormat => defmt::write!(
                        f,
                        "Received data does not conform to the peripheral configuration"
                    ),
                    ErrorKind::ChipSelectFault => defmt::write!(
                        f,
                        "An error occurred while asserting or deasserting the Chip Select pin"
                    ),
                    ErrorKind::Other => defmt::write!(
                        f,
                        "A different error occurred. The original error may contain more information"
                    ),
                    _ => defmt::write!(
                        f,
                        "An unknown error occured.")
                };
            }
            SpiDevError::CSPinError(cserr) => {
                defmt::write!(f, "Failed set CS pin:");

            }
            SpiDevError::MutexError => {
                defmt::write!(f, "Failed to acquire mutex");
            }
            SpiDevError::NotImplemented => {
                defmt::write!(f, "Requiested operation was not implemented.");
            }
        }
    }
}
impl<SPI, CSPIN> Debug for SpiDevError<SPI, CSPIN>
    where
        SPI: SpiBus,
        CSPIN: OutputPin
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
impl<SPI, CSPIN> embedded_hal::spi::Error for SpiDevError<SPI, CSPIN>
    where
        SPI: SpiBus,
        CSPIN: OutputPin
{
    fn kind(&self) -> ErrorKind {
        match self {
            SpiDevError::SPIError(spi_error) => {
               //Todo: make the error propagation proper here
                embedded_hal::spi::ErrorKind::Overrun
            }
            SpiDevError::CSPinError(_) => {
                embedded_hal::spi::ErrorKind::ChipSelectFault
            }
            SpiDevError::NotImplemented => {
                embedded_hal::spi::ErrorKind::Other
            }
            SpiDevError::MutexError => {
                embedded_hal::spi::ErrorKind::Other
            }
        }
    }
}

impl<SPI, CSPIN> embedded_hal::spi::ErrorType
for Spidev<'_, SPI, CSPIN>
    where
        SPI: SpiBus,
        CSPIN: OutputPin
{
    type Error = SpiDevError<SPI, CSPIN>;
}


impl<SPI, CSPIN> SpiDevice
for Spidev<'_, SPI, CSPIN>
    where
        SPI: SpiBus,
        CSPIN: OutputPin,
{
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {

        let mut spi = interrupt::free(|cs| {
            self.guarded_spi.borrow(cs).replace(None)
        }).expect("aaaaaa");
        let result :Result<(), SpiDevError::<SPI, CSPIN>>= self.cs_executor.selected(||{
            for operation in operations {
                match operation {
                    Operation::Read(inbuf) => {
                        spi.read(inbuf).map_err(|e|SpiDevError::<SPI, CSPIN>::SPIError(e))?
                    }
                    Operation::Write(outbuf) => {
                        spi.write(outbuf).map_err(|e|SpiDevError::<SPI, CSPIN>::SPIError(e))?
                    }
                    Operation::Transfer(inbuf, outbuf) => {
                        spi.transfer(inbuf, outbuf).map_err(|e|SpiDevError::<SPI, CSPIN>::SPIError(e))?
                    }
                    Operation::TransferInPlace(inoutbuf) => {
                        spi.transfer_in_place(inoutbuf).map_err(|e|SpiDevError::<SPI, CSPIN>::SPIError(e))?
                    }
                    Operation::DelayNs(_) => {Err(SpiDevError::<SPI, CSPIN>::NotImplemented)?}

                }
            }
            Ok(())
        }).map_err(|e|SpiDevError::<SPI, CSPIN>::CSPinError(e))?;
        result?;
        interrupt::free(|cs| {
            self.guarded_spi.borrow(cs).replace(Some(spi));
        });
        Ok(())
    }

    /// spi write implementation managing the CS pin
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let mut ops :[Operation<'_, u8>;1] =[Operation::Write(data)];
        self.transaction(&mut ops)
    }

    fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        let mut ops :[Operation<'_, u8>;1] =[Operation::TransferInPlace(data)];
        self.transaction(&mut ops)
    }
}
