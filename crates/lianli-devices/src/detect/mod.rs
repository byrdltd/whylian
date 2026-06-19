//! Device discovery and HID/USB backend opening.

mod backends;
mod binding;
mod controllers;
mod enumerate;

pub use backends::{
    hidraw_path_for_usb_topology, open_hid_backend_hidapi, open_hid_backend_rusb,
    open_hid_lcd_by_topology, open_hid_lcd_by_vid_pid, open_hid_lcd_device_rusb,
};
pub use binding::ensure_hid_devices_bound;
pub use controllers::{create_hid_lcd_device, create_wired_controllers, WiredControllerSet};
pub use enumerate::{enumerate_devices, enumerate_hid_devices, probe_tl_lcd_port_indices_rusb};

use lianli_shared::device_id::DeviceFamily;
use rusb::{Device, GlobalContext};

/// A detected USB device with its identified family.
#[derive(Debug)]
pub struct DetectedDevice {
    pub device: Device<GlobalContext>,
    pub family: DeviceFamily,
    pub name: &'static str,
    pub vid: u16,
    pub pid: u16,
    pub bus: u8,
    pub address: u8,
    pub serial: Option<String>,
    /// HID usage page filter from the device entry. When set, only the HID
    /// interface with this usage page should be opened.
    pub hid_usage_page: Option<u16>,
}

impl DetectedDevice {
    /// Stable device ID: serial if unique, otherwise USB port path (bus-port topology).
    pub fn device_id(&self) -> String {
        match &self.serial {
            Some(s) if !is_non_unique_serial(s.as_str()) => {
                format!("hid:{}", s)
            }
            _ => {
                let port_path = self
                    .device
                    .port_numbers()
                    .ok()
                    .filter(|p| !p.is_empty())
                    .map(|ports| {
                        let parts: Vec<String> = ports.iter().map(|p| p.to_string()).collect();
                        format!("{}-{}", self.bus, parts.join("."))
                    })
                    .unwrap_or_else(|| format!("{}-{}", self.bus, self.address));
                format!("hid:{:04x}:{:04x}:{}", self.vid, self.pid, port_path)
            }
        }
    }
}

/// A detected HID device with its identified family.
#[derive(Debug, Clone)]
pub struct DetectedHidDevice {
    pub family: DeviceFamily,
    pub name: &'static str,
    pub vid: u16,
    pub pid: u16,
    pub path: std::ffi::CString,
    pub serial: Option<String>,
    /// USB port path (e.g. "1-5.3") for stable device IDs.
    pub usb_port_path: Option<String>,
    /// (port, fan_index) for daisy-chained TL LCD fans, read during enumeration.
    pub port_index: Option<(u8, u8)>,
}

/// Known non-unique HID serial strings (chip manufacturer names, firmware
/// version markers, etc. — not actual per-device serials).
const NON_UNIQUE_SERIALS: &[&str] = &["Nuvoton"];

fn is_non_unique_serial(s: &str) -> bool {
    NON_UNIQUE_SERIALS.contains(&s) || s.starts_with("TL_LCDV")
}

impl DetectedHidDevice {
    /// Stable device ID: serial if unique, otherwise USB port path.
    pub fn device_id(&self) -> String {
        match &self.serial {
            Some(s) if !is_non_unique_serial(s.as_str()) => {
                format!("hid:{}", s)
            }
            _ => match &self.usb_port_path {
                Some(port_path) => {
                    format!("hid:{:04x}:{:04x}:{}", self.vid, self.pid, port_path)
                }
                None => {
                    let path_bytes = self.path.as_bytes();
                    let hash: u32 = path_bytes
                        .iter()
                        .fold(0u32, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u32));
                    format!("hid:{:04x}:{:04x}:{:04x}", self.vid, self.pid, hash as u16)
                }
            },
        }
    }
}
