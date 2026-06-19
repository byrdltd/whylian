//! HydroShift LCD / Galahad2 LCD / Vision AIO driver (pump + fan + LCD + temp).
//!
//! HydroShift LCD:   VID=0x0416, PID=0x7398/0x7399/0x739A
//! Galahad2 LCD:     VID=0x0416, PID=0x7391
//! Galahad2 Vision:  VID=0x0416, PID=0x7395
//!
//! All use an identical protocol with three HID report types:
//!   A-command (64B, Report ID 1): pump/fan PWM, handshake, firmware
//!   B-command (1024B out, Report ID 2): LCD control, JPEG frames
//!   C-command (512B, Report ID 3): LCD frames (firmware >= 1.2)
//!
//! LCD: 480x480 pixels, 24fps. Pump/fan PWM: 0-100%.
//! Coolant temperature sensor available.

mod controller;
mod protocol;
mod rgb;

pub use controller::HydroShiftLcdController;
pub use rgb::AioLcdRgbController;

/// AIO LCD device variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AioLcdVariant {
    /// HydroShift LCD (0x7398)
    HydroShiftLcd,
    /// HydroShift LCD RGB (0x7399)
    HydroShiftLcdRgb,
    /// HydroShift LCD TL (0x739A)
    HydroShiftLcdTl,
    /// Galahad2 LCD (0x7391)
    Galahad2Lcd,
    /// Galahad2 Vision (0x7395)
    Galahad2Vision,
}

impl AioLcdVariant {
    pub fn from_pid(pid: u16) -> Option<Self> {
        match pid {
            0x7398 => Some(Self::HydroShiftLcd),
            0x7399 => Some(Self::HydroShiftLcdRgb),
            0x739A => Some(Self::HydroShiftLcdTl),
            0x7391 => Some(Self::Galahad2Lcd),
            0x7395 => Some(Self::Galahad2Vision),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::HydroShiftLcd => "HydroShift LCD",
            Self::HydroShiftLcdRgb => "HydroShift LCD RGB",
            Self::HydroShiftLcdTl => "HydroShift LCD TL",
            Self::Galahad2Lcd => "Galahad II LCD",
            Self::Galahad2Vision => "Galahad II Vision",
        }
    }

    /// Whether this variant has pump head RGB (SetPumpLighting 0x83).
    /// Galahad2 LCD + Vision have pump RGB; HydroShift variants do not.
    pub fn has_pump_rgb(&self) -> bool {
        matches!(self, Self::Galahad2Lcd | Self::Galahad2Vision)
    }

    pub fn c_command_min_firmware(&self) -> (u32, u32) {
        match self {
            Self::HydroShiftLcd | Self::HydroShiftLcdRgb | Self::HydroShiftLcdTl => (1, 2),
            Self::Galahad2Lcd | Self::Galahad2Vision => (1, 5),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LcdControlMode {
    LocalUi = 0,
    Application = 1,
    LocalH264 = 2,
    LocalAvi = 3,
    LcdSetting = 4,
    LcdTest = 5,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ScreenRotation {
    Rotate0 = 0,
    Rotate90 = 1,
    Rotate180 = 2,
    Rotate270 = 3,
}

impl ScreenRotation {
    pub fn from_degrees(degrees: u16) -> Self {
        match degrees {
            90 => Self::Rotate90,
            180 => Self::Rotate180,
            270 => Self::Rotate270,
            _ => Self::Rotate0,
        }
    }
}

/// Handshake response: RPM + temperature.
#[derive(Debug, Clone)]
pub struct AioHandshake {
    pub fan_rpm: u16,
    pub pump_rpm: u16,
    pub temp_valid: bool,
    pub coolant_temp: f32,
}
