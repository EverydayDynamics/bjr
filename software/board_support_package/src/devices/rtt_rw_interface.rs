use core::fmt::Write;
use device_traits::Reader;
use rtt_target;
use rtt_target::{DownChannel, UpChannel};

pub struct RttRWInterface {
    up_channel: UpChannel,
    down_channel: DownChannel,
}
impl RttRWInterface {
    pub fn new(up_channel: UpChannel, down_channel: DownChannel) -> RttRWInterface {
        RttRWInterface {
            up_channel,
            down_channel,
        }
    }
}
impl Write for RttRWInterface {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.up_channel.write_str(s)
    }
}
impl Reader for RttRWInterface {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        self.down_channel.read(buf)
    }
}
