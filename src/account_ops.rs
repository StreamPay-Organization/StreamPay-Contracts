//! Per-account active stream limits for the StreamPay contract.
//!
//! Each sender may hold at most [`crate::constants::DEFAULT_OPERATION_LIMIT`]
//! concurrent active streams unless the admin tightens the cap through
//! [`crate::StreamPayContract::set_operation_limit`]. Slots are reserved on
//! stream creation and released when a stream completes or is cancelled.

use crate::error::Error;
use crate::storage;
use soroban_sdk::{Address, Env};

/// Reserves `count` active-stream slots for `account`.
///
/// Returns [`Error::OperationLimitExceeded`] when the reservation would exceed
/// the configured per-account limit, and [`Error::Overflow`] when the running
/// count would wrap `u32`.
pub fn reserve_slots(env: &Env, account: &Address, count: u32) -> Result<(), Error> {
    if count == 0 {
        return Ok(());
    }

    let limit = storage::read_operation_limit(env);
    let current = storage::read_account_op_count(env, account);
    let new_count = current.checked_add(count).ok_or(Error::Overflow)?;
    if new_count > limit {
        return Err(Error::OperationLimitExceeded);
    }

    storage::write_account_op_count(env, account, new_count);
    Ok(())
}

/// Releases one active-stream slot previously held by `account`.
pub fn release_slot(env: &Env, account: &Address) {
    let current = storage::read_account_op_count(env, account);
    if current > 0 {
        storage::write_account_op_count(env, account, current - 1);
    }
}
