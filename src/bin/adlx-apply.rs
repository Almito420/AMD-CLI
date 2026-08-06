//! Silent GPU tuning applier for Task Scheduler / autostart.
//!
//! Built as a "windows" subsystem app: it NEVER opens a console window and
//! prints nothing. Inputs are the same as `adlx-profile apply` (CLI flags
//! and/or an XML profile, XML overriding per field; CPU always ignored).
//!
//! Logging is OFF by default. Pass `--log` (or `--log=<path>`) to record only
//! FAILURES to a log file (default `adlx-apply.log` next to the exe). Success
//! stays silent.
//!
//! Examples:
//!   adlx-apply.exe "C:\p\profile.xml"                 (apply whole profile, silent)
//!   adlx-apply.exe --gpu-max-clock 2500               (just one setting)
//!   adlx-apply.exe --skip fan "C:\p\profile.xml"      (profile, but leave fans alone)
//!   adlx-apply.exe --log "C:\p\profile.xml"           (log failures)
//!
//! Exit codes: 0 ok, 1 apply error, 2 bad arguments / nothing to apply.
#![windows_subsystem = "windows"]

use adlx_profile::{adlx, parse_apply_args, resolve_set, ApplyArgs};
use std::io::Write;

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let tokens: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_apply_args(&tokens) {
        Ok(a) => a,
        // Respect "silent by default": a bad invocation is signalled only by the
        // exit code (2), not by a log the user did not ask for.
        Err(_) => return 2,
    };

    let set = match resolve_set(&args) {
        Ok(s) => s,
        Err(e) => {
            log(&args, &format!("ERROR: {e}"));
            return 1;
        }
    };

    let nothing = set.min_clock.is_none()
        && set.max_clock.is_none()
        && set.voltage.is_none()
        && set.mem_clock.is_none()
        && set.power_limit.is_none()
        && set.fan_curve.is_none();
    if nothing {
        log(&args, "ERROR: nothing to apply (no XML and no tuning flags)");
        return 2;
    }

    match do_apply(&set) {
        Ok(lines) => {
            let problems: Vec<&str> = lines
                .iter()
                .filter(|l| l.contains("FAILED") || l.contains("not supported"))
                .map(String::as_str)
                .collect();
            if problems.is_empty() {
                0
            } else {
                log(&args, &format!("PARTIAL: {}", problems.join("; ")));
                1
            }
        }
        Err(e) => {
            log(&args, &format!("ERROR: {e}"));
            1
        }
    }
}

fn do_apply(set: &adlx::TuningSet) -> Result<Vec<String>, String> {
    let adlx = adlx::Adlx::init()?;
    adlx.apply(set)
}

fn log(args: &ApplyArgs, msg: &str) {
    if args.log {
        write_log(args.log_path.as_deref(), msg);
    }
}

/// Append a timestamped line to the log file. None -> adlx-apply.log next to exe.
fn write_log(path: Option<&str>, msg: &str) {
    let target = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => match std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("adlx-apply.log"))) {
            Some(p) => p,
            None => return,
        },
    };
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(target) {
        let _ = writeln!(f, "[epoch {epoch}] {msg}");
    }
}
