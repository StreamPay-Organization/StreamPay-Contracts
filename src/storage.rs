//! Storage layout and access helpers for the StreamPay contract.
//!
//! Instance storage holds singleton configuration (admin, token, counter).
//! Persistent storage holds individual streams keyed by their id.

use crate::types::Stream;
use soroban_sdk::{contracttype, Address, Env};

/// Number of ledgers (~6 days) used as the persistent storage bump threshold.
pub const BUMP_THRESHOLD: u32 = 100_000;
/// Number of ledgers (~30 days) used as the persistent storage extend amount.
pub const BUMP_EXTEND: u32 = 518_400;

/// Keys used to address values in contract storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// The admin address (instance).
    Admin,
    /// The streamed token SAC address (instance).
    Token,
    /// The monotonically increasing stream counter (instance).
    Counter,
    /// A stream stored by its id (persistent).
    Stream(u64),
}

/// Returns `true` if the contract has been initialized (admin is set).
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// Reads the admin address from instance storage.
///
/// Panics if the admin has not been set; callers must guard with
/// [`has_admin`] or the `NotInitialized` error first.
pub fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

/// Writes the admin address into instance storage.
pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Reads the streamed token address from instance storage.
pub fn read_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

/// Writes the streamed token address into instance storage.
pub fn write_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
}

/// Reads the current stream counter, defaulting to zero.
pub fn read_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::Counter)
        .unwrap_or(0)
}

/// Writes the stream counter into instance storage.
pub fn write_counter(env: &Env, counter: u64) {
    env.storage().instance().set(&DataKey::Counter, &counter);
}

/// Returns `true` if a stream exists for the given id.
pub fn has_stream(env: &Env, id: u64) -> bool {
    env.storage().persistent().has(&DataKey::Stream(id))
}

/// Reads a stream from persistent storage, returning `None` if absent.
pub fn read_stream(env: &Env, id: u64) -> Option<Stream> {
    env.storage().persistent().get(&DataKey::Stream(id))
}

/// Writes a stream into persistent storage and extends its lifetime.
pub fn write_stream(env: &Env, id: u64, stream: &Stream) {
    let key = DataKey::Stream(id);
    env.storage().persistent().set(&key, stream);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_EXTEND);
}
