//! Universal Screen 8.8" display driver.
//!
//! VID=0x1CBE, PID=0xA088 — 1920x480 LCD via WinUSB.
//!
//! Uses the generic WinUSB LCD protocol (DES-CBC encrypted headers).

use crate::winusb_lcd::WinUsbLcdDevice;
use anyhow::Result;
use lianli_shared::screen::ScreenInfo;
use rusb::{Device, GlobalContext};

pub const VID: u16 = 0x1CBE;
pub const PID: u16 = 0xA088;

/// Open a Universal Screen 8.8" device.
pub fn open(device: Device<GlobalContext>) -> Result<WinUsbLcdDevice> {
    WinUsbLcdDevice::new(
        device,
        ScreenInfo::UNIVERSAL_SCREEN,
        "Universal Screen 8.8\"",
    )
}
