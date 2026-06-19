//! TURZX desktop-mode USB display protocol.
//!
//! Decoded from the Windows `LIANLI_display_driver.dll` — see `target/TURZX.md`
//! for the full protocol reference and the packet-builder provenance.

mod device;
mod edid;
mod framing;
mod vendor_caps;

pub use device::{DeviceIdentity, TurzxDisplay};
pub use edid::{build_edid, patch_edid_serial};
pub use framing::{
    build_config_packet, build_power_off, fragment_stream_a, pack_fragment, pack_frame, tlv,
};
pub use vendor_caps::{parse_vendor_desc, pick_format, pick_mode, Mode, VendorCaps};

pub const VID: u16 = 0x1A86;
pub const PID_RANGE: std::ops::RangeInclusive<u16> = 0xAD10..=0xAD3F;

pub const MAGIC: u8 = 0xAF;
pub const CTRL_OP: u8 = 0x20;

pub const FMT_MJPEG: u16 = 0x0111;
pub const FMT_H264: u16 = 0x0112;

pub const STREAM_A_FRAG: u8 = 0x6C;
pub const STREAM_A_FINAL: u8 = 0x6D;
pub const STREAM_B_FRAG: u8 = 0x68;
pub const STREAM_B_FINAL: u8 = 0x69;
pub const STREAM_C: u8 = 0x6B;
pub const COMMIT: u8 = 0x66;

/// True for TURZX desktop-mode panels (CH340, VID=0x1A86).
///
/// Most PIDs sit in `0xAD10..=0xAD3F`; Lancool 207 (`0xACD1`) and Universal
/// Screen 8.8" (`0xACE1`) desktop firmware uses PIDs outside that range.
pub fn is_turzx(vid: u16, pid: u16) -> bool {
    vid == VID && (PID_RANGE.contains(&pid) || pid == 0xACD1 || pid == 0xACE1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turzx_matches_standard_range_and_outliers() {
        assert!(is_turzx(VID, 0xAD20));
        assert!(is_turzx(VID, 0xAD3F));
        assert!(is_turzx(VID, 0xACD1));
        assert!(is_turzx(VID, 0xACE1));
        assert!(!is_turzx(VID, 0xE304));
        assert!(!is_turzx(0x1CBE, 0xA034));
    }
}
