use super::enumerate::enumerate_devices;
use hidapi::HidApi;
use lianli_shared::device_id::uses_hid;
use lianli_transport::RusbHidTransport;
use rusb::{Device, GlobalContext};
use std::collections::HashSet;
use std::time::Duration;
use tracing::{info, warn};

/// Ensure all known HID devices have hidraw nodes by performing USB resets
/// on devices the kernel failed to bind. Some devices persist malformed HID
/// report descriptors across reboots; a USB port reset restores them.
pub fn ensure_hid_devices_bound() {
    let usb_hid_devices: Vec<(u16, u16, &str)> = match enumerate_devices() {
        Ok(devs) => devs
            .into_iter()
            .filter(|d| uses_hid(d.family))
            .map(|d| (d.vid, d.pid, d.name))
            .collect(),
        Err(_) => return,
    };

    if usb_hid_devices.is_empty() {
        return;
    }

    let api = match HidApi::new() {
        Ok(api) => api,
        Err(_) => return,
    };

    let hid_vids_pids: HashSet<(u16, u16)> = api
        .device_list()
        .map(|d| (d.vendor_id(), d.product_id()))
        .collect();

    let mut reset_count = 0u32;
    for (vid, pid, name) in &usb_hid_devices {
        if hid_vids_pids.contains(&(*vid, *pid)) {
            continue;
        }
        info!("No hidraw node for {name} ({vid:04x}:{pid:04x}), performing USB reset");
        let dev = rusb::devices().ok().and_then(|devs| {
            devs.iter().find(|d| {
                d.device_descriptor()
                    .map(|desc| desc.vendor_id() == *vid && desc.product_id() == *pid)
                    .unwrap_or(false)
            })
        });
        if let Some(usb_dev) = dev {
            if device_responds_to_descriptor(&usb_dev) {
                nudge_kernel_to_bind_hid(&usb_dev);
                if poll_for_hidraw(*vid, *pid, Duration::from_millis(2000)) {
                    info!("{name}: hidraw appeared after kernel bind, no reset needed");
                    continue;
                }
                info!("{name}: hidraw still missing after wait, falling through to reset");
            }
            match RusbHidTransport::reset_usb_device(&usb_dev) {
                Ok(()) => {
                    info!("USB reset successful for {name}");
                    reset_count += 1;
                }
                Err(e) => {
                    let msg = format!("{e}");
                    if msg.contains("Entity not found") {
                        info!("USB reset successful for {name} (device re-enumerated)");
                        reset_count += 1;
                    } else {
                        warn!("USB reset failed for {name}: {e}");
                    }
                }
            }
        }
    }

    if reset_count > 0 {
        info!("Waiting 3s for {reset_count} device(s) to re-enumerate after USB reset");
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}

fn nudge_kernel_to_bind_hid(device: &Device<GlobalContext>) {
    let Ok(handle) = device.open() else { return };
    let Ok(config) = device.active_config_descriptor() else {
        return;
    };
    for iface in config.interfaces() {
        for desc in iface.descriptors() {
            if desc.class_code() == 0x03 {
                let _ = handle.attach_kernel_driver(desc.interface_number());
            }
        }
    }
}

fn poll_for_hidraw(vid: u16, pid: u16, max_wait: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < max_wait {
        if let Ok(api) = HidApi::new() {
            if api
                .device_list()
                .any(|d| d.vendor_id() == vid && d.product_id() == pid)
            {
                return true;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn device_responds_to_descriptor(device: &Device<GlobalContext>) -> bool {
    let Ok(desc) = device.device_descriptor() else {
        return false;
    };
    let Ok(handle) = device.open() else {
        return false;
    };
    let langs = match handle.read_languages(Duration::from_millis(250)) {
        Ok(l) if !l.is_empty() => l,
        _ => return desc.product_string_index().is_some(),
    };
    if let Some(_idx) = desc.product_string_index() {
        return handle
            .read_product_string(langs[0], &desc, Duration::from_millis(250))
            .is_ok();
    }
    true
}
