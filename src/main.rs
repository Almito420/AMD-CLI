//! adlx-profile: read an AMD Adrenalin tuning XML profile and/or CLI flags and
//! apply GPU tuning (core clock, voltage, memory clock, power, fan) via ADLX.
//!
//! Inputs merge: CLI flags are the base, an XML profile overrides per field
//! (XML wins), and `--skip <domains>` removes any you don't want applied. An
//! XML applies EVERYTHING it contains (fan included); the CPU section is always
//! ignored.

use adlx_profile::{adlx, feature, parse_apply_args, profile::System, resolve_set, ApplyArgs};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("dump") => match args.get(2) {
            Some(path) => match System::from_file(path) {
                Ok(sys) => {
                    dump(&sys);
                    ExitCode::SUCCESS
                }
                Err(e) => fail(&e.to_string()),
            },
            None => usage("dump <profile.xml>"),
        },
        Some("gpu-info") => run(gpu_info),
        Some("reset") => run(|adlx| {
            adlx.reset_to_factory()?;
            println!("reset all GPU tuning to factory (stock)");
            Ok(())
        }),
        Some("set-min-clock") => match args.get(2).and_then(|s| s.parse::<i32>().ok()) {
            Some(mhz) => run(move |adlx| {
                let applied = adlx.set_gfx_min_clock(mhz)?;
                println!("min clock -> {applied} MHz");
                Ok(())
            }),
            None => usage("set-min-clock <MHz>"),
        },
        Some("plan") => match parse_apply_args(&args[2..]) {
            Ok(a) => match resolve_set(&a) {
                Ok(set) => {
                    println!("would apply ({}):", source_label(&a));
                    print_set(&set);
                    ExitCode::SUCCESS
                }
                Err(e) => fail(&e.to_string()),
            },
            Err(e) => fail(&e),
        },
        Some("apply") => match parse_apply_args(&args[2..]) {
            Ok(a) => {
                let set = match resolve_set(&a) {
                    Ok(s) => s,
                    Err(e) => return fail(&e),
                };
                run(move |adlx| {
                    println!("applying ({}):", source_label(&a));
                    print_set(&set);
                    for line in adlx.apply(&set)? {
                        println!("  {line}");
                    }
                    Ok(())
                })
            }
            Err(e) => fail(&e),
        },
        _ => help(),
    }
}

fn run(f: impl FnOnce(&adlx::Adlx) -> Result<(), String>) -> ExitCode {
    let adlx = match adlx::Adlx::init() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("ADLX init failed: {e}");
            eprintln!("(is the AMD driver installed? try running as Administrator)");
            return ExitCode::FAILURE;
        }
    };
    match f(&adlx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(&e),
    }
}

fn fail(msg: &str) -> ExitCode {
    eprintln!("error: {msg}");
    ExitCode::FAILURE
}

fn usage(u: &str) -> ExitCode {
    eprintln!("usage: adlx-profile {u}");
    ExitCode::FAILURE
}

fn source_label(a: &ApplyArgs) -> String {
    match &a.xml {
        Some(p) => format!("from XML {p}, CLI fills gaps"),
        None => "from CLI flags".into(),
    }
}

fn gpu_info(adlx: &adlx::Adlx) -> Result<(), String> {
    let t = adlx.read_gfx_tuning()?;
    println!("GPU: {} (DeviceId {})", t.gpu_name, t.device_id);
    println!(
        "core clock min: {} MHz   (range {}..{} step {})",
        t.cur_min, t.min_range.min, t.min_range.max, t.min_range.step
    );
    println!(
        "core clock max: {} MHz   (range {}..{} step {})",
        t.cur_max, t.max_range.min, t.max_range.max, t.max_range.step
    );
    println!(
        "voltage:        {} mV    (range {}..{} step {})",
        t.cur_volt, t.volt_range.min, t.volt_range.max, t.volt_range.step
    );
    print_opt("mem clock:      ", t.cur_mem, t.mem_range, "MHz");
    print_opt("power limit:    ", t.cur_power, t.power_range, "%");
    Ok(())
}

fn print_opt(label: &str, cur: Option<i32>, range: Option<adlx::AdlxIntRange>, unit: &str) {
    match (cur, range) {
        (Some(v), Some(r)) => println!(
            "{label}{v} {unit}    (range {}..{} step {})",
            r.min, r.max, r.step
        ),
        _ => println!("{label}(not supported)"),
    }
}

fn print_set(set: &adlx::TuningSet) {
    let f = |o: Option<i32>| match o {
        Some(v) => v.to_string(),
        None => "(skip)".to_string(),
    };
    println!("  min clock  : {}", f(set.min_clock));
    println!("  max clock  : {}", f(set.max_clock));
    println!("  voltage    : {}", f(set.voltage));
    println!("  mem clock  : {}", f(set.mem_clock));
    println!("  power limit: {}", f(set.power_limit));
    match &set.fan_curve {
        Some(c) => println!("  fan curve  : {c:?}"),
        None => println!("  fan curve  : (skip)"),
    }
}

fn dump(sys: &System) {
    let g = &sys.gpu;
    println!("GPU  DevID={} RevID={}", g.dev_id, g.rev_id);
    println!("{} feature(s):", g.features.len());
    for feat in &g.features {
        let (label, confident) = feature::label(feat.id).unwrap_or(("(unmapped)", false));
        let mark = if confident { "" } else { " ?" };
        println!(
            "  FEATURE {:>3}  enabled={:<5}  {}{}",
            feat.id,
            feat.enabled_bool(),
            label,
            mark
        );
        for s in &feat.states.state {
            println!(
                "      state {:>2}  enabled={:<5}  value={}",
                s.id,
                s.enabled_bool(),
                s.value
            );
        }
    }
}

fn help() -> ExitCode {
    eprintln!("adlx-profile - AMD Adrenalin GPU tuning tool (via ADLX)\n");
    eprintln!("commands:");
    eprintln!("  gpu-info                     read live GPU tuning + ranges");
    eprintln!("  plan  [flags] [profile.xml]  show what would be applied (no GPU changes)");
    eprintln!("  apply [flags] [profile.xml]  apply tuning (XML overrides CLI per field)");
    eprintln!("  dump  <profile.xml>          parse and print a raw profile");
    eprintln!("  reset                        reset all tuning to factory/stock");
    eprintln!("  set-min-clock <MHz>          convenience: set only GPU min clock\n");
    eprintln!("apply/plan flags (any combination):");
    eprintln!("  --xml <path> | <path>        XML profile (applies everything, CPU ignored)");
    eprintln!("  --gpu-min-clock <MHz>        GPU minimum core clock");
    eprintln!("  --gpu-max-clock <MHz>        GPU maximum core clock (offset on RDNA4)");
    eprintln!("  --voltage <mV>               GPU voltage (offset on RDNA4)");
    eprintln!("  --mem-clock <MHz>            memory / VRAM max clock");
    eprintln!("  --power-limit <%>            power limit");
    eprintln!("  --fan-curve \"t:s,t:s,...\"     fan curve, e.g. 51:18,58:20,66:28");
    eprintln!("  --skip <csv>                 exclude domains: clock,voltage,mem,power,fan");
    ExitCode::FAILURE
}
