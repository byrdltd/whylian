//! Heuristics for pairing wireless AIOs with external radiator fan clusters.
//!
//! HydroShift II LCD-S/C (WaterBlock/WaterBlock2) report `fan_count == 0` on the
//! AIO wireless record; radiator fans appear as a separate TLV2 device on the
//! same dongle. L-Connect 3 routes fan PWM to that TLV2 entity, not the AIO MAC.

use super::discovery::DiscoveredDevice;
use super::fan_type::WirelessFanType;

/// TLV2 fan controller that may host an AIO's radiator fans.
pub fn is_tlv2_radiator_candidate(dev: &DiscoveredDevice) -> bool {
    dev.fan_count > 0
        && matches!(
            dev.fan_type,
            WirelessFanType::Tlv2Led | WirelessFanType::Tlv2Lcd
        )
}

/// Find the radiator fan cluster paired with a wireless AIO.
///
/// When the AIO already reports integrated fans (`fan_count > 0`), returns `None`.
/// Otherwise looks for a bound TLV2 device on the same master MAC.
pub fn find_aio_radiator_cluster<'a>(
    aio: &DiscoveredDevice,
    devices: &'a [DiscoveredDevice],
) -> Option<&'a DiscoveredDevice> {
    if !aio.fan_type.is_aio() || aio.fan_count > 0 {
        return None;
    }

    devices
        .iter()
        .filter(|d| d.master_mac == aio.master_mac && is_tlv2_radiator_candidate(d))
        .max_by_key(|d| d.fan_count)
}

/// Effective radiator fan count for GUI / control loops.
pub fn effective_aio_fan_count(aio: &DiscoveredDevice, devices: &[DiscoveredDevice]) -> u8 {
    if aio.fan_count > 0 {
        return aio.fan_count;
    }
    find_aio_radiator_cluster(aio, devices)
        .map(|r| r.fan_count)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mac(b: u8) -> [u8; 6] {
        [0x4d, 0xee, 0xca, 0xe5, 0x66, b]
    }

    fn master() -> [u8; 6] {
        [0x3f, 0xdb, 0xd8, 0xe5, 0x66, 0xe4]
    }

    fn sample_aio() -> DiscoveredDevice {
        DiscoveredDevice {
            mac: mac(0xe1),
            master_mac: master(),
            channel: 8,
            rx_type: 3,
            device_type: 11,
            fan_count: 0,
            fan_types: [0; 4],
            fan_rpms: [0, 0, 0, 2458],
            current_pwm: [0; 4],
            cmd_seq: 0,
            fan_type: WirelessFanType::WaterBlock2,
            list_index: 1,
            coolant_temp_c: Some(32),
            effect_index: [0; 4],
        }
    }

    fn sample_tlv2_radiator() -> DiscoveredDevice {
        DiscoveredDevice {
            mac: mac(0xe0),
            master_mac: master(),
            channel: 8,
            rx_type: 2,
            device_type: 0,
            fan_count: 3,
            fan_types: [28, 28, 28, 0],
            fan_rpms: [1270, 1257, 1260, 0],
            current_pwm: [127, 127, 127, 0],
            cmd_seq: 0,
            fan_type: WirelessFanType::Tlv2Led,
            list_index: 0,
            coolant_temp_c: None,
            effect_index: [0; 4],
        }
    }

    #[test]
    fn pairs_waterblock2_with_tlv2_on_same_master() {
        let aio = sample_aio();
        let radiator = sample_tlv2_radiator();
        let devices = [radiator.clone(), aio.clone()];
        let found = find_aio_radiator_cluster(&aio, &devices).unwrap();
        assert_eq!(found.mac, radiator.mac);
        assert_eq!(effective_aio_fan_count(&aio, &devices), 3);
    }

    #[test]
    fn integrated_aio_fans_skip_pairing() {
        let mut aio = sample_aio();
        aio.fan_count = 2;
        let radiator = sample_tlv2_radiator();
        let devices = [radiator, aio.clone()];
        assert!(find_aio_radiator_cluster(&aio, &devices).is_none());
        assert_eq!(effective_aio_fan_count(&aio, &devices), 2);
    }

    #[test]
    fn ignores_tlv2_on_different_master() {
        let aio = sample_aio();
        let mut radiator = sample_tlv2_radiator();
        radiator.master_mac = [0xaa; 6];
        let devices = [radiator, aio.clone()];
        assert!(find_aio_radiator_cluster(&aio, &devices).is_none());
        assert_eq!(effective_aio_fan_count(&aio, &devices), 0);
    }
}
