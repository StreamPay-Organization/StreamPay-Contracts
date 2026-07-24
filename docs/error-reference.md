# Error Reference

All errors are variants of the `Error` enum in `src/error.rs` and are surfaced
as Soroban contract errors with a stable `u32` discriminant.

| Code | Name | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | `initialize` was called more than once. |
| 2 | `NotInitialized` | The contract has not yet been initialized. |
| 3 | `StreamNotFound` | No stream exists for the given id. |
| 4 | `InvalidAmount` | The provided amount is zero or negative. |
| 5 | `InvalidTimeRange` | The start/end time range is invalid (`end <= start`, or `new_end <= current end`). |
| 6 | `Unauthorized` | The caller is not authorized for this action (not the admin, sender, or recipient as required). |
| 7 | `Overflow` | A checked arithmetic operation overflowed. |
| 8 | `AlreadyCancelled` | The stream has already been cancelled. |
| 9 | `NothingToWithdraw` | No vested-but-unwithdrawn balance is available. |
| 10 | `AlreadyCompleted` | The stream has been fully withdrawn and is complete. |
| 11 | `EndTimeInPast` | The proposed `end_time` is at or before the current ledger timestamp. |
| 12 | `AmountBelowMinimum` | The amount is below `MIN_STREAM_AMOUNT`. |
| 13 | `StreamNotActive` | The stream is cancelled or completed; the operation requires an active stream. |
| 14 | `NoPendingAdminAction` | No admin transfer has been scheduled. |
| 15 | `TimelockNotExpired` | The scheduled admin action cannot execute until its timelock elapses. |
| 16 | `InvalidAdminAction` | The proposed admin action would not change contract state (e.g., transferring admin to the current admin). |
| 17 | *(reserved)* | Reserved for future use. |
| 18 | `SupplyCapExceeded` | The operation would push `total_supply` above the global `supply_cap`. |
| 19 | `EmptyBatch` | A batch entrypoint was called with an empty list of operations. |
| 20 | `StreamAlreadySettled` | `emergency_withdraw` was called on a stream whose remaining balance is zero (fully withdrawn or already cancelled with no residual escrow). |

See the README and the sources under `src/` for the authoritative implementation.
