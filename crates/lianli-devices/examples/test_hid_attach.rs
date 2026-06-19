//! Quick test: enumerate HID devices, try USB reset, hidapi open, rusb transport.
//! Run with: cargo run --example test_hid_attach -p lianli-devices

fn main() {
    println!("=== HID Attach Test ===\n");

    // Step 1: List all USB devices with HID interfaces
    let devices = rusb::devices().expect("Failed to enumerate USB devices");
    let mut hid_devices: Vec<(u16, u16, u8, u8, String)> = Vec::new();

    for device in devices.iter() {
        let desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        let vid = desc.vendor_id();
        let pid = desc.product_id();

        let config = match device.active_config_descriptor() {
            Ok(c) => c,
            Err(_) => continue,
        };

        for iface in config.interfaces() {
            for iface_desc in iface.descriptors() {
                if iface_desc.class_code() == 0x03 {
                    let iface_num = iface_desc.interface_number();
                    let name = match device.open() {
                        Ok(h) => h
                            .read_product_string_ascii(&desc)
                            .unwrap_or_else(|_| format!("{:04x}:{:04x}", vid, pid)),
                        Err(_) => format!("{:04x}:{:04x}", vid, pid),
                    };
                    hid_devices.push((vid, pid, iface_num, device.bus_number(), name));
                }
            }
        }
    }

    if hid_devices.is_empty() {
        println!("No USB devices with HID interfaces found.");
        return;
    }

    println!("Found {} HID interface(s):\n", hid_devices.len());
    for (i, (vid, pid, iface, bus, name)) in hid_devices.iter().enumerate() {
        println!("  [{i}] {name} ({vid:04x}:{pid:04x}) bus={bus} interface={iface}");
    }

    print!("\nSelect device [0-{}]: ", hid_devices.len() - 1);
    use std::io::Write;
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let idx: usize = match input.trim().parse() {
        Ok(i) if i < hid_devices.len() => i,
        _ => {
            println!("Invalid selection");
            return;
        }
    };
    let (vid, pid, iface_num, _, name) = &hid_devices[idx];
    let vid = *vid;
    let pid = *pid;
    let iface_num = *iface_num;
    println!("\n--- Testing with: {name} ({vid:04x}:{pid:04x}) interface={iface_num} ---\n");

    // Step 0: USB device reset (fixes devices with malformed HID descriptors)
    println!("[0] Trying USB device reset (USBDEVFS_RESET)...");
    let find_usb_device = || {
        rusb::devices().ok()?.iter().find(|d| {
            d.device_descriptor()
                .map(|desc| desc.vendor_id() == vid && desc.product_id() == pid)
                .unwrap_or(false)
        })
    };
    if let Some(usb_dev) = find_usb_device() {
        match lianli_transport::RusbHidTransport::reset_usb_device(&usb_dev) {
            Ok(()) => {
                println!("    USB reset successful. Waiting 3 seconds for re-enumeration...");
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
            Err(e) => println!("    USB reset failed: {e}"),
        }
    } else {
        println!("    Could not find device via rusb");
    }

    // Step 1: Try hidapi::open(vid, pid)
    println!("\n[1] Trying hidapi::open({vid:04x}, {pid:04x})...");
    match hidapi::HidApi::new() {
        Ok(api) => match api.open(vid, pid) {
            Ok(dev) => {
                println!("    SUCCESS: hidapi opened device");
                let info = dev.get_device_info();
                if let Ok(info) = info {
                    println!(
                        "    Manufacturer: {:?}, Product: {:?}",
                        info.manufacturer_string(),
                        info.product_string()
                    );
                }
                drop(dev);
            }
            Err(e) => println!("    FAILED: {e}"),
        },
        Err(e) => println!("    FAILED to init hidapi: {e}"),
    }

    // Step 2: Try rusb detach kernel driver, then hidapi::open
    println!("\n[2] Trying rusb detach kernel driver, then hidapi::open...");
    if let Some(usb_dev) = find_usb_device() {
        match usb_dev.open() {
            Ok(handle) => match handle.kernel_driver_active(iface_num) {
                Ok(true) => {
                    println!("    Kernel driver IS active on interface {iface_num}");
                    match handle.detach_kernel_driver(iface_num) {
                        Ok(()) => {
                            println!("    Detached kernel driver successfully");

                            println!("    Retrying hidapi::open({vid:04x}, {pid:04x})...");
                            match hidapi::HidApi::new() {
                                Ok(api) => match api.open(vid, pid) {
                                    Ok(dev) => {
                                        println!("    SUCCESS: hidapi opened after detach!");
                                        drop(dev);
                                    }
                                    Err(e) => println!("    FAILED: {e}"),
                                },
                                Err(e) => println!("    hidapi init failed: {e}"),
                            }

                            let _ = handle.attach_kernel_driver(iface_num);
                            println!("    Re-attached kernel driver");
                        }
                        Err(e) => println!("    Failed to detach: {e}"),
                    }
                }
                Ok(false) => {
                    println!("    Kernel driver NOT active on interface {iface_num}");
                    println!("    (hidapi should work without detaching)");
                }
                Err(rusb::Error::NotSupported) => {
                    println!("    kernel_driver_active not supported on this platform");
                }
                Err(e) => println!("    Error checking kernel driver: {e}"),
            },
            Err(e) => println!("    Failed to open USB device: {e}"),
        }
    } else {
        println!("    Could not find device via rusb");
    }

    // Step 3: Try RusbHidTransport (our actual transport layer)
    println!("\n[3] Trying RusbHidTransport::open_by_usage...");
    if let Some(usb_dev) = find_usb_device() {
        match lianli_transport::RusbHidTransport::open_by_usage(usb_dev, None) {
            Ok(transport) => {
                println!("    SUCCESS: RusbHidTransport opened");

                println!("    Trying write + read...");
                let mut pkt = [0u8; 64];
                pkt[0] = 0x01; // Report ID
                match transport.write(&pkt) {
                    Ok(n) => println!("    Write: {n} bytes sent"),
                    Err(e) => println!("    Write failed: {e}"),
                }

                let mut buf = [0u8; 64];
                match transport.read_timeout(&mut buf, 500) {
                    Ok(0) => println!("    Read: timeout (no response, but transport works)"),
                    Ok(n) => {
                        println!("    Read: {n} bytes");
                        println!("    Data: {:02x?}", &buf[..n.min(32)]);
                    }
                    Err(e) => println!("    Read failed: {e}"),
                }

                drop(transport);
                println!("    Dropped transport (kernel driver re-attached)");
            }
            Err(e) => println!("    FAILED: {e}"),
        }
    }

    println!("\n=== Done ===");
}
