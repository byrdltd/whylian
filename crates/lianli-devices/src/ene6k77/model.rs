/// ENE 6K77 model variant, determined by PID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ene6k77Model {
    /// 0xA100 — SL Fan (4 groups, 4 fans max each)
    SlFan,
    /// 0xA101 — AL Fan (4 groups, dual-ring LEDs)
    AlFan,
    /// 0xA102 — SL Infinity (4 groups)
    SlInfinity,
    /// 0xA103 — SL V2 Fan (4 groups, 6 fans max each)
    SlV2Fan,
    /// 0xA104 — AL V2 Fan (4 groups, 6 fans max each)
    AlV2Fan,
    /// 0xA105 — SL V2A Fan
    SlV2aFan,
    /// 0xA106 — SL Redragon
    SlRedragon,
}

impl Ene6k77Model {
    pub fn from_pid(pid: u16) -> Option<Self> {
        match pid {
            0xA100 => Some(Self::SlFan),
            0xA101 => Some(Self::AlFan),
            0xA102 => Some(Self::SlInfinity),
            0xA103 => Some(Self::SlV2Fan),
            0xA104 => Some(Self::AlV2Fan),
            0xA105 => Some(Self::SlV2aFan),
            0xA106 => Some(Self::SlRedragon),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::SlFan => "SL Fan",
            Self::AlFan => "AL Fan",
            Self::SlInfinity => "SL Infinity",
            Self::SlV2Fan => "SL V2 Fan",
            Self::AlV2Fan => "AL V2 Fan",
            Self::SlV2aFan => "SL V2A Fan",
            Self::SlRedragon => "SL Redragon",
        }
    }

    /// Whether this is a V2 model (supports 6 fans/group, 9-byte RPM response).
    pub fn is_v2(&self) -> bool {
        matches!(self, Self::SlV2Fan | Self::AlV2Fan | Self::SlV2aFan)
    }

    /// Whether this model uses doubled port encoding (0x10|(group*2) for effects).
    pub fn uses_double_port(&self) -> bool {
        matches!(self, Self::AlFan | Self::AlV2Fan | Self::SlInfinity)
    }

    /// Max fans per group.
    pub fn max_fans_per_group(&self) -> u8 {
        if self.is_v2() {
            6
        } else {
            4
        }
    }

    pub fn palette_size(&self) -> usize {
        match self {
            Self::AlV2Fan => 6,
            _ => 4,
        }
    }

    pub fn single_ring_leds_per_fan(&self) -> usize {
        match self {
            Self::SlFan | Self::SlV2Fan | Self::SlV2aFan | Self::SlRedragon => 16,
            _ => 0,
        }
    }

    pub fn inner_leds_per_fan(&self) -> usize {
        if self.uses_double_port() {
            8
        } else {
            0
        }
    }

    pub fn outer_leds_per_fan(&self) -> usize {
        if self.uses_double_port() {
            12
        } else {
            0
        }
    }

    /// Frame commit value for `[REPORT_ID, 0x60, hi, lo]`. SLV2/SLV2A use 4;
    /// every other variant uses 1.
    pub fn frame_commit_value(&self) -> u16 {
        match self {
            Self::SlV2Fan | Self::SlV2aFan => 4,
            _ => 1,
        }
    }
}

/// Firmware version info read from the device.
#[derive(Debug, Clone)]
pub struct Ene6k77Firmware {
    pub customer_id: u8,
    pub project_id: u8,
    pub major_id: u8,
    pub minor_id: u8,
    pub fine_tune: u8,
}

impl std::fmt::Display for Ene6k77Firmware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version = if self.fine_tune < 8 {
            "1.0".to_string()
        } else {
            let v = ((self.fine_tune >> 4) * 10 + (self.fine_tune & 0x0F) + 2) as f32 / 10.0;
            format!("{v:.1}")
        };
        write!(
            f,
            "v{} (cust={:#04x} proj={:#04x} major={:#04x} minor={:#04x})",
            version, self.customer_id, self.project_id, self.major_id, self.minor_id
        )
    }
}
