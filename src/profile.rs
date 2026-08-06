//! Parser for the AMD Adrenalin tuning XML format
//! (as exported to %LOCALAPPDATA%\AMD\Radeonsoftware\uv.xml).
//!
//! The file describes GPU tuning as a flat list of FEATUREs, each holding one
//! or more STATEs. The FEATURE ID meanings are Adrenalin-internal and are
//! documented in `feature.rs` (reverse-engineered, must be validated against
//! the Adrenalin UI before being trusted for apply).

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename = "SYSTEM")]
pub struct System {
    /// Newer Adrenalin exports include a <CPU> section. We never read or touch
    /// CPU tuning; consume and discard it so real exports still parse.
    #[serde(rename = "CPU", default)]
    _cpu: Option<serde::de::IgnoredAny>,
    #[serde(rename = "GPU")]
    pub gpu: Gpu,
}

#[derive(Debug, Deserialize)]
pub struct Gpu {
    /// PCI device id, e.g. "73BF" (Navi 21). Hex string, no 0x prefix.
    #[serde(rename = "@DevID")]
    pub dev_id: String,
    #[serde(rename = "@RevID")]
    pub rev_id: String,
    #[serde(rename = "PPW")]
    pub ppw: Option<Ppw>,
    #[serde(rename = "FEATURE", default)]
    pub features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub struct Ppw {
    #[serde(rename = "@Value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    #[serde(rename = "@ID")]
    pub id: u32,
    /// Raw Enabled attribute. Inconsistent across the file: sometimes numeric
    /// ("0", "3"), sometimes boolean ("True"/"False"). Kept as-is; use
    /// `enabled_bool()` for a best-effort interpretation.
    #[serde(rename = "@Enabled")]
    pub enabled: String,
    #[serde(rename = "STATES")]
    pub states: States,
}

#[derive(Debug, Deserialize)]
pub struct States {
    #[serde(rename = "STATE", default)]
    pub state: Vec<State>,
}

#[derive(Debug, Deserialize)]
pub struct State {
    #[serde(rename = "@ID")]
    pub id: u32,
    #[serde(rename = "@Enabled")]
    pub enabled: String,
    /// Signed because some values use -1 as "unset".
    #[serde(rename = "@Value")]
    pub value: i64,
}

impl Feature {
    /// Best-effort interpretation of the Enabled attribute as a boolean.
    /// Treats "True"/nonzero as enabled, "False"/"0" as disabled.
    pub fn enabled_bool(&self) -> bool {
        parse_enabled(&self.enabled)
    }
}

impl State {
    pub fn enabled_bool(&self) -> bool {
        parse_enabled(&self.enabled)
    }
}

fn parse_enabled(s: &str) -> bool {
    match s.trim() {
        "True" | "true" => true,
        "False" | "false" => false,
        other => other.parse::<i64>().map(|n| n != 0).unwrap_or(false),
    }
}

impl System {
    /// Look up a STATE value by (FEATURE id, STATE id).
    pub fn state_value(&self, feature_id: u32, state_id: u32) -> Option<i64> {
        self.gpu
            .features
            .iter()
            .find(|f| f.id == feature_id)?
            .states
            .state
            .iter()
            .find(|s| s.id == state_id)
            .map(|s| s.value)
    }

    /// The FEATURE-level Enabled flag for a feature id, if present.
    pub fn feature_enabled(&self, feature_id: u32) -> Option<bool> {
        self.gpu
            .features
            .iter()
            .find(|f| f.id == feature_id)
            .map(|f| f.enabled_bool())
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ProfileError> {
        let text = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ProfileError::Io(path.as_ref().display().to_string(), e))?;
        Self::from_str(&text)
    }

    pub fn from_str(xml: &str) -> Result<Self, ProfileError> {
        quick_xml::de::from_str(xml).map_err(ProfileError::Parse)
    }
}

#[derive(Debug)]
pub enum ProfileError {
    Io(String, std::io::Error),
    Parse(quick_xml::DeError),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileError::Io(p, e) => write!(f, "reading {p}: {e}"),
            ProfileError::Parse(e) => write!(f, "parsing XML: {e}"),
        }
    }
}

impl std::error::Error for ProfileError {}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../sample_uv.xml");

    #[test]
    fn parses_sample() {
        let sys = System::from_str(SAMPLE).expect("parse");
        assert_eq!(sys.gpu.dev_id, "73BF");
        assert_eq!(sys.gpu.rev_id, "C0");
        // Fan curve feature (ID 22) has 11 states in the sample.
        let fan = sys.gpu.features.iter().find(|f| f.id == 22).expect("fan feature");
        assert_eq!(fan.states.state.len(), 11);
    }

    #[test]
    fn ignores_cpu_section() {
        // Real Adrenalin exports (e.g. RX 9070 XT) include a <CPU> block before
        // <GPU>; it must not break parsing and must not be read.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<SYSTEM>
    <CPU>
        <FEATURE ID="100" Enabled="1"><STATES><STATE ID="0" Enabled="False" Value="0"/></STATES></FEATURE>
        <FEATURE ID="0" Enabled="True"><STATES><STATE ID="0" Enabled="False" Value="0"/></STATES></FEATURE>
    </CPU>
    <GPU DevID="7550" RevID="C0">
        <PPW Value="1"/>
        <FEATURE ID="12" Enabled="False"><STATES><STATE ID="0" Enabled="False" Value="-80"/></STATES></FEATURE>
    </GPU>
</SYSTEM>"#;
        let sys = System::from_str(xml).expect("parse with CPU section");
        assert_eq!(sys.gpu.dev_id, "7550");
        assert_eq!(sys.state_value(12, 0), Some(-80));
    }

    #[test]
    fn parse_enabled_variants() {
        assert!(parse_enabled("True"));
        assert!(!parse_enabled("False"));
        assert!(!parse_enabled("0"));
        assert!(parse_enabled("3"));
    }
}
