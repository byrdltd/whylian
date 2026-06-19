use super::runtime::SendError;
use super::ServiceManager;
use lianli_media::MediaAsset;
use std::sync::Arc;
use tracing::{debug, info, warn};

impl ServiceManager {
    pub(super) fn stream_target(&mut self, this_asset: Arc<MediaAsset>) {
        // Find ID of matching target
        let target_id = self
            .targets
            .iter()
            .find(|(_, t)| t.asset.config_key == this_asset.config_key)
            .map(|(id, _)| *id);

        if let Some(id) = target_id {
            if let Some(target) = self.targets.get_mut(&id) {
                let send_result = {
                    let wireless = self.wireless.lock();
                    target.send_frame(&*wireless, &mut self.packet_builder)
                };
                match send_result {
                    Ok(true) => {
                        target.consecutive_errors = 0;
                        if target.frame_counter % 30 == 0 {
                            debug!(
                                "LCD[{}] streamed {} frames",
                                target.index, target.frame_counter
                            );
                        }
                    }
                    Ok(false) => {}
                    Err(SendError::Usb(err)) => {
                        if let Some(target) = self.targets.get_mut(&id) {
                            target.consecutive_errors += 1;
                            if target.consecutive_errors < 3 {
                                warn!(
                                    "LCD[{}] USB error ({}/3): {err}",
                                    target.index, target.consecutive_errors
                                );
                                return;
                            }
                        }
                        self.handle_usb_error(id, err);
                    }
                    Err(SendError::Other(err)) => {
                        warn!("LCD[{}] media error: {err}", target.index);
                        let mut removed = self.targets.remove(&id).unwrap();
                        removed.stop();
                    }
                }
            }
        }
    }

    fn handle_usb_error(&mut self, index: usize, err: lianli_transport::TransportError) {
        if let Some(mut target) = self.targets.remove(&index) {
            warn!("LCD[{index}] USB error: {err}");
            target.stop();
        }
        if matches!(err, lianli_transport::TransportError::Timeout) && self.recover_wireless() {
            info!("Wireless link recovered");
            self.activate_wireless_control();
        }
    }
}
