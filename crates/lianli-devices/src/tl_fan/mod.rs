//! TL Fan controller driver.
//!
//! VID=0x0416, PID=0x7372
//!
//! Protocol uses HID Output Reports with Report ID 0x01.
//! 64-byte packets with a 6-byte header: [reportId, cmd, reserved, pktNumHi, pktNumLo, dataLen].
//! Each command expects a synchronous response (read after write).
//!
//! The controller supports 4 ports, each with multiple fans.
//! Fan speed is set per-fan via command 0xAA.
//! RPM values are only available from the handshake response (0xA1).

mod controller;
mod port_rgb;

pub use controller::TlFanController;
pub use port_rgb::TlFanPortDevice;

/// Number of LEDs per TL fan.
const LEDS_PER_FAN: u16 = 20;

/// Information about a single detected fan.
#[derive(Debug, Clone)]
pub struct TlFanInfo {
    pub port: u8,
    pub fan_index: u8,
    pub rpm: u16,
    pub is_detected: bool,
}

/// TL Fan handshake result containing discovered fans per port.
#[derive(Debug, Clone)]
pub struct TlFanHandshake {
    /// Fans detected on each port. Index = port number (0-3).
    pub port_fan_counts: [u8; 4],
    /// All detected fans with their RPM values.
    pub fans: Vec<TlFanInfo>,
}
