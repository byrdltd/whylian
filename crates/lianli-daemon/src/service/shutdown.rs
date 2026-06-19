use super::ServiceManager;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::info;

impl ServiceManager {
    pub(super) fn shutdown(&mut self) {
        for target in self.targets.values_mut() {
            target.stop();
        }
        self.targets.clear();

        if let Some(fan_controller) = self.fan_controller.take() {
            info!("Stopping fan controller...");
            fan_controller.stop();
        }

        if let Some(aio) = self.aio_controller.take() {
            info!("Stopping AIO controller...");
            aio.stop();
        }

        // Drop RGB controller before HID backends so device handles are released cleanly
        self.rgb_controller = None;
        self.ipc_state.lock().rgb_controller = None;
        self.wired_fan_devices = Arc::new(HashMap::new());
        self.hid_backends.clear();

        self.wireless.lock().stop();

        // Stop OpenRGB server
        self.openrgb_stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.openrgb_thread.take() {
            let _ = thread.join();
        }

        // Stop IPC server
        self.ipc_stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.ipc_thread.take() {
            let _ = thread.join();
        }
    }
}
