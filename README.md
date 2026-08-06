# adlx-profile

A small, dependency-free Windows CLI that applies AMD Radeon GPU tuning
(core clock, voltage, memory clock, power limit, fan curve) from an **AMD
Adrenalin tuning XML** and/or from individual command-line flags. Built for
**automation** - switch tuning profiles from a `.bat`, Task Scheduler, or a
login script, with no window popping up.

It talks to the GPU through AMD's official **ADLX** API (`amdadlx64.dll`,
installed with the Adrenalin driver) - the same tuning path Adrenalin itself
uses, so it does not fight the driver and nothing is flashed. All changes are
**live and temporary**: a reboot returns the card to stock.

> Unofficial tool, not affiliated with AMD. Overclocking/undervolting is at
> your own risk.

## Supported GPUs

Navi / RDNA2, RDNA3, RDNA4 - i.e. Radeon **RX 6000 / RX 7000 / RX 9000**
series. Older architectures (Vega/Polaris/RX 5000) are not supported.

Validated live on an RX 6900 XT (RDNA2). On RDNA4 (RX 9070 XT) some values are
**offsets** rather than absolutes (see below) - verify with `gpu-info`/`plan`
before relying on it.

## Two binaries

| Binary | Use |
| --- | --- |
| `adlx-profile.exe` | Interactive. Prints to the console (`gpu-info`, `plan`, `apply`, ...). |
| `adlx-apply.exe` | Silent. Never opens a console window, prints nothing. For Task Scheduler / autostart. Logs only failures, and only with `--log`. |

Both share the same apply logic and flags.

## Quick start

```
adlx-profile.exe gpu-info                       read live tuning + ranges
adlx-profile.exe plan  profile.xml              show what would be applied (no changes)
adlx-profile.exe apply profile.xml              apply a full Adrenalin profile
adlx-profile.exe apply --gpu-max-clock 2500     apply just one setting
adlx-profile.exe reset                          reset all tuning to stock
```

## How input works

You can pass an XML profile, individual flags, or both:

- An **XML** profile applies **everything it contains** (clock, voltage, memory,
  power, fan). The `<CPU>` section that newer Adrenalin exports include is
  **always ignored**.
- **CLI flags** let you set any single value without a full XML.
- When both are given, the **XML wins per field** and the CLI fills in the rest.
- `--skip <domains>` excludes anything you don't want touched.

Export a profile from Adrenalin's *Performance > Tuning*; the file lives at
`%LOCALAPPDATA%\AMD\Radeonsoftware\uv.xml`. Copy it somewhere and point the tool
at it.

### Flags (for `apply` and `plan`)

```
--xml <path> | <path>     XML profile (applies everything, CPU ignored)
--gpu-min-clock <MHz>     GPU minimum core clock
--gpu-max-clock <MHz>     GPU maximum core clock   (offset on RDNA4)
--voltage <mV>            GPU voltage              (offset on RDNA4)
--mem-clock <MHz>         memory / VRAM max clock
--power-limit <%>         power limit
--fan-curve "t:s,t:s,..." fan curve, e.g. 51:18,58:20,66:28,76:40,87:100
--skip <csv>              exclude domains: clock,voltage,mem,power,fan
```

Every value is clamped to the range the GPU reports via ADLX. A value of `0`
for clock/voltage in an XML means "not set" and is skipped. Memory is only
applied when the profile actually tuned it (Adrenalin's memory feature flag is
enabled), so applying a stock export won't switch memory into manual mode.

## Automation (Task Scheduler)

Use the silent binary. Create a task with:

- **Program/script:** `C:\path\to\adlx-apply.exe`
- **Arguments:** `"C:\path\to\profile.xml"` (or any flags, e.g. `--skip fan "C:\path\to\profile.xml"`)
- **Trigger:** *At log on*, with a 30-60s delay (so the driver is ready).
- **Run with highest privileges** is recommended.

Because ADLX tuning is not persistent across reboots, an *At log on* task is the
intended way to reapply a profile automatically.

`adlx-apply.exe` is silent by default. Add `--log` (or `--log=C:\path\my.log`)
to record failures to a log file next to the exe; successful runs stay silent.
The exit code always reflects success (0) or failure (1/2).

Example `.bat`:

```bat
@echo off
"%~dp0adlx-apply.exe" "%~dp0profiles\oc.xml"
```

## Notes / caveats

- **Live only.** Tuning is reapplied through the driver and reverts on reboot
  (or sometimes on its own) - reapply via Task Scheduler.
- **RDNA4 offsets.** On RX 9000, GPU max clock and voltage are offsets, not
  absolute values. The tool clamps to whatever range ADLX reports; check with
  `gpu-info`/`plan` on that machine before trusting an `apply`.
- **Fans.** If another program manages your fans, use `--skip fan` (or don't
  pass a fan curve) so the tool leaves them alone.
- **Requires the AMD driver** (provides `amdadlx64.dll`). If a `Set*` call is
  rejected, try running elevated.

## Build

```
cargo build --release
```

Produces `target\release\adlx-profile.exe` and `target\release\adlx-apply.exe`.
Rust 2021+/2024 toolchain, Windows x64.

## License

MIT - see [LICENSE](LICENSE).
