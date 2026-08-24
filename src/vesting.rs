//! Linear vesting math for the StreamPay contract.
//!
//! ## Arithmetic model
//!
//! Vesting is split into two segments separated by an *accrual checkpoint*
//! stored on every [`Stream`]:
//!
//! ```text
//! vested(now) = stream.accrued
//!             + linear(stream.accrued_at .. stream.end,
//!                      stream.total - stream.accrued,
//!                      now)
//! ```
//!
//! On stream creation `accrued = 0` and `accrued_at = start`, so the formula
//! reduces to the familiar `total * elapsed / duration`.
//!
//! The checkpoint is advanced whenever the schedule is mutated (e.g.
//! [`crate::StreamPayContract::extend_stream`]).  Snapshotting the current
//! vested amount before applying the mutation guarantees:
//!
//! * **Monotonicity** – `vested` never decreases; extension cannot claw back
//!   already-vested funds.
//! * **No over-release** – `withdrawable = vested - withdrawn` is always
//!   non-negative; it is clamped to `0` rather than returning an error.
//! * **Preview/mutation agreement** – `withdrawable_amount` (view) and
//!   `withdraw` (mutation) call the same function, so they always agree.
//!
//! ## Rounding
//!
//! Integer division truncates (floor).  The recipient always receives the
//! strictly earned amount; any sub-unit remainder stays in the contract and
//! is released on the *next* withdrawal once another full unit has accrued.
//! The total released over a stream's lifetime equals `stream.total` because
//! the final withdrawal compares against `stream.total` directly.
//!
//! ## Units
//!
//! Timestamps are ledger timestamps in seconds (Unix epoch, `u64`).  Token
//! amounts are in the token's base unit (`i128`, matching the Stellar token
//! interface).
//!
//! All arithmetic is checked; on overflow the functions return
//! [`Error::Overflow`].

use crate::error::Error;
use crate::normalize::clamp_to_window;
use crate::types::Stream;

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

/// Linear interpolation from `segment_start` to `end` over `remaining`
/// tokens, evaluated at `now`.
///
/// Returns `0` at or before `segment_start`, `remaining` at or after `end`,
/// and a truncating linear interpolation in between.  Both endpoints are
/// handled explicitly so no division by zero can occur.
///
/// The caller must guarantee `end > segment_start`; this invariant is enforced
/// by `create_stream` / `extend_stream` and is safe to assume here.
fn linear_segment(
    segment_start: u64,
    end: u64,
    remaining: i128,
    now: u64,
) -> Result<i128, Error> {
    // If the segment has zero duration (checkpoint landed exactly at end),
    // the entire remaining amount is already vested.
    if end <= segment_start {
        return Ok(remaining);
    }

    if now <= segment_start {
        return Ok(0);
    }
    if now >= end {
        return Ok(remaining);
    }

    let elapsed = (now - segment_start) as i128;
    let duration = (end - segment_start) as i128;
    // `elapsed < duration` so the cast is safe and `duration > 0`.
    let numerator = remaining.checked_mul(elapsed).ok_or(Error::Overflow)?;
    // `duration > 0` so division is safe.
    Ok(numerator / duration)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Computes the total linearly vested amount of `stream` at timestamp `now`.
///
/// Uses the two-segment formula:
/// ```text
/// vested = stream.accrued + linear(stream.accrued_at..stream.end,
///                                   stream.total - stream.accrued, now)
/// ```
///
/// The result is clamped to `[0, stream.total]`.
pub fn vested(stream: &Stream, now: u64) -> Result<i128, Error> {
    let remaining = stream
        .total
        .checked_sub(stream.accrued)
        .ok_or(Error::Overflow)?;

    let segment = linear_segment(stream.accrued_at, stream.end, remaining, now)?;

    let total_vested = stream.accrued.checked_add(segment).ok_or(Error::Overflow)?;

    // Clamp to [0, total] for safety; `total_vested` should already be in
    // range by construction.
    Ok(total_vested.clamp(0, stream.total))
}

/// Returns the portion of `stream.total` that has not yet vested at `now`.
pub fn unvested(stream: &Stream, now: u64) -> Result<i128, Error> {
    let v = vested(stream, now)?;
    stream.total.checked_sub(v).ok_or(Error::Overflow)
}

/// Returns the vested-but-unwithdrawn amount of `stream` at `now`.
///
/// This is `vested(now) - withdrawn`, clamped to `[0, total - withdrawn]`:
/// the exact value that a [`crate::StreamPayContract::withdraw`] call would
/// transfer at this timestamp.
///
/// The clamp to zero (rather than propagating a negative value as an error)
/// is intentional: it preserves monotonicity even if a schedule mutation
/// temporarily makes the raw difference negative on an already-withdrawn
/// stream.
pub fn withdrawable(stream: &Stream, now: u64) -> Result<i128, Error> {
    let v = vested(stream, now)?;
    // `v >= 0` and `withdrawn >= 0`; the difference may be negative for an
    // instant if `withdrawn` was set above `vested` by a schedule extension
    // that increased `accrued` to exactly `withdrawn` — but that cannot
    // happen with the checkpoint pattern because `accrued` is always set to
    // `vested(now)` which is >= `withdrawn` at the snapshot moment.
    //
    // We saturate to 0 defensively rather than using checked_sub to avoid
    // returning an error in the view path.
    Ok((v - stream.withdrawn).max(0))
}

/// Returns how many seconds of the stream's window have elapsed at `now`.
///
/// The result is clamped to `[0, end - start]`.
pub fn elapsed(stream: &Stream, now: u64) -> u64 {
    clamp_to_window(stream.start, stream.end, now) - stream.start
}

/// Returns how far the stream's time window has progressed, in basis points.
///
/// The result is `0` before `start`, `10_000` (100 %) at or after `end`, and
/// a linear interpolation in between.  Unlike [`vested`], this depends only on
/// the time window and not on `total`, so it never overflows.
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

/// Advances the accrual checkpoint on `stream` to `now`.
///
/// This must be called by every entrypoint that mutates the vesting schedule
/// (currently [`crate::StreamPayContract::extend_stream`]) *before* applying
/// the mutation.  It pins the currently-vested amount so that the recalculated
/// segment starts from the present rather than from the original `start`.
///
/// Returns `Err(Error::Overflow)` only if the underlying `vested` call
/// overflows, which is not expected in practice.
pub fn advance_checkpoint(stream: &mut Stream, now: u64) -> Result<(), Error> {
    let v = vested(stream, now)?;
    stream.accrued = v;
    stream.accrued_at = now;
    Ok(())
}
