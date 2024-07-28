#![no_main]
#![no_std]

use bjr as _;
#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("BSP loaded!");
    bjr::exit()
}

