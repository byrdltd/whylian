use super::{DetectedDevice, DetectedHidDevice};
use anyhow::Result;
use hidapi::HidApi;
use lianli_shared::device_id::{uses_hid, DeviceFamily, UsbId, KNOWN_DEVICES};
use parking_lot::Mutex;
use rusb::{Device, GlobalContext};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, warn};

/// Look up the USB port path for a device by VID/PID. Returns e.g. "1-5.3"
/// (bus-port topology), stable across reboots.
pub(super) fn usb_port_path(vid: u16, pid: u16) -> Option<String> {
    let devices = rusb::devices().ok()?;
    for device in devices.iter() {
        let desc = device.device_descriptor().ok()?;
        if desc.vendor_id() == vid && desc.product_id() == pid {
            let bus = device.bus_number();
            let ports = device.port_numbers().ok()?;
            if !ports.is_empty() {
                let parts: Vec<String> = ports.iter().map(|p| p.to_string()).collect();
                return Some(format!("{}-{}", bus, parts.join(".")));
            }
        }
    }
    None
}

/// Enumerate all Lian Li USB devices on the system.
pub fn enumerate_devices() -> Result<Vec<DetectedDevice>> {
    let usb_devices = rusb::devices()?;
    let mut found = Vec::new();

    for device in usb_devices.iter() {
        let desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let vid = desc.vendor_id();
        let pid = desc.product_id();
        let id = UsbId::new(vid, pid);

        if let Some(entry) = KNOWN_DEVICES.iter().find(|e| e.id == id) {
            let bus = device.bus_number();
            let address = device.address();

            let serial = device
                .open()
                .ok()
                .and_then(|h| h.read_serial_number_string_ascii(&desc).ok());

            debug!(
                "Found {} ({:04x}:{:04x}) at bus {} addr {} serial={}",
                entry.name,
                vid,
                pid,
                bus,
                address,
                serial.as_deref().unwrap_or("none")
            );

            found.push(DetectedDevice {
                device,
                family: entry.family,
                name: entry.name,
                vid,
                pid,
                bus,
                address,
                serial,
                hid_usage_page: entry.hid_usage_page,
            });
        }
    }

    found.sort_by_key(|d| (d.bus, d.address));
    Ok(found)
}

/// Enumerate all known Lian Li HID devices.
pub fn enumerate_hid_devices(api: &HidApi) -> Vec<DetectedHidDevice> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();

    for dev_info in api.device_list() {
        let vid = dev_info.vendor_id();
        let pid = dev_info.product_id();
        let id = UsbId::new(vid, pid);

        if let Some(entry) = KNOWN_DEVICES.iter().find(|e| e.id == id) {
            if !uses_hid(entry.family) {
                continue;
            }

            if let Some(required_page) = entry.hid_usage_page {
                if dev_info.usage_page() != required_page {
                    continue;
                }
            } else {
                let path = dev_info.path().to_owned();
                if !seen.insert((vid, pid, path)) {
                    continue;
                }
            }

            let serial = dev_info.serial_number().map(|s| s.to_string());

            debug!(
                "Found HID {} ({:04x}:{:04x}) usage_page={:#06x} path={:?} serial={:?}",
                entry.name,
                vid,
                pid,
                dev_info.usage_page(),
                dev_info.path(),
                serial
            );

            found.push(DetectedHidDevice {
                family: entry.family,
                name: entry.name,
                vid,
                pid,
                path: dev_info.path().to_owned(),
                usb_port_path: usb_port_path(vid, pid),
                serial,
                port_index: None,
            });
        }
    }

    found
}

/// Probe TL LCD identities via rusb. Hidapi reads alias across daisy-chained
/// TL LCDs (3+ devices sharing iSerial collide somewhere below us), so the
/// daemon always uses rusb for this family regardless of `hid_driver` setting.
pub fn probe_tl_lcd_port_indices_rusb(
    devices: &[DetectedDevice],
) -> std::collections::HashMap<String, (u8, u8)> {
    use lianli_transport::{HidBackend, RusbHidTransport};
    let mut out = std::collections::HashMap::new();
    for det in devices.iter().filter(|d| d.family == DeviceFamily::TlLcd) {
        let transport =
            match RusbHidTransport::open_by_usage(det.device.clone(), det.hid_usage_page) {
                Ok(t) => t,
                Err(e) => {
                    warn!("TL LCD rusb open failed for {}: {e:#}", det.device_id());
                    continue;
                }
            };
        let backend = Arc::new(Mutex::new(HidBackend::from_rusb(transport)));
        let tl = crate::tl_lcd::TlLcdDevice::new(backend);
        match tl.read_identity_raw() {
            Ok(ident) => {
                out.insert(det.device_id(), (ident.port, ident.index));
            }
            Err(e) => warn!("TL LCD identity read failed for {}: {e:#}", det.device_id()),
        }
    }
    out
}

/// Find HID devices matching a specific family.
pub(super) fn find_hid_devices_by_family(
    api: &HidApi,
    family: DeviceFamily,
) -> Vec<DetectedHidDevice> {
    enumerate_hid_devices(api)
        .into_iter()
        .filter(|d| d.family == family)
        .collect()
}

/// Find the rusb `Device` matching a VID/PID pair.
pub(super) fn find_usb_device(vid: u16, pid: u16) -> Option<Device<GlobalContext>> {
    rusb::devices().ok()?.iter().find(|d| {
        d.device_descriptor()
            .map(|desc| desc.vendor_id() == vid && desc.product_id() == pid)
            .unwrap_or(false)
    })
}
