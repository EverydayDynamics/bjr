#![no_main]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod boards;
mod devices;
#[cfg(feature = "main_board")]
pub mod main_board;
#[cfg(feature = "mock_board")]
pub mod mock_board;
mod utils;

#[cfg(feature = "main_board")]
pub use main_board::MyBoard as Board;
#[cfg(feature = "mock_board")]
pub use mock_board::MockBoard as Board;
