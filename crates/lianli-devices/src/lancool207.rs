//! Lancool 207 Digital case LCD panel driver.
//!
//! VID=0x1CBE, PID=0xA065 — 1472x720 LCD via WinUSB.
//!
//! Uses the generic WinUSB LCD protocol (DES-CBC encrypted headers).

use crate::winusb_lcd::WinUsbLcdDevice;
use anyhow::Result;
use lianli_shared::screen::ScreenInfo;
use rusb::{Device, GlobalContext};

pub const VID: u16 = 0x1CBE;
pub const PID: u16 = 0xA065;

/// Open a Lancool 207 Digital device.
pub fn open(device: Device<GlobalContext>) -> Result<WinUsbLcdDevice> {
    WinUsbLcdDevice::new(device, ScreenInfo::LANCOOL_207, "Lancool 207 Digital")
}
