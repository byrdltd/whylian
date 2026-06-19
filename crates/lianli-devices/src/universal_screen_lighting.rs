//! Universal Screen 8.8" LED Ring (VID=0x0416, PID=0x8050).

use crate::winusb_led::WinUsbLedDevice;
use anyhow::Result;
use rusb::{Device, GlobalContext};

pub const VID: u16 = 0x0416;
pub const PID: u16 = 0x8050;
pub const LED_COUNT: u16 = 60;

pub fn open(device: Device<GlobalContext>) -> Result<WinUsbLedDevice> {
    WinUsbLedDevice::new(device, LED_COUNT, "Universal Screen 8.8\" LED Ring")
}
