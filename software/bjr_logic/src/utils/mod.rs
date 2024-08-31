pub mod test_helper;
pub fn usec2sec(usecs: u64) -> f32 {
    //TODO handle overflows
    usecs as f32 / 1e6
}
