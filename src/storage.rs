//! Storage layout and access helpers for the StreamPay contract.
//!
//! Instance storage holds singleton configuration (admin, token, counter).
//! Persistent storage holds individual streams keyed by their id.

use soroban_sdk::contracttype;

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
