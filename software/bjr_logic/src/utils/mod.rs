#![no_std]
use core::fmt::{self, Write};

pub struct DisplayStr<'a> {
    content: fmt::Arguments<'a>,
}

impl<'a> DisplayStr<'a> {
    pub fn new(content: fmt::Arguments<'a>) -> Self {
        DisplayStr { content }
    }
}

impl fmt::Display for DisplayStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::write(f, self.content)
    }
}

#[macro_export]
macro_rules! str_to_display {
    ($($arg:tt)*) => {{
        DisplayStr::new(format_args!($($arg)*))
    }};
}
pub fn usec2sec(usecs:u64) -> f32 {
    //TODO handle overflows
    usecs as f32 / 10e6
}