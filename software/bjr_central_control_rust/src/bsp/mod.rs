pub mod traits;
mod devices;
#[cfg(feature = "mock_board")]
pub mod mock_board;
mod real_board;

#[cfg(feature = "mock_board")]
pub use mock_board::MyBoard as Board;