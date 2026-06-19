use super::{
    DiskDirection, NetDirection, NvidiaMetric, PwmHeader, SensorInfo, SensorName, SensorSource,
    Unit,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn enumerate_sensors() -> Vec<SensorInfo> {
    let mut sensors = Vec::new();

    let mut mem_idx: usize = 0;
    let mut gfx_idx: usize = 0;
    let gpu_names = get_amd_gpu_names();

    sensors.push(SensorInfo {
        source: SensorSource::CpuUsage,
        sensor_name: None,
        display_name: Some("CPU: Usage".to_string()),
        divider: 100,
        unit: Unit::PERCENT,
        current_value: Some(0.0),
    });
    sensors.push(SensorInfo {
        source: SensorSource::MemUsage,
        sensor_name: None,
        display_name: Some("RAM: Usage".to_string()),
        divider: 1,
        unit: Unit::PERCENT,
        current_value: Some(0.0),
    });
    sensors.push(SensorInfo {
        source: SensorSource::MemUsed,
        sensor_name: None,
        display_name: Some("RAM: Used".to_string()),
        divider: 1024 * 1024,
        unit: Unit::SIZE,
        current_value: Some(0.0),
    });
    sensors.push(SensorInfo {
        source: SensorSource::MemFree,
        sensor_name: None,
        display_name: Some("RAM: Free".to_string()),
        divider: 1024 * 1024,
        unit: Unit::SIZE,
        current_value: Some(0.0),
    });

    let hwmon_path = "/sys/class/hwmon/";
    if let Ok(entries) = std::fs::read_dir(hwmon_path) {
        let mut sorted_entries: Vec<_> = entries.flatten().collect();
        sorted_entries.sort_by_cached_key(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .strip_prefix("hwmon")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(u32::MAX)
        });
        for entry in sorted_entries {
            let path = entry.path();
            let name = match std::fs::read_to_string(path.join("name")) {
                Ok(n) => n.trim().to_string(),
                Err(_) => continue,
            };

            let pci_id = get_pci_id_from_path(path.clone());
            let pci_id_stripped = pci_id.strip_prefix("0000:").unwrap_or(&pci_id).to_string();

            let result = get_display_name(&path, &pci_id_stripped, &gpu_names, mem_idx, gfx_idx);
            mem_idx = result.1;
            gfx_idx = result.2;
            let display_name = match result.0 {
                Some(name) => name,
                None => continue,
            };

            let device_path = std::fs::read_link(path.join("device"))
                .ok()
                .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()));

            if let Ok(files) = std::fs::read_dir(&path) {
                let mut device_sensors: Vec<SensorInfo> = Vec::new();

                for file in files.flatten() {
                    let fname = file.file_name().to_string_lossy().to_string();
                    if fname.ends_with("_input") {
                        let prefix = fname.strip_suffix("_input").unwrap();
                        let label = std::fs::read_to_string(path.join(format!("{}_label", prefix)))
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|_| "".to_string());
                        let display_label = get_label_name(prefix, &label);
                        let (unit, divider) = get_unit(prefix);
                        let value = read_sysfs_file(&file.path()).map(|v| v / divider as f32);
                        let sensor_name = Some(SensorName {
                            device_name: display_name.clone(),
                            sensor_name: display_label,
                        });
                        let device_path = if let Some(dev) = &device_path {
                            if dev.starts_with("DEADBEEF") {
                                pci_id.to_string()
                            } else {
                                dev.to_string()
                            }
                        } else {
                            pci_id.to_string()
                        };

                        device_sensors.push(SensorInfo {
                            source: SensorSource::Hwmon {
                                name: name.clone(),
                                label: prefix.to_string(),
                                device_path,
                            },
                            sensor_name,
                            display_name: None,
                            divider,
                            unit,
                            current_value: value,
                        });
                    }
                }

                device_sensors.sort_by_cached_key(|s| s.get_display_name());
                sensors.extend(device_sensors);
            }
        }
    }

    if let Ok(output) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,name,temperature.gpu,utilization.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(", ").collect();
                if parts.len() >= 4 {
                    let gpu_index: u32 = parts[0].trim().parse().unwrap_or(0);
                    let gpu_name = parts[1].trim();
                    let temp: Option<f32> = parts[2].trim().parse().ok();
                    let usage: Option<f32> = parts[3].trim().parse().ok();

                    sensors.push(SensorInfo {
                        source: SensorSource::NvidiaGpu {
                            gpu_index,
                            metric: NvidiaMetric::Temp,
                        },
                        sensor_name: None,
                        display_name: Some(format!("{gpu_name}: Temp")),
                        current_value: temp,
                        unit: Unit::C,
                        divider: 1,
                    });

                    sensors.push(SensorInfo {
                        source: SensorSource::NvidiaGpu {
                            gpu_index,
                            metric: NvidiaMetric::Usage,
                        },
                        sensor_name: None,
                        display_name: Some(format!("{gpu_name}: Usage")),
                        current_value: usage,
                        unit: Unit::PERCENT,
                        divider: 1,
                    });
                }
            }
        }
    }

    enumerate_amd_gpu_usage(&gpu_names, &mut sensors);
    enumerate_network_sensors(&mut sensors);
    enumerate_disk_sensors(&mut sensors);

    sensors
}

fn enumerate_amd_gpu_usage(gpu_names: &HashMap<String, String>, sensors: &mut Vec<SensorInfo>) {
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return;
    };
    let mut cards: Vec<(u32, std::path::PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let idx: u32 = name.strip_prefix("card")?.parse().ok()?;
            Some((idx, e.path()))
        })
        .collect();
    cards.sort_by_key(|(idx, _)| *idx);

    for (card_index, card_path) in cards {
        let busy_path = card_path.join("device/gpu_busy_percent");
        if !busy_path.exists() {
            continue;
        }
        let vendor = std::fs::read_to_string(card_path.join("device/vendor"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if vendor != "0x1002" {
            continue;
        }

        let pci_id = std::fs::read_link(card_path.join("device"))
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
            .and_then(|s| s.strip_prefix("0000:").map(|t| t.to_string()));
        let name = pci_id
            .as_ref()
            .and_then(|id| gpu_names.get(id).cloned())
            .unwrap_or_else(|| format!("AMD GPU {card_index}"));

        let current_value = std::fs::read_to_string(&busy_path)
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok());

        sensors.push(SensorInfo {
            source: SensorSource::AmdGpuUsage { card_index },
            sensor_name: None,
            display_name: Some(format!("{name}: Usage")),
            current_value,
            unit: Unit::PERCENT,
            divider: 1,
        });
    }
}

pub fn get_pci_id_from_path(hwmon_path: PathBuf) -> String {
    let device_path = hwmon_path.join("device");

    let opt_full_path = std::fs::canonicalize(device_path).ok();
    if opt_full_path.is_none() {
        return "None".to_string();
    }

    let full_path = opt_full_path.unwrap();

    for component in full_path.ancestors() {
        if let Some(name_os) = component.file_name() {
            let name = name_os.to_string_lossy();
            if name.contains(':') && name.contains('.') && name.len() >= 7 {
                return name.into_owned();
            }
        }
        if component == Path::new("/sys/devices") {
            break;
        }
    }

    let name = std::fs::read_to_string(hwmon_path.join("name"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    format!("platform:{}", name)
}

pub(super) fn get_unit(prefix: &str) -> (Unit, usize) {
    if prefix.starts_with("temp") {
        (Unit::C, 1000)
    } else if prefix.starts_with("fan") {
        (Unit::RPM, 1)
    } else if prefix.starts_with("in") {
        (Unit::V, 1)
    } else if prefix.starts_with("freq") {
        (Unit::FREQ, 1000 * 1000)
    } else {
        (Unit::WO, 1)
    }
}

pub fn get_label_name(prefix: &str, label: &str) -> String {
    let lower_label = label.to_lowercase();
    let lower_prefix = prefix.to_lowercase();
    if lower_label.ends_with("ctl") || lower_label.ends_with("package id 0") {
        "Control Temp".to_string()
    } else if lower_label.ends_with("junction") && lower_prefix.starts_with("temp") {
        "Hotspot Temp".to_string()
    } else if lower_label.ends_with("edge") && lower_prefix.starts_with("temp") {
        "Edge Temp".to_string()
    } else if lower_label.ends_with("mem") && lower_prefix.starts_with("temp") {
        "VRAM Temp".to_string()
    } else if lower_label.ends_with("sclk") && lower_prefix.starts_with("freq") {
        "System Clock".to_string()
    } else if lower_label.ends_with("mclk") && lower_prefix.starts_with("freq") {
        "Memory Clock".to_string()
    } else if lower_label.ends_with("vddgfx") && lower_prefix.starts_with("in") {
        "GPU Voltage".to_string()
    } else if let Some(idx) = lower_label.find("ccd") {
        format!("Temp Die {}", &lower_label[idx + 3..])
    } else if let Some(idx) = lower_label.find("core ") {
        format!("Temp Core {}", &lower_label[idx + 5..])
    } else if let Some(idx) = lower_label.find("fan") {
        format!("Fan {}", &lower_label[idx + 3..])
    } else if let Some(idx) = lower_prefix.find("fan") {
        format!("Fan {}", &lower_prefix[idx + 3..])
    } else if label.is_empty() {
        prefix.to_string()
    } else {
        label.to_string()
    }
}

fn get_amd_gpu_names() -> HashMap<String, String> {
    let mut gpus = HashMap::new();

    let output = match Command::new("lspci").output() {
        Ok(o) => o,
        Err(_) => return gpus,
    };
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let line_lower = line.to_lowercase();
        if (line_lower.contains("vga") || line_lower.contains("display"))
            && line_lower.contains("amd")
        {
            if let Some((addr, full_desc)) = line.split_once(' ') {
                let clean_name = if let Some((_, actual_name)) = full_desc.split_once(": ") {
                    actual_name.trim()
                } else {
                    full_desc.trim()
                };
                gpus.insert(addr.to_string(), clean_name.to_string());
            }
        }
    }

    clean_common_prefixes(gpus)
}

fn clean_common_prefixes(mut gpus: HashMap<String, String>) -> HashMap<String, String> {
    if gpus.len() <= 1 {
        return gpus;
    }

    let values: Vec<&String> = gpus.values().collect();
    let mut common_prefix = values[0].clone();

    for name in values.iter().skip(1) {
        while !name.starts_with(&common_prefix) && !common_prefix.is_empty() {
            common_prefix.pop();
        }
    }

    if !common_prefix.is_empty() {
        let prefix_len = common_prefix.len();
        for value in gpus.values_mut() {
            *value = value[prefix_len..].trim().to_string();
        }
    }

    gpus
}

pub fn get_display_name(
    hwmon_path: &Path,
    pci_id_stripped: &str,
    gpu_names: &HashMap<String, String>,
    mem_idx: usize,
    gfx_idx: usize,
) -> (Option<String>, usize, usize) {
    let model_path = hwmon_path.join("device").join("model");

    if let Ok(model_name) = std::fs::read_to_string(model_path) {
        return (Some(model_name.trim().to_string()), mem_idx, gfx_idx);
    }

    if let Ok(generic_name) = std::fs::read_to_string(hwmon_path.join("name")) {
        let name = generic_name.trim();
        if name == "nvme" {
            return (Some("NVMe Storage Device".to_string()), mem_idx, gfx_idx);
        }
        if matches!(
            name,
            "k10temp" | "k8temp" | "coretemp" | "zenpower" | "zenpower3"
        ) {
            return (Some("CPU".to_string()), mem_idx, gfx_idx);
        }
        if name == "amdgpu" {
            if let Some(gpu_name) = gpu_names.get(pci_id_stripped) {
                return (Some(gpu_name.clone()), mem_idx, gfx_idx + 1);
            }
            return (Some(format!("AMD GPU {}", gfx_idx)), mem_idx, gfx_idx + 1);
        }
        if name == "nouveau" {
            return (Some("NVidia GPU".to_string()), mem_idx, gfx_idx + 1);
        }
        let common_drivers = ["nct", "it8", "f71", "gigabyte_wmi", "w83"];
        if common_drivers.iter().any(|&d| name.starts_with(d)) {
            return (Some("Motherboard".to_string()), mem_idx, gfx_idx);
        }
        if name.starts_with("spd") {
            return (
                Some(format!("DDR5 RAM Module {}", mem_idx + 1)),
                mem_idx + 1,
                gfx_idx,
            );
        }
        if name.starts_with("ee1004") {
            return (
                Some(format!("DDR4 RAM Module {}", mem_idx + 1)),
                mem_idx + 1,
                gfx_idx,
            );
        }
        if name.starts_with("jc42") {
            return (
                Some(format!("DDR3/ECC RAM Module {}", mem_idx + 1)),
                mem_idx + 1,
                gfx_idx,
            );
        }
        if name == "acpitz" {
            return (None, mem_idx, gfx_idx);
        }

        (Some(name.to_string()), mem_idx, gfx_idx)
    } else {
        (Some("Unknown Device".to_string()), mem_idx, gfx_idx)
    }
}

fn enumerate_network_sensors(sensors: &mut Vec<SensorInfo>) {
    let Ok(content) = std::fs::read_to_string("/proc/net/dev") else {
        return;
    };
    let mut ifaces: Vec<String> = Vec::new();
    for line in content.lines() {
        let Some((name, _)) = line.split_once(':') else {
            continue;
        };
        let trimmed = name.trim();
        if trimmed == "lo" || trimmed.is_empty() {
            continue;
        }
        ifaces.push(trimmed.to_string());
    }
    ifaces.sort();
    for iface in ifaces {
        sensors.push(SensorInfo {
            source: SensorSource::NetworkRate {
                iface: iface.clone(),
                direction: NetDirection::Rx,
            },
            sensor_name: None,
            display_name: Some(format!("Network {iface}: Rx")),
            divider: 1_000_000,
            unit: Unit::MBps,
            current_value: Some(0.0),
        });
        sensors.push(SensorInfo {
            source: SensorSource::NetworkRate {
                iface: iface.clone(),
                direction: NetDirection::Tx,
            },
            sensor_name: None,
            display_name: Some(format!("Network {iface}: Tx")),
            divider: 1_000_000,
            unit: Unit::MBps,
            current_value: Some(0.0),
        });
    }
}

fn enumerate_disk_sensors(sensors: &mut Vec<SensorInfo>) {
    let Ok(entries) = std::fs::read_dir("/sys/block") else {
        return;
    };
    let mut devices: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let skip = name.starts_with("loop")
                || name.starts_with("ram")
                || name.starts_with("dm-")
                || name.starts_with("zram")
                || name.starts_with("sr");
            if skip {
                None
            } else {
                Some(name)
            }
        })
        .collect();
    devices.sort();
    for device in devices {
        sensors.push(SensorInfo {
            source: SensorSource::DiskRate {
                device: device.clone(),
                direction: DiskDirection::Read,
            },
            sensor_name: None,
            display_name: Some(format!("Disk {device}: Read")),
            divider: 1_000_000,
            unit: Unit::MBps,
            current_value: Some(0.0),
        });
        sensors.push(SensorInfo {
            source: SensorSource::DiskRate {
                device: device.clone(),
                direction: DiskDirection::Write,
            },
            sensor_name: None,
            display_name: Some(format!("Disk {device}: Write")),
            divider: 1_000_000,
            unit: Unit::MBps,
            current_value: Some(0.0),
        });
    }
}

fn read_sysfs_file(path: &Path) -> Option<f32> {
    let content = std::fs::read_to_string(path).ok()?;
    let value: f32 = content.trim().parse().ok()?;
    Some(value)
}

pub fn enumerate_pwm_headers() -> Vec<PwmHeader> {
    let gpu_names = get_amd_gpu_names();
    let mut headers = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") else {
        return headers;
    };
    let mut mem_idx = 0usize;
    let mut gfx_idx = 0usize;
    for entry in entries.flatten() {
        let dir = entry.path();
        let pci_id = dir
            .join("device")
            .read_link()
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
            .unwrap_or_default()
            .replace("0000:", "");
        let (friendly, mi, gi) = get_display_name(&dir, &pci_id, &gpu_names, mem_idx, gfx_idx);
        mem_idx = mi;
        gfx_idx = gi;
        let chip_label = friendly.unwrap_or_else(|| {
            std::fs::read_to_string(dir.join("name"))
                .unwrap_or_default()
                .trim()
                .to_string()
        });
        for i in 1..=10 {
            let pwm_path = dir.join(format!("pwm{i}"));
            if !pwm_path.exists() {
                break;
            }
            let hwmon = dir.file_name().unwrap_or_default().to_string_lossy();
            let id = format!("{hwmon}/pwm{i}");
            let label = format!("{chip_label} Fan{i}");
            headers.push(PwmHeader {
                id,
                label,
                path: pwm_path,
            });
        }
    }
    headers.sort_by(|a, b| a.id.cmp(&b.id));
    headers
}

pub fn read_pwm_header(id: &str) -> Option<u8> {
    let path = Path::new("/sys/class/hwmon").join(id);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
}
