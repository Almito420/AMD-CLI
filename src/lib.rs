pub mod adlx;
pub mod feature;
pub mod profile;

use adlx::TuningSet;
use profile::System;

/// Extract a full TuningSet from a parsed Adrenalin profile (the XML defines
/// everything; CPU is never read). A value of 0 in the clock/voltage/mem fields
/// means "not manually set" and is left untouched (setting 0 would clamp up to
/// the hardware range minimum). Power limit keeps 0 (a valid default). The fan
/// curve is always included when present -- the XML applies everything.
pub fn profile_to_tuning_set(sys: &System) -> TuningSet {
    let v = |coord: (u32, u32)| sys.state_value(coord.0, coord.1).map(|x| x as i32);
    let nonzero = |coord: (u32, u32)| v(coord).filter(|&x| x != 0);
    // Memory tuning is only present when FEATURE 5 is Enabled. Otherwise the
    // export just carries the stock frequency (e.g. 2000) as an informational
    // default, NOT a value to apply -- applying it would wrongly switch memory
    // into manual tuning. (Confirmed across stock/tuned exports.)
    let mem_clock = if sys.feature_enabled(feature::MEM_CLOCK.0) == Some(true) {
        nonzero(feature::MEM_CLOCK)
    } else {
        None
    };
    let fan = fan_curve(sys);
    TuningSet {
        min_clock: nonzero(feature::MIN_CLOCK),
        max_clock: nonzero(feature::MAX_CLOCK),
        voltage: nonzero(feature::VOLTAGE),
        mem_clock,
        power_limit: v(feature::POWER_LIMIT),
        fan_curve: (!fan.is_empty()).then_some(fan),
    }
}

/// Extract the fan curve from FEATURE 22 as (temperature C, speed %) points.
/// Even STATE ids are temperature, odd ids are speed; a temperature of 0
/// terminates the curve (the export pads with a trailing zero).
pub fn fan_curve(sys: &System) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let mut k = 0u32;
    loop {
        match (
            sys.state_value(feature::FAN.0, 2 * k),
            sys.state_value(feature::FAN.0, 2 * k + 1),
        ) {
            (Some(t), Some(s)) if t != 0 => points.push((t as i32, s as i32)),
            _ => break,
        }
        k += 1;
    }
    points
}

/// Merge a CLI-provided set with an XML-provided set. The XML wins on any field
/// it actually sets; the CLI fills in every field the XML leaves unset.
pub fn merge(cli: TuningSet, xml: TuningSet) -> TuningSet {
    TuningSet {
        min_clock: xml.min_clock.or(cli.min_clock),
        max_clock: xml.max_clock.or(cli.max_clock),
        voltage: xml.voltage.or(cli.voltage),
        mem_clock: xml.mem_clock.or(cli.mem_clock),
        power_limit: xml.power_limit.or(cli.power_limit),
        fan_curve: xml.fan_curve.or(cli.fan_curve),
    }
}

/// Blank out domains named in `skips` (e.g. "fan", "voltage", "clock").
pub fn apply_skips(set: &mut TuningSet, skips: &[String]) {
    for s in skips {
        match s.to_ascii_lowercase().as_str() {
            "clock" => {
                set.min_clock = None;
                set.max_clock = None;
            }
            "min-clock" => set.min_clock = None,
            "max-clock" => set.max_clock = None,
            "voltage" | "volt" => set.voltage = None,
            "mem" | "vram" | "mem-clock" => set.mem_clock = None,
            "power" | "power-limit" => set.power_limit = None,
            "fan" => set.fan_curve = None,
            _ => {}
        }
    }
}

/// Parsed arguments for an apply/plan operation, shared by both binaries.
#[derive(Debug, Default)]
pub struct ApplyArgs {
    /// XML profile path, if given (positional or `--xml`).
    pub xml: Option<String>,
    /// Tuning values taken directly from CLI flags.
    pub cli: TuningSet,
    /// Domains to exclude from the final set.
    pub skips: Vec<String>,
    /// `--log` given (only failures are logged).
    pub log: bool,
    /// Explicit `--log=<path>`.
    pub log_path: Option<String>,
}

/// Parse the tuning flags shared by `apply`/`plan`/silent-apply.
/// Recognizes `--xml <p>` or a bare positional as the profile path,
/// `--gpu-min-clock/--gpu-max-clock/--voltage/--mem-clock/--power-limit <n>`,
/// `--fan-curve "t:s,t:s,..."`, `--skip <csv>`, `--log[=path]`.
pub fn parse_apply_args(tokens: &[String]) -> Result<ApplyArgs, String> {
    let mut a = ApplyArgs::default();
    let mut i = 0;
    // helper to fetch the value following a flag
    let need = |i: &mut usize, name: &str| -> Result<String, String> {
        *i += 1;
        tokens
            .get(*i)
            .cloned()
            .ok_or_else(|| format!("{name} needs a value"))
    };
    while i < tokens.len() {
        let t = &tokens[i];
        match t.as_str() {
            "--xml" => a.xml = Some(need(&mut i, "--xml")?),
            "--gpu-min-clock" => a.cli.min_clock = Some(parse_int(&need(&mut i, "--gpu-min-clock")?)?),
            "--gpu-max-clock" => a.cli.max_clock = Some(parse_int(&need(&mut i, "--gpu-max-clock")?)?),
            "--voltage" => a.cli.voltage = Some(parse_int(&need(&mut i, "--voltage")?)?),
            "--mem-clock" => a.cli.mem_clock = Some(parse_int(&need(&mut i, "--mem-clock")?)?),
            "--power-limit" => a.cli.power_limit = Some(parse_int(&need(&mut i, "--power-limit")?)?),
            "--fan-curve" => a.cli.fan_curve = Some(parse_fan_curve(&need(&mut i, "--fan-curve")?)?),
            "--skip" => {
                for s in need(&mut i, "--skip")?.split(',') {
                    let s = s.trim();
                    if !s.is_empty() {
                        a.skips.push(s.to_string());
                    }
                }
            }
            "--log" => a.log = true,
            other if other.starts_with("--log=") => {
                a.log = true;
                a.log_path = Some(other["--log=".len()..].to_string());
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag: {other}"));
            }
            _ => {
                if a.xml.is_none() {
                    a.xml = Some(t.clone());
                } else {
                    return Err(format!("unexpected argument: {t}"));
                }
            }
        }
        i += 1;
    }
    Ok(a)
}

/// Resolve the final TuningSet from parsed args: XML (if any) merged over CLI,
/// XML winning per field, then skips removed.
pub fn resolve_set(args: &ApplyArgs) -> Result<TuningSet, String> {
    let mut set = match &args.xml {
        Some(path) => {
            let sys = System::from_file(path).map_err(|e| e.to_string())?;
            merge(args.cli.clone(), profile_to_tuning_set(&sys))
        }
        None => args.cli.clone(),
    };
    apply_skips(&mut set, &args.skips);
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xml(mem_enabled: &str, mem_val: i32) -> String {
        format!(
            r#"<SYSTEM><GPU DevID="73BF" RevID="C0">
                <FEATURE ID="5" Enabled="{mem_enabled}"><STATES>
                    <STATE ID="0" Enabled="False" Value="{mem_val}"/>
                    <STATE ID="1" Enabled="False" Value="0"/></STATES></FEATURE>
            </GPU></SYSTEM>"#
        )
    }

    #[test]
    fn mem_skipped_when_feature_disabled() {
        // Stock export: FEATURE 5 disabled, carries default 2000 -> must NOT apply.
        let sys = System::from_str(&xml("False", 2000)).unwrap();
        assert_eq!(profile_to_tuning_set(&sys).mem_clock, None);
    }

    #[test]
    fn mem_applied_when_feature_enabled() {
        let sys = System::from_str(&xml("True", 2046)).unwrap();
        assert_eq!(profile_to_tuning_set(&sys).mem_clock, Some(2046));
    }

    #[test]
    fn xml_wins_over_cli_per_field() {
        let cli = TuningSet {
            max_clock: Some(9999),
            power_limit: Some(3),
            ..Default::default()
        };
        let xmlset = TuningSet {
            max_clock: Some(2400),
            ..Default::default()
        };
        let m = merge(cli, xmlset);
        assert_eq!(m.max_clock, Some(2400)); // XML wins
        assert_eq!(m.power_limit, Some(3)); // CLI fills the gap
    }
}

fn parse_int(s: &str) -> Result<i32, String> {
    s.trim()
        .parse::<i32>()
        .map_err(|_| format!("not an integer: {s}"))
}

/// Parse "51:18,58:20,66:28" into [(51,18),(58,20),(66,28)] (temp:speed).
fn parse_fan_curve(s: &str) -> Result<Vec<(i32, i32)>, String> {
    let mut pts = Vec::new();
    for pair in s.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let (t, sp) = pair
            .split_once(':')
            .ok_or_else(|| format!("fan point must be temp:speed, got '{pair}'"))?;
        pts.push((parse_int(t)?, parse_int(sp)?));
    }
    if pts.is_empty() {
        return Err("fan curve is empty".into());
    }
    Ok(pts)
}
