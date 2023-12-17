use spin::Mutex;
use embedded_hal::blocking::i2c;
use embedded_hal::blocking::i2c::{SevenBitAddress, Write, WriteRead};

/// `std` `Mutex`-based shared bus [`I2c`] implementation.
///
/// Sharing is implemented with an `std` [`Mutex`](std::sync::Mutex). It allows a single bus across multiple threads,
/// with finer-grained locking than [`CriticalSectionDevice`](super::CriticalSectionDevice). The downside is that
/// it is only available in `std` targets.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub struct MutexDevice<'a, T> {
    bus: &'a Mutex<T>,
}

impl<'a, T> MutexDevice<'a, T> {
    /// Create a new `MutexDevice`
    pub fn new(bus: &'a Mutex<T>) -> Self {
        Self { bus }
    }
}

impl<'a, T,E> Write for MutexDevice<'a, T>
    where
        T: Write<Error = E>,
{
    type Error = E;
    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.lock();
        bus.write(address, write)
    }
}
impl<'a, T,E> WriteRead for MutexDevice<'a, T>
    where
        T: WriteRead<Error = E>,
{
    type Error = E;

    fn write_read(&mut self, address: SevenBitAddress, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.lock();
        bus.write_read(address, bytes, buffer)    }
}
