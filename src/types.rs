//! Core data types for the StreamPay contract.

use soroban_sdk::{contracttype, Address};

/// Parameters for one stream in a [`crate::StreamPayContract::create_stream_batch`]
/// call.
///
/// Every item in a batch shares the entrypoint's sender, while allowing a
/// distinct recipient, amount, and vesting window.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamRequest {
    /// The account receiving this stream.
    pub recipient: Address,
    /// The amount to escrow for this stream.
    pub total_amount: i128,
    /// The ledger timestamp at which vesting begins.
    pub start_time: u64,
    /// The ledger timestamp at which vesting completes.
    pub end_time: u64,
}

/// Lifecycle status of a payment stream.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// The stream is active and vesting over time.
    Active = 0,
    /// The stream was cancelled before its end time.
    Cancelled = 1,
    /// The stream has been fully withdrawn.
    Completed = 2,
}

/// A computed, point-in-time snapshot of a stream's vesting figures.
///
/// Returned by view calls so off-chain clients can fetch the headline numbers
/// in a single round trip instead of combining several getters.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamSummary {
    /// The total amount escrowed for the stream.
    pub total: i128,
    /// The amount vested so far at the queried timestamp.
    pub vested: i128,
    /// The amount already withdrawn by the recipient.
    pub withdrawn: i128,
    /// The vested-but-unwithdrawn amount available to the recipient now.
    pub withdrawable: i128,
    /// Vesting progress in basis points (0..=10_000) by elapsed time.
    pub progress_bps: u32,
    /// The current lifecycle status of the stream.
    pub status: Status,
}

/// A linear payment stream.
///
/// Tokens vest linearly from `start` to `end`. The escrowed `total` is held by
/// the contract; `withdrawn` tracks how much the recipient has already pulled.
///
/// ## Schedule mutation and the accrual checkpoint
///
/// `accrued` and `accrued_at` form a checkpoint that pins already-vested funds
/// whenever the schedule is mutated (e.g. `extend_stream`).  The vesting math
/// is split into two segments:
///
/// ```text
/// vested(now) = accrued + linear(accrued_at..end, total - accrued, now)
/// ```
///
/// On creation both fields are `0`, so the formula reduces to the simple
/// linear case.  On each `extend_stream` the contract snapshots the current
/// vested amount into `accrued` and records `now` as `accrued_at`, ensuring
/// that pushing the end time forward never reduces what has already vested.
/// This guarantees `withdrawable` is monotonically non-decreasing over time
/// and that `vested - withdrawn >= 0` always holds.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    /// The account funding the stream.
    pub sender: Address,
    /// The account receiving the streamed tokens.
    pub recipient: Address,
    /// The total amount escrowed for the stream.
    pub total: i128,
    /// The amount already withdrawn by the recipient.
    pub withdrawn: i128,
    /// The ledger timestamp at which vesting begins.
    pub start: u64,
    /// The ledger timestamp at which vesting completes.
    pub end: u64,
    /// The current lifecycle status of the stream.
    pub status: Status,
    /// Amount locked in as already-vested at the last schedule mutation.
    ///
    /// On creation this is `0`.  It is set to `vested(now)` whenever the
    /// schedule is mutated (e.g. `extend_stream`) so that past vesting is
    /// never recalculated under a new window.
    pub accrued: i128,
    /// Ledger timestamp at which `accrued` was last snapshotted.
    ///
    /// Linear vesting for the remaining `total - accrued` runs from this
    /// point to `end`.  On creation this equals `start`.
    pub accrued_at: u64,
}
