//! Mapping of Adrenalin FEATURE IDs to tuning parameters.
//!
//! The core clock / voltage / power IDs below were CONFIRMED by diffing an
//! Adrenalin export before/after changing one known value at a time on an
//! RX 6900 XT (DevID 73BF, RDNA2):
//!   FEATURE 26 STATE 3 = GPU min core clock (MHz)   1333 -> 1343
//!   FEATURE 26 STATE 4 = GPU max core clock (MHz)   2370 -> 2413
//!   FEATURE 12 STATE 0 = GPU voltage (mV)           1100 -> 1074
//!   FEATURE  3 STATE 0 = power limit (%)               0 -> 1
//!   FEATURE 22          = fan curve (temp/speed)
//!   FEATURE  5 STATE 0 = memory/VRAM max clock (MHz)  2000 -> 2046 / 2020
//!
//! The remaining IDs are still unconfirmed guesses.

/// FEATURE/STATE coordinates of the confirmed tuning values.
pub const MIN_CLOCK: (u32, u32) = (26, 3);
pub const MAX_CLOCK: (u32, u32) = (26, 4);
pub const VOLTAGE: (u32, u32) = (12, 0);
pub const POWER_LIMIT: (u32, u32) = (3, 0);
pub const MEM_CLOCK: (u32, u32) = (5, 0);
/// Fan curve feature id (.0); states hold temp/speed pairs (even=temp, odd=speed).
pub const FAN: (u32, u32) = (22, 0);

/// Returns a label for a FEATURE ID, and whether the mapping is confirmed.
pub fn label(id: u32) -> Option<(&'static str, bool)> {
    Some(match id {
        26 => ("GPU core clock (state 3=min, state 4=max, MHz)", true),
        12 => ("GPU voltage (mV)", true),
        3 => ("power limit (%)", true),
        22 => ("fan curve (temp/speed)", true),
        5 => ("memory / VRAM max clock (MHz)", true),
        8 => ("GPU voltage offset?", false),
        18 => ("(unknown)", false),
        17 => ("(unknown)", false),
        19 => ("(unknown)", false),
        20 => ("(unknown)", false),
        21 => ("(unknown)", false),
        27 => ("(unknown)", false),
        100 | 101 | 102 => ("(unknown, high-id feature)", false),
        _ => return None,
    })
}
