#![no_main]
#![no_std]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use stm32f4xx_hal as _; // memory layout
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
#[entry]
fn main() -> ! {
    let _x = 42;

    loop {}
}
