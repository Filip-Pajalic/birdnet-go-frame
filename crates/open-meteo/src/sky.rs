//! What the sky is doing, from a WMO weather code.

/// The kinds of weather a page names, grouped from WMO code table 4677 as Open-Meteo sends it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sky {
    /// Code 0.
    Clear,
    /// Codes 1 and 2.
    PartlyCloudy,
    /// Code 3.
    Overcast,
    /// Codes 45 and 48.
    Fog,
    /// Codes 51 to 57.
    Drizzle,
    /// Codes 61 to 67 and 80 to 82.
    Rain,
    /// Codes 71 to 77, 85 and 86.
    Snow,
    /// Codes 95 to 99.
    Thunder,
    /// Any other code.
    Unknown,
}

impl Sky {
    /// Returns the sky for WMO code `code`.
    #[must_use]
    pub fn from_code(code: u32) -> Self {
        match code {
            0 => Self::Clear,
            1 | 2 => Self::PartlyCloudy,
            3 => Self::Overcast,
            45 | 48 => Self::Fog,
            51..=57 => Self::Drizzle,
            61..=67 | 80..=82 => Self::Rain,
            71..=77 | 85 | 86 => Self::Snow,
            95..=99 => Self::Thunder,
            _ => Self::Unknown,
        }
    }

    /// Returns the translation key that names this sky, e.g. `sky_rain`.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Clear => "sky_clear",
            Self::PartlyCloudy => "sky_partly_cloudy",
            Self::Overcast => "sky_overcast",
            Self::Fog => "sky_fog",
            Self::Drizzle => "sky_drizzle",
            Self::Rain => "sky_rain",
            Self::Snow => "sky_snow",
            Self::Thunder => "sky_thunder",
            Self::Unknown => "sky_unknown",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn groups_codes() {
        assert_eq!(Sky::from_code(0), Sky::Clear);
        assert_eq!(Sky::from_code(81), Sky::Rain);
        assert_eq!(Sky::from_code(86), Sky::Snow);
        assert_eq!(Sky::from_code(4), Sky::Unknown);
        assert_eq!(Sky::Rain.key(), "sky_rain");
    }
}
