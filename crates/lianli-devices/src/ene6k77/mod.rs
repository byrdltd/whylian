//! ENE 6K77 wired fan controller driver (SL/AL series).
//!
//! VID=0x0CF2, PID=0xA100-0xA106
//!
//! Protocol uses HID Feature Reports with Report ID 0xE0.
//! Each controller has 4 fan groups with independent PWM duty control.
//! RPM is read via feature report 0x50 sub-command 0x00.

mod controller;
mod group_rgb;
mod model;

pub use controller::Ene6k77Controller;
pub use group_rgb::Ene6k77GroupDevice;
pub use model::{Ene6k77Firmware, Ene6k77Model};

use std::time::Duration;

const REPORT_ID: u8 = 0xE0;
const CMD_DELAY: Duration = Duration::from_millis(20);
