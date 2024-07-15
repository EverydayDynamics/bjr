use crate::bsp::traits::Button;
pub struct MockButton {
}

impl MockButton
{
    pub fn new() -> Self {
        MockButton { }
    }
}

impl Button for MockButton
{
    fn is_pressed(&mut self) -> bool {
        return true;
    }
}
