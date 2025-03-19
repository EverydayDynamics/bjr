use device_traits::{TelemetrySender, TelemetrySenderError};
use rtt_target;
use rtt_target::UpChannel;

pub struct RttTelemetry {
    up_channel: UpChannel,
}
impl RttTelemetry {
    pub fn new(up_channel: UpChannel) -> RttTelemetry {
        RttTelemetry {
            up_channel,
        }
    }
}
impl TelemetrySender for RttTelemetry {
    fn send(&mut self, data: &[u8]) -> Result<(), TelemetrySenderError> {
        let sent_bytes = self.up_channel.write(data);
        if sent_bytes < data.len() {
            Err(TelemetrySenderError::SendError)
        } else {
            Ok(())
        }
    }
}
