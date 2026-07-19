//! Input normalization helpers for the StreamPay contract.
//!
//! Entry points accept raw caller-supplied values (timestamps, sentinel
//! shorthands) that need to be brought into canonical form before validation
//! and storage. Centralizing that normalization here keeps the rules out of
//! the business logic in `lib.rs` and guarantees every entry point applies
//! the same conventions.

/// Sentinel `start_time` meaning "start at the current ledger timestamp".
///
/// Passing `0` as `start_time` to
/// [`crate::StreamPayContract::create_stream`] is shorthand for starting the
/// stream immediately, so callers do not have to read the ledger clock first.
pub const START_NOW: u64 = 0;

/// Normalizes a caller-supplied `start_time` against the ledger clock `now`.
///
/// Returns `now` when `start_time` is the [`START_NOW`] sentinel (`0`), and
/// the unchanged `start_time` otherwise. Applied by `create_stream` before
/// its time-range validation, so a normalized start still goes through the
/// usual `end > start` checks.
pub fn normalize_start_time(now: u64, start_time: u64) -> u64 {
    if start_time == START_NOW {
        now
    } else {
        start_time
    }
}

/// Clamps a timestamp `now` into the window `[start, end]`.
///
/// Returns `start` before the window opens, `end` after it closes, and `now`
/// unchanged in between. The vesting views use this so time outside the
/// stream window never produces out-of-range elapsed values.
pub fn clamp_to_window(start: u64, end: u64, now: u64) -> u64 {
    if now <= start {
        start
    } else if now >= end {
        end
    } else {
        now
    }
}
