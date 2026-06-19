use lianli_devices::wireless::{
    find_aio_radiator_cluster, pump_rpm_to_timer, DiscoveredDevice, SharedWirelessController,
    WirelessFanType, AIO_PARAM_LEN,
};
use lianli_shared::aio::AioConfig;
use lianli_shared::config::AppConfig;
use lianli_shared::fan::{FanCurve, FanSpeed};
use lianli_shared::media::SensorSourceConfig;
use lianli_shared::sensors::{
    enumerate_sensors, read_sensor_value, resolve_sensor, ResolvedSensor, SensorInfo, SensorSource,
};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::{debug, info, warn};

const TICK: Duration = Duration::from_secs(1);

pub struct AioController {
    wireless: SharedWirelessController,
    state: Arc<Mutex<State>>,
    stop_flag: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

struct State {
    config: AppConfig,
    needs_reinit: bool,
}

impl AioController {
    pub fn new(wireless: SharedWirelessController, config: AppConfig) -> Self {
        Self {
            wireless,
            state: Arc::new(Mutex::new(State {
                config,
                needs_reinit: false,
            })),
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread: None,
        }
    }

    pub fn set_config(&self, config: AppConfig) {
        let mut state = self.state.lock();
        state.config = config;
        state.needs_reinit = true;
    }

    pub fn start(&mut self) {
        if self.thread.is_some() {
            return;
        }
        let wireless = Arc::clone(&self.wireless);
        let state = Arc::clone(&self.state);
        let stop = Arc::clone(&self.stop_flag);
        self.thread = Some(thread::spawn(move || run(wireless, state, stop)));
    }

    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for AioController {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn run(wireless: SharedWirelessController, state: Arc<Mutex<State>>, stop_flag: Arc<AtomicBool>) {
    let all_sensors = enumerate_sensors();
    let mut sensor_cache: HashMap<SensorSource, ResolvedSensor> = HashMap::new();
    let mut switched: HashSet<[u8; 6]> = HashSet::new();

    while !stop_flag.load(Ordering::Relaxed) {
        let cfg = {
            let mut s = state.lock();
            if s.needs_reinit {
                switched.clear();
                s.needs_reinit = false;
            }
            s.config.clone()
        };
        let curves: HashMap<String, FanCurve> = cfg
            .fan_curves
            .iter()
            .map(|c| (c.name.clone(), c.clone()))
            .collect();
        let devices: Vec<DiscoveredDevice> = wireless.lock().devices();

        for device in &devices {
            if !device.is_aio() {
                continue;
            }
            let device_id = format!("wireless:{}", device.mac_str());
            let Some(aio_cfg) = cfg.aio.get(&device_id) else {
                continue;
            };

            if !switched.contains(&device.mac) {
                match wireless.lock().switch_to_wireless_theme(&device.mac) {
                    Ok(()) => {
                        switched.insert(device.mac);
                        info!("AIO {}: wireless theme mode engaged", device.mac_str());
                    }
                    Err(e) => {
                        warn!(
                            "AIO {}: switch_to_wireless_theme failed (will retry): {e:#}",
                            device.mac_str()
                        );
                    }
                }
            }

            let param = build_aio_param(aio_cfg, device, &curves, &mut sensor_cache, &all_sensors);
            if let Err(e) = wireless.lock().set_aio_params(&device.mac, &param) {
                warn!("AIO {}: set_aio_params failed: {e:#}", device.mac_str());
            }

            if let Some((target_mac, fan_count)) = resolve_fan_pwm_target(device, aio_cfg, &devices)
            {
                let fan_pwm =
                    compute_fan_pwm(aio_cfg, fan_count, &curves, &mut sensor_cache, &all_sensors);
                if let Err(e) = wireless.lock().set_fan_speeds_by_mac(target_mac, &fan_pwm) {
                    warn!(
                        "AIO {}: set_fan_speeds failed on {}: {e:#}",
                        device.mac_str(),
                        mac_label(target_mac, device)
                    );
                }
            }
        }

        let live_macs: HashSet<[u8; 6]> = devices.iter().map(|d| d.mac).collect();
        switched.retain(|m| live_macs.contains(m));

        thread::sleep(TICK);
    }

    debug!("AioController stopped");
}

fn mac_str(mac: &[u8; 6]) -> String {
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

fn mac_label(target_mac: &[u8; 6], aio: &DiscoveredDevice) -> String {
    if target_mac == &aio.mac {
        "integrated fans".to_string()
    } else {
        format!("radiator {}", mac_str(target_mac))
    }
}

/// Resolve which wireless MAC receives radiator fan PWM for this AIO.
fn resolve_fan_pwm_target<'a>(
    aio: &'a DiscoveredDevice,
    aio_cfg: &AioConfig,
    devices: &'a [DiscoveredDevice],
) -> Option<(&'a [u8; 6], u8)> {
    if aio.fan_count > 0 {
        return Some((&aio.mac, aio.fan_count));
    }

    if let Some(ref radiator_id) = aio_cfg.radiator_fan_device_id {
        if let Some(radiator) = devices
            .iter()
            .find(|d| format!("wireless:{}", mac_str(&d.mac)) == *radiator_id)
        {
            if radiator.fan_count > 0 {
                return Some((&radiator.mac, radiator.fan_count));
            }
        }
    }

    find_aio_radiator_cluster(aio, devices).map(|r| (&r.mac, r.fan_count))
}

fn compute_fan_pwm(
    aio_cfg: &AioConfig,
    fan_count: u8,
    curves: &HashMap<String, FanCurve>,
    sensor_cache: &mut HashMap<SensorSource, ResolvedSensor>,
    all_sensors: &[SensorInfo],
) -> [u8; 4] {
    let mut fan_pwm = [0u8; 4];
    for (i, slot) in aio_cfg.fan_speeds.iter().enumerate() {
        if (i as u8) >= fan_count {
            continue;
        }
        fan_pwm[i] = match slot {
            FanSpeed::Constant(b) => *b,
            FanSpeed::Curve(name) => match curves.get(name) {
                Some(curve) => {
                    let source = curve.effective_source();
                    let pct = match resolve_and_read(&source, sensor_cache, all_sensors) {
                        Some(temp) => interpolate_curve(&curve.curve, temp).clamp(0.0, 100.0),
                        None => 0.0,
                    };
                    (pct * 2.55) as u8
                }
                None => 0,
            },
        };
    }
    fan_pwm
}

fn build_aio_param(
    cfg: &AioConfig,
    device: &DiscoveredDevice,
    curves: &HashMap<String, FanCurve>,
    sensor_cache: &mut HashMap<SensorSource, ResolvedSensor>,
    all_sensors: &[SensorInfo],
) -> [u8; AIO_PARAM_LEN] {
    let mut p = [0u8; AIO_PARAM_LEN];

    let (cpu_temp, cpu_temp_ok) = read_optional(&cfg.cpu_temp_source, sensor_cache, all_sensors);
    let (cpu_load, cpu_load_ok) = read_optional(&cfg.cpu_load_source, sensor_cache, all_sensors);
    let (gpu_temp, gpu_temp_ok) = read_optional(&cfg.gpu_temp_source, sensor_cache, all_sensors);
    let (gpu_load, gpu_load_ok) = read_optional(&cfg.gpu_load_source, sensor_cache, all_sensors);

    p[0] = cpu_temp;
    p[1] = cpu_load;
    p[2] = gpu_temp;
    p[3] = gpu_load;
    p[6] = cfg.loop_interval;
    p[7] = 1;
    p[8] = cpu_temp_ok as u8;
    p[9] = cpu_load_ok as u8;
    p[10] = gpu_temp_ok as u8;
    p[11] = gpu_load_ok as u8;
    write_argb(&mut p[13..17], cfg.str_color);
    write_argb(&mut p[17..21], cfg.val_color);
    write_argb(&mut p[21..25], cfg.unit_color);
    p[25] = cfg.brightness.min(100);
    p[26] = 1;
    p[27] = cfg.theme_index.min(12);

    let rpm = resolve_pump_rpm(
        &cfg.pump_target_rpm,
        device.fan_type,
        curves,
        sensor_cache,
        all_sensors,
    );
    let timer = pump_rpm_to_timer(rpm, device.fan_type).unwrap_or(0);
    p[28] = (timer >> 8) as u8;
    p[29] = (timer & 0xFF) as u8;
    p[30] = cfg.rotation.min(3);
    p
}

fn read_optional(
    source: &Option<SensorSourceConfig>,
    cache: &mut HashMap<SensorSource, ResolvedSensor>,
    all_sensors: &[SensorInfo],
) -> (u8, bool) {
    let Some(cfg) = source else {
        return (0, false);
    };
    let src = cfg.to_sensor_source();
    match resolve_and_read(&src, cache, all_sensors) {
        Some(v) => (v.clamp(0.0, 99.0) as u8, true),
        None => (0, false),
    }
}

fn resolve_and_read(
    source: &SensorSource,
    cache: &mut HashMap<SensorSource, ResolvedSensor>,
    all_sensors: &[SensorInfo],
) -> Option<f32> {
    let resolved = if let Some(r) = cache.get(source) {
        r.clone()
    } else {
        let divider = all_sensors
            .iter()
            .find(|s| s.source == *source)
            .map_or(1, |s| s.divider);
        let r = resolve_sensor(source, divider)?;
        cache.insert(source.clone(), r.clone());
        r
    };
    match read_sensor_value(&resolved) {
        Ok(v) => Some(v),
        Err(_) => {
            cache.remove(source);
            None
        }
    }
}

fn resolve_pump_rpm(
    speed: &FanSpeed,
    variant: WirelessFanType,
    curves: &HashMap<String, FanCurve>,
    sensor_cache: &mut HashMap<SensorSource, ResolvedSensor>,
    all_sensors: &[SensorInfo],
) -> u32 {
    let Some((min_rpm, max_rpm)) = variant.pump_rpm_range() else {
        return 0;
    };
    let pct = match speed {
        FanSpeed::Constant(b) => (*b as f32 / 255.0) * 100.0,
        FanSpeed::Curve(name) => match curves.get(name) {
            Some(curve) => {
                let source = curve.effective_source();
                match resolve_and_read(&source, sensor_cache, all_sensors) {
                    Some(temp) => interpolate_curve(&curve.curve, temp).clamp(0.0, 100.0),
                    None => 50.0,
                }
            }
            None => 50.0,
        },
    };
    let span = (max_rpm - min_rpm) as f32;
    (min_rpm as f32 + (pct / 100.0) * span).round() as u32
}

fn write_argb(dst: &mut [u8], rgba: [u8; 4]) {
    dst[0] = rgba[3];
    dst[1] = rgba[0];
    dst[2] = rgba[1];
    dst[3] = rgba[2];
}

fn interpolate_curve(curve: &[(f32, f32)], temp: f32) -> f32 {
    if curve.is_empty() {
        return 50.0;
    }
    if curve.len() == 1 {
        return curve[0].1;
    }
    let mut sorted = curve.to_vec();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    if temp <= sorted[0].0 {
        return sorted[0].1;
    }
    if temp >= sorted[sorted.len() - 1].0 {
        return sorted[sorted.len() - 1].1;
    }
    for i in 0..sorted.len() - 1 {
        let (t1, s1) = sorted[i];
        let (t2, s2) = sorted[i + 1];
        if temp >= t1 && temp <= t2 {
            let ratio = (temp - t1) / (t2 - t1);
            return s1 + ratio * (s2 - s1);
        }
    }
    50.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use lianli_devices::wireless::WirelessFanType;

    fn fresh_cache() -> (HashMap<SensorSource, ResolvedSensor>, Vec<SensorInfo>) {
        (HashMap::new(), Vec::new())
    }

    #[test]
    fn resolve_pump_rpm_constant_maps_linearly() {
        let (mut cache, sensors) = fresh_cache();
        let curves = HashMap::new();
        let rpm = resolve_pump_rpm(
            &FanSpeed::Constant(0),
            WirelessFanType::WaterBlock,
            &curves,
            &mut cache,
            &sensors,
        );
        assert_eq!(rpm, 1600);
        let rpm = resolve_pump_rpm(
            &FanSpeed::Constant(255),
            WirelessFanType::WaterBlock,
            &curves,
            &mut cache,
            &sensors,
        );
        assert_eq!(rpm, 2500);
        let rpm = resolve_pump_rpm(
            &FanSpeed::Constant(128),
            WirelessFanType::WaterBlock,
            &curves,
            &mut cache,
            &sensors,
        );
        assert!(rpm >= 2048 && rpm <= 2054, "got {rpm}");
    }

    #[test]
    fn resolve_pump_rpm_square_uses_wider_range() {
        let (mut cache, sensors) = fresh_cache();
        let curves = HashMap::new();
        let rpm = resolve_pump_rpm(
            &FanSpeed::Constant(255),
            WirelessFanType::WaterBlock2,
            &curves,
            &mut cache,
            &sensors,
        );
        assert_eq!(rpm, 3200);
    }

    #[test]
    fn resolve_pump_rpm_non_aio_returns_zero() {
        let (mut cache, sensors) = fresh_cache();
        let curves = HashMap::new();
        let rpm = resolve_pump_rpm(
            &FanSpeed::Constant(128),
            WirelessFanType::Slv3Led,
            &curves,
            &mut cache,
            &sensors,
        );
        assert_eq!(rpm, 0);
    }

    #[test]
    fn read_optional_none_yields_disabled() {
        let (mut cache, sensors) = fresh_cache();
        let (val, ok) = read_optional(&None, &mut cache, &sensors);
        assert_eq!(val, 0);
        assert!(!ok);
    }
}
