//! Galahad II LCD / Vision AIO driver.
//!
//! VID=0x0416, PID=0x7391 (LCD) / 0x7395 (Vision)
//!
//! Uses an identical protocol to HydroShift LCD — same A/B/C command structure,
//! same pump/fan/LCD/temp commands. This module re-exports the shared driver.

pub use crate::hydroshift_lcd::{
    AioHandshake, AioLcdVariant, HydroShiftLcdController as Galahad2LcdController, LcdControlMode,
    ScreenRotation,
};
