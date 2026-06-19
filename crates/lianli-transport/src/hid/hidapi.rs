use crate::error::TransportError;
use hidapi::{HidApi, HidDevice};
use tracing::debug;

/// HID transport wrapping the `hidapi` crate (hidraw backend on Linux).
pub struct HidTransport {
    device: HidDevice,
    vid: u16,
    pid: u16,
}

impl HidTransport {
    pub fn open(api: &HidApi, vid: u16, pid: u16) -> Result<Self, TransportError> {
        let device = api
            .open(vid, pid)
            .map_err(|_| TransportError::DeviceNotFound { vid, pid })?;
        debug!("Opened HID device {vid:04x}:{pid:04x}");
        Ok(Self { device, vid, pid })
    }

    pub fn open_path(api: &HidApi, path: &std::ffi::CStr) -> Result<Self, TransportError> {
        let device = api.open_path(path)?;
        Ok(Self {
            device,
            vid: 0,
            pid: 0,
        })
    }

    pub fn write(&self, data: &[u8]) -> Result<usize, TransportError> {
        self.device
            .write(data)
            .map_err(|e| TransportError::Write(e.to_string()))
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, TransportError> {
        self.device
            .read(buf)
            .map_err(|e| TransportError::Read(e.to_string()))
    }

    pub fn read_timeout(&self, buf: &mut [u8], timeout_ms: i32) -> Result<usize, TransportError> {
        self.device
            .read_timeout(buf, timeout_ms)
            .map_err(|e| TransportError::Read(e.to_string()))
    }

    pub fn vid(&self) -> u16 {
        self.vid
    }

    pub fn pid(&self) -> u16 {
        self.pid
    }

    pub fn inner(&self) -> &HidDevice {
        &self.device
    }
}

/// Enumerate HID devices matching a VID/PID.
pub fn find_hid_devices(api: &HidApi, vid: u16, pid: u16) -> Vec<hidapi::DeviceInfo> {
    api.device_list()
        .filter(|info| info.vendor_id() == vid && info.product_id() == pid)
        .cloned()
        .collect()
}
