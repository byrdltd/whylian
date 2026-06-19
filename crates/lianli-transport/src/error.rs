use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("USB error: {0}")]
    Usb(#[from] rusb::Error),

    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),

    #[error("device {vid:04x}:{pid:04x} not found")]
    DeviceNotFound { vid: u16, pid: u16 },

    #[error("write failed: {0}")]
    Write(String),

    #[error("read failed: {0}")]
    Read(String),

    #[error("timeout")]
    Timeout,

    #[error("{0}")]
    Other(String),
}
