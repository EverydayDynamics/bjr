#![no_main]
#![no_std]

#[cfg(feature = "mock_board")]
pub mod mock_board;
#[cfg(feature = "main_board")]
pub mod main_board;
mod utils;
mod devices;
pub mod boards;

#[cfg(feature = "mock_board")]
pub use mock_board::MockBoard as Board;
#[cfg(feature = "main_board")]
pub use main_board::MyBoard as Board;
