//! Linear vesting math for the StreamPay contract.
//!
//! Given a stream's total amount and time window, [`vested`] computes how much
//! has vested at a particular ledger timestamp:
//!
//! * `0` before `start`
//! * `total` at or after `end`
//! * a linear interpolation in between
//!
//! All arithmetic is checked; on overflow the functions return [`Error::Overflow`].

use crate::error::Error;
use crate::normalize::clamp_to_window;
use crate::types::Stream;

/// Computes the linearly vested amount of `stream` at timestamp `now`.
pub fn vested(stream: &Stream, now: u64) -> Result<i128, Error> {
    if now <= stream.start {
        return Ok(0);
    }
    if now >= stream.end {
        return Ok(stream.total);
    }

    let elapsed = (now - stream.start) as i128;
    let duration = (stream.end - stream.start) as i128;

    let numerator = stream.total.checked_mul(elapsed).ok_or(Error::Overflow)?;
    let result = numerator.checked_div(duration).ok_or(Error::Overflow)?;
    Ok(result)
}

/// Returns the portion of `stream.total` that has not yet vested at `now`.
pub fn unvested(stream: &Stream, now: u64) -> Result<i128, Error> {
    let vested = vested(stream, now)?;
    stream.total.checked_sub(vested).ok_or(Error::Overflow)
}

/// Returns the vested-but-unwithdrawn amount of `stream` at `now`.
///
/// This is `vested(now) - withdrawn`, clamped to be non-negative: the value a
/// withdrawal would transfer to the recipient.
pub fn withdrawable(stream: &Stream, now: u64) -> Result<i128, Error> {
    let vested = vested(stream, now)?;
    let available = vested
        .checked_sub(stream.withdrawn)
        .ok_or(Error::Overflow)?;
    Ok(if available > 0 { available } else { 0 })
}

/// Returns how many seconds of the stream's window have elapsed at `now`.
///
/// The result is clamped to `[0, end - start]`: `0` before `start` and the
/// full window length at or after `end`.
pub fn elapsed(stream: &Stream, now: u64) -> u64 {
    clamp_to_window(stream.start, stream.end, now) - stream.start
}

/// Returns how far the stream's time window has progressed, in basis points.
///
/// The result is `0` before `start`, `10_000` (100%) at or after `end`, and a
/// linear interpolation in between. Unlike [`vested`], this depends only on the
/// time window and not on `total`, so it never overflows.
pub fn progress_bps(stream: &Stream, now: u64) -> u32 {
    if now <= stream.start {
        return 0;
    }
    if now >= stream.end {
        return 10_000;
    }

    let elapsed = (now - stream.start) as u128;
    let duration = (stream.end - stream.start) as u128;
    (elapsed * 10_000 / duration) as u32
}
