use super::enumerate::{find_hid_devices_by_family, find_usb_device};
use super::{DetectedDevice, DetectedHidDevice};
use anyhow::Result;
use hidapi::HidApi;
use lianli_shared::device_id::DeviceFamily;
use lianli_transport::{HidBackend, HidBackendKind, HidReopener, RusbHidTransport};
use parking_lot::Mutex;
use rusb::{Device, GlobalContext};
use std::sync::Arc;
use std::time::Duration;
use tracing::warn;

/// Build a reopener that re-acquires the same HID device via hidapi by VID/PID.
/// Used to recover from stale handles after USB suspend/resume.
fn make_hidapi_reopener(path: std::ffi::CString) -> HidReopener {
    Arc::new(move || {
        let api = HidApi::new().map_err(|e| anyhow::anyhow!("hidapi init: {e}"))?;
        let dev = api
            .open_path(&path)
            .map_err(|e| anyhow::anyhow!("hidapi open_path {:?}: {e}", path))?;
        Ok(HidBackendKind::Hidapi(dev))
    })
}

/// Build a reopener that re-acquires the same HID device via the rusb backend.
/// Matches by USB topology (bus + port_numbers) to disambiguate multiple devices
/// sharing the same VID:PID (e.g. daisy-chained TL LCD fans).
fn make_rusb_reopener(
    vid: u16,
    pid: u16,
    bus: u8,
    port_numbers: Vec<u8>,
    usage_page: Option<u16>,
) -> HidReopener {
    Arc::new(move || {
        let usb_dev = rusb::devices()
            .map_err(|e| anyhow::anyhow!("rusb devices: {e}"))?
            .iter()
            .find(|d| {
                d.bus_number() == bus
                    && d.port_numbers().ok().as_deref() == Some(&port_numbers[..])
                    && d.device_descriptor()
                        .map(|desc| desc.vendor_id() == vid && desc.product_id() == pid)
                        .unwrap_or(false)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "USB device {vid:04x}:{pid:04x} at {bus}-{:?} not enumerable on reopen",
                    port_numbers
                )
            })?;
        let transport = RusbHidTransport::open_by_usage(usb_dev, usage_page)
            .map_err(|e| anyhow::anyhow!("rusb hid open: {e}"))?;
        Ok(HidBackendKind::Rusb(transport))
    })
}

/// Try opening a device, retrying on failure. First two retries are plain
/// reopens, only the last retry does a USB port reset.
fn try_open_with_retry<T>(
    usb_device: Option<&Device<GlobalContext>>,
    label: &str,
    mut open_fn: impl FnMut() -> Result<T>,
) -> Result<T> {
    const MAX_RETRIES: u32 = 3;
    const RESET_AT: u32 = 2;
    for attempt in 0..=MAX_RETRIES {
        match open_fn() {
            Ok(t) => return Ok(t),
            Err(e) if attempt < MAX_RETRIES => {
                if attempt == RESET_AT {
                    if let Some(usb_dev) = usb_device {
                        warn!(
                            "{label}: open attempt {} failed: {e}, resetting USB device",
                            attempt + 1
                        );
                        let _ = RusbHidTransport::reset_usb_device(usb_dev);
                        std::thread::sleep(Duration::from_secs(3));
                    } else {
                        return Err(e.context(format!(
                            "{label}: failed and no USB device available for reset"
                        )));
                    }
                } else {
                    warn!(
                        "{label}: open attempt {} failed: {e}, retrying",
                        attempt + 1
                    );
                    std::thread::sleep(Duration::from_millis(250));
                }
            }
            Err(e) => {
                return Err(e.context(format!(
                    "{label}: failed after {} attempts",
                    MAX_RETRIES + 1
                )));
            }
        }
    }
    unreachable!()
}

fn open_with_retry<T>(
    usb_device: &Device<GlobalContext>,
    open_fn: impl FnMut() -> Result<T>,
) -> Result<T> {
    try_open_with_retry(Some(usb_device), "rusb open", open_fn)
}

/// Open a detected HID device as an LCD controller via hidapi.
pub fn open_hid_lcd_device(
    api: &HidApi,
    det: &DetectedHidDevice,
) -> Option<Result<Box<dyn crate::traits::LcdDevice>>> {
    let pid = det.pid;
    match det.family {
        DeviceFamily::HydroShiftLcd | DeviceFamily::Galahad2Lcd => {
            Some(open_hidapi_with_retry(api, det, |backend| {
                let backend = Arc::new(Mutex::new(backend));
                crate::hydroshift_lcd::HydroShiftLcdController::new(backend, pid)
                    .map(|d| Box::new(d) as Box<dyn crate::traits::LcdDevice>)
            }))
        }
        DeviceFamily::TlLcd => Some(open_hidapi_with_retry(api, det, |backend| {
            let backend = Arc::new(Mutex::new(backend));
            let mut tl = crate::tl_lcd::TlLcdDevice::new(backend);
            crate::traits::LcdDevice::initialize(&mut tl)?;
            Ok(Box::new(tl) as Box<dyn crate::traits::LcdDevice>)
        })),
        _ => None,
    }
}

fn usb_topology_string(bus: u8, port_numbers: &[u8]) -> String {
    format!(
        "{}-{}",
        bus,
        port_numbers
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(".")
    )
}

pub fn hidraw_path_for_usb_topology(bus: u8, port_numbers: &[u8]) -> Option<std::ffi::CString> {
    if port_numbers.is_empty() {
        return None;
    }
    let topology = usb_topology_string(bus, port_numbers);
    let needles = [format!("/{topology}/"), format!("/{topology}:")];
    let class_dir = std::path::Path::new("/sys/class/hidraw");
    for entry in std::fs::read_dir(class_dir).ok()?.flatten() {
        let Ok(resolved) = std::fs::canonicalize(entry.path()) else {
            continue;
        };
        let resolved_str = resolved.to_string_lossy();
        if needles.iter().any(|n| resolved_str.contains(n.as_str())) {
            let name = entry.file_name();
            let name = name.to_str()?;
            return std::ffi::CString::new(format!("/dev/{name}")).ok();
        }
    }
    None
}

/// Open a HID LCD device by USB topology (bus + port path).
/// Picks the specific hidraw belonging to that USB device, then opens it via hidapi.
/// Required for devices like TL LCD where multiple physical units share VID:PID.
pub fn open_hid_lcd_by_topology(
    vid: u16,
    pid: u16,
    family: DeviceFamily,
    bus: u8,
    port_numbers: &[u8],
) -> Result<Box<dyn crate::traits::LcdDevice>> {
    let target_path = hidraw_path_for_usb_topology(bus, port_numbers).ok_or_else(|| {
        anyhow::anyhow!(
            "no hidraw matching USB topology {} for {vid:04x}:{pid:04x}",
            usb_topology_string(bus, port_numbers)
        )
    })?;
    let api = HidApi::new().map_err(|e| anyhow::anyhow!("hidapi init: {e}"))?;
    let det = find_hid_devices_by_family(&api, family)
        .into_iter()
        .find(|d| d.path == target_path)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no enumerated HID device matching path {:?} for {vid:04x}:{pid:04x}",
                target_path
            )
        })?;
    match open_hid_lcd_device(&api, &det) {
        Some(Ok(ctrl)) => Ok(ctrl),
        Some(Err(e)) => Err(e.context("HID LCD open by topology")),
        None => Err(anyhow::anyhow!("family does not support LCD")),
    }
}

/// Open a HID LCD device by VID/PID using hidapi with retry logic.
///
/// Unlike `open_hid_lcd_device` (which requires a pre-enumerated `DetectedHidDevice`),
/// this function handles the case where no hidraw node exists yet by performing
/// USB reset + re-enumeration before retrying.
pub fn open_hid_lcd_by_vid_pid(
    vid: u16,
    pid: u16,
    family: DeviceFamily,
) -> Result<Box<dyn crate::traits::LcdDevice>> {
    let usb_device = find_usb_device(vid, pid);

    for attempt in 0..=3u32 {
        let api = HidApi::new().map_err(|e| anyhow::anyhow!("hidapi init: {e}"))?;
        let hid_devs = find_hid_devices_by_family(&api, family);

        if let Some(det) = hid_devs.into_iter().next() {
            match open_hid_lcd_device(&api, &det) {
                Some(Ok(ctrl)) => return Ok(ctrl),
                Some(Err(e)) if attempt < 3 => {
                    warn!(
                        "HID LCD open attempt {} failed ({vid:04x}:{pid:04x}): {e}, resetting USB",
                        attempt + 1
                    );
                }
                Some(Err(e)) => {
                    return Err(e.context("HID LCD open failed after 4 attempts"));
                }
                None => {
                    return Err(anyhow::anyhow!("family does not support LCD"));
                }
            }
        } else if attempt < 3 {
            warn!(
                "No hidraw node for {:04x}:{:04x} (attempt {}), resetting USB",
                vid,
                pid,
                attempt + 1
            );
        } else {
            return Err(anyhow::anyhow!(
                "no HID device found for {vid:04x}:{pid:04x} after 4 attempts"
            ));
        }

        if let Some(ref usb_dev) = usb_device {
            let _ = RusbHidTransport::reset_usb_device(usb_dev);
            std::thread::sleep(Duration::from_secs(3));
        } else {
            return Err(anyhow::anyhow!(
                "no USB device found for reset ({vid:04x}:{pid:04x})"
            ));
        }
    }
    unreachable!()
}

/// Open a detected HID device as an LCD controller via rusb.
pub fn open_hid_lcd_device_rusb(
    det: &DetectedDevice,
) -> Option<Result<Box<dyn crate::traits::LcdDevice>>> {
    match det.family {
        DeviceFamily::HydroShiftLcd | DeviceFamily::Galahad2Lcd => {
            let pid = det.pid;
            let bus = det.bus;
            let port_numbers = det.device.port_numbers().unwrap_or_default();
            Some(open_with_retry(&det.device, || {
                let transport =
                    RusbHidTransport::open_by_usage(det.device.clone(), det.hid_usage_page)?;
                let mut backend =
                    HidBackend::from_rusb(transport).with_reopener(make_rusb_reopener(
                        det.vid,
                        det.pid,
                        bus,
                        port_numbers.clone(),
                        det.hid_usage_page,
                    ));
                backend.read_flush();
                let backend = Arc::new(Mutex::new(backend));
                crate::hydroshift_lcd::HydroShiftLcdController::new(backend, pid)
                    .map(|d| Box::new(d) as Box<dyn crate::traits::LcdDevice>)
            }))
        }
        DeviceFamily::TlLcd => {
            let bus = det.bus;
            let port_numbers = det.device.port_numbers().unwrap_or_default();
            Some(open_with_retry(&det.device, || {
                let transport =
                    RusbHidTransport::open_by_usage(det.device.clone(), det.hid_usage_page)?;
                let backend = HidBackend::from_rusb(transport).with_reopener(make_rusb_reopener(
                    det.vid,
                    det.pid,
                    bus,
                    port_numbers.clone(),
                    det.hid_usage_page,
                ));
                let backend = Arc::new(Mutex::new(backend));
                let mut tl = crate::tl_lcd::TlLcdDevice::new(backend);
                crate::traits::LcdDevice::initialize(&mut tl)?;
                Ok(Box::new(tl) as Box<dyn crate::traits::LcdDevice>)
            }))
        }
        _ => None,
    }
}

/// Wrap hidapi open with retry logic. On failure, performs USB reset and retries.
pub fn open_hidapi_with_retry<T>(
    api: &HidApi,
    det: &DetectedHidDevice,
    mut create_fn: impl FnMut(HidBackend) -> Result<T>,
) -> Result<T> {
    let usb_device = find_usb_device(det.vid, det.pid);
    let label = format!("HID open {} ({:04x}:{:04x})", det.name, det.vid, det.pid);

    let backend = try_open_with_retry(usb_device.as_ref(), &label, || {
        let hid_dev = api
            .open_path(&det.path)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut backend =
            HidBackend::from_hidapi(hid_dev).with_reopener(make_hidapi_reopener(det.path.clone()));
        backend.read_flush();
        Ok(backend)
    })?;
    create_fn(backend)
}

/// Open a shared HID backend via hidapi with retry logic.
/// Returns an `Arc<Mutex<HidBackend>>` that can be shared between multiple controllers.
pub fn open_hid_backend_hidapi(
    api: &HidApi,
    det: &DetectedHidDevice,
) -> Result<Arc<Mutex<HidBackend>>> {
    open_hidapi_with_retry(api, det, |backend| Ok(Arc::new(Mutex::new(backend))))
}

/// Open a shared HID backend via rusb with retry logic.
/// Returns an `Arc<Mutex<HidBackend>>` that can be shared between multiple controllers.
pub fn open_hid_backend_rusb(det: &DetectedDevice) -> Result<Arc<Mutex<HidBackend>>> {
    let vid = det.vid;
    let pid = det.pid;
    let bus = det.bus;
    let port_numbers = det.device.port_numbers().unwrap_or_default();
    let usage_page = det.hid_usage_page;
    open_with_retry(&det.device, || {
        let transport = RusbHidTransport::open_by_usage(det.device.clone(), det.hid_usage_page)?;
        let backend = HidBackend::from_rusb(transport).with_reopener(make_rusb_reopener(
            vid,
            pid,
            bus,
            port_numbers.clone(),
            usage_page,
        ));
        Ok(Arc::new(Mutex::new(backend)))
    })
}
