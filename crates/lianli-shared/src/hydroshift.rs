//! HydroShift II AdvanceMode defaults — LCD auto-config and coolant fan curves.

use crate::config::{AppConfig, LcdConfig};
use crate::fan::{FanCurve, FanSpeed};
use crate::media::{MediaType, SensorSourceConfig};
use crate::sensors::SensorSource;

/// Fan curve name created for AIO coolant temperature control.
pub const COOLANT_CURVE_NAME: &str = "Coolant";

/// L-Connect AdvanceMode wireless theme index for HydroShift II AIO heads.
pub const ADVANCE_MODE_THEME_INDEX: u8 = 4;

/// Default coolant-based fan curve (°C → duty %).
pub fn default_coolant_fan_curve(aio_device_id: &str) -> FanCurve {
    FanCurve {
        name: COOLANT_CURVE_NAME.to_string(),
        temp_source: Some(SensorSource::WirelessCoolant {
            device_id: aio_device_id.to_string(),
        }),
        temp_command: String::new(),
        curve: vec![
            (25.0, 25.0),
            (35.0, 40.0),
            (45.0, 60.0),
            (55.0, 80.0),
            (65.0, 100.0),
        ],
    }
}

/// Default LCD entry for a HydroShift II USB display (WinUSB 480×480).
pub fn default_hydroshift2_lcd(serial: &str, aio_device_id: Option<&str>) -> LcdConfig {
    let coolant_source = aio_device_id
        .map(|id| SensorSourceConfig::WirelessCoolant {
            device_id: id.to_string(),
        })
        .unwrap_or(SensorSourceConfig::CpuUsage);
    LcdConfig {
        index: None,
        serial: Some(serial.to_string()),
        media_type: MediaType::Custom,
        path: None,
        fps: None,
        update_interval_ms: Some(1000),
        rgb: None,
        orientation: 0.0,
        sensor: None,
        sensor_source_1: SensorSourceConfig::CpuUsage,
        sensor_source_2: coolant_source,
        doublegauge: None,
        template_id: Some("cooler".to_string()),
        smooth_edges: None,
        custom_h264: Some(true),
        aio_512_frame: Some(true),
    }
}

/// True when all four fan slots still use the factory constant ~50% PWM.
pub fn fan_speeds_are_factory_default(speeds: &[FanSpeed; 4]) -> bool {
    speeds.iter().all(|s| matches!(s, FanSpeed::Constant(128)))
}

/// True when pump speed is still the factory constant ~50%.
pub fn pump_speed_is_factory_default(pump: &FanSpeed) -> bool {
    matches!(pump, FanSpeed::Constant(128))
}

impl AppConfig {
    /// Ensure a coolant fan curve exists for the given wireless AIO.
    pub fn ensure_coolant_fan_curve(&mut self, aio_device_id: &str) -> bool {
        if self.fan_curves.iter().any(|c| c.name == COOLANT_CURVE_NAME) {
            return false;
        }
        self.fan_curves
            .push(default_coolant_fan_curve(aio_device_id));
        true
    }

    /// Point AIO fan/pump slots at the coolant curve when still on factory constants.
    pub fn apply_coolant_curve_to_aio(&mut self, aio_device_id: &str) -> bool {
        let Some(aio) = self.aio.get_mut(aio_device_id) else {
            return false;
        };
        let has_curve = self.fan_curves.iter().any(|c| c.name == COOLANT_CURVE_NAME);
        if !has_curve {
            return false;
        }

        let mut changed = false;
        if fan_speeds_are_factory_default(&aio.fan_speeds) {
            aio.fan_speeds = [
                FanSpeed::Curve(COOLANT_CURVE_NAME.to_string()),
                FanSpeed::Curve(COOLANT_CURVE_NAME.to_string()),
                FanSpeed::Curve(COOLANT_CURVE_NAME.to_string()),
                FanSpeed::Curve(COOLANT_CURVE_NAME.to_string()),
            ];
            changed = true;
        }
        if pump_speed_is_factory_default(&aio.pump_target_rpm) {
            aio.pump_target_rpm = FanSpeed::Curve(COOLANT_CURVE_NAME.to_string());
            changed = true;
        }
        changed
    }

    /// Auto-add LCD config for discovered HydroShift II USB displays.
    pub fn ensure_hydroshift_lcd(&mut self, serial: &str, aio_device_id: Option<&str>) -> bool {
        if self
            .lcds
            .iter()
            .any(|l| l.serial.as_deref() == Some(serial))
        {
            return false;
        }
        self.lcds
            .push(default_hydroshift2_lcd(serial, aio_device_id));
        true
    }

    /// Apply AdvanceMode wireless theme when the AIO entry is still at factory theme 0.
    pub fn apply_advance_mode_theme(
        &mut self,
        aio_device_id: &str,
        is_hydroshift_aio: bool,
    ) -> bool {
        if !is_hydroshift_aio {
            return false;
        }
        let Some(aio) = self.aio.get_mut(aio_device_id) else {
            return false;
        };
        if aio.theme_index != 0 {
            return false;
        }
        aio.theme_index = ADVANCE_MODE_THEME_INDEX;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coolant_curve_uses_wireless_source() {
        let curve = default_coolant_fan_curve("wireless:aa:bb:cc:dd:ee:ff");
        assert_eq!(curve.name, COOLANT_CURVE_NAME);
        assert!(matches!(
            curve.temp_source,
            Some(SensorSource::WirelessCoolant { .. })
        ));
        assert!(curve.curve.len() >= 3);
    }

    #[test]
    fn factory_default_detection() {
        assert!(fan_speeds_are_factory_default(&[
            FanSpeed::Constant(128),
            FanSpeed::Constant(128),
            FanSpeed::Constant(128),
            FanSpeed::Constant(128),
        ]));
        assert!(!fan_speeds_are_factory_default(&[
            FanSpeed::Curve("Coolant".into()),
            FanSpeed::Constant(128),
            FanSpeed::Constant(128),
            FanSpeed::Constant(128),
        ]));
    }

    #[test]
    fn ensure_lcd_and_curve_are_idempotent() {
        let mut cfg = AppConfig::default();
        let aio_id = "wireless:4d:ee:ca:e5:66:e1";
        assert!(cfg.ensure_coolant_fan_curve(aio_id));
        assert!(!cfg.ensure_coolant_fan_curve(aio_id));
        assert!(cfg.ensure_hydroshift_lcd("serial123", None));
        assert!(!cfg.ensure_hydroshift_lcd("serial123", None));
    }
}
