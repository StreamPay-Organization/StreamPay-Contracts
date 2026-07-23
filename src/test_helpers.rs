#![cfg(test)]

use crate::error::Error;
use crate::types::{Status, StreamRequest};
use crate::{StreamPayContract, StreamPayContractClient};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env, Vec};

// --- Fixtures ---------------------------------------------------------------


/// Contract state for fully-initialized happy-path tests.
pub struct Setup<'a> {
    pub env: Env,
    pub contract: StreamPayContractClient<'a>,
    pub token: TokenClient<'a>,
    pub token_admin: StellarAssetClient<'a>,
    pub admin: Address,
    pub sender: Address,
    pub recipient: Address,
}

impl<'a> Setup<'a> {
    pub fn new() -> Setup<'a> {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);

        let issuer = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(issuer);
        let token = TokenClient::new(&env, &sac.address());
        let token_admin = StellarAssetClient::new(&env, &sac.address());

        let contract_id = env.register(StreamPayContract, ());
        let contract = StreamPayContractClient::new(&env, &contract_id);
        contract.initialize(&admin, &sac.address());

        token_admin.mint(&sender, &1_000_000);

        Setup {
            env,
            contract,
            token,
            token_admin,
            admin,
            sender,
            recipient,
        }
    }

    /// Create a setup and deploy a default linear stream of 1,000 tokens from 100 to 200.
    pub fn new_with_stream() -> (Setup<'a>, u64) {
        let s = Setup::new();
        let id = s
            .contract
            .create_stream(&s.sender, &s.recipient, &1_000, &100, &200);
        (s, id)
    }

    /// Create a setup and configure a global supply cap of `cap`.
    pub fn new_with_cap(cap: i128) -> Setup<'a> {
        let s = Setup::new();
        s.contract.set_supply_cap(&cap);
        s
    }

    /// Advance the ledger timestamp by `delta` seconds.
    pub fn advance_time(&self, delta: u64) {
        let current = self.env.ledger().timestamp();
        self.env
            .ledger()
            .with_mut(|l| l.timestamp = current + delta);
    }

    /// Set the ledger timestamp to an explicit value.
    pub fn set_time(&self, ts: u64) {
        set_time(&self.env, ts);
    }

    /// Mint `amount` additional tokens to the sender so it can fund new
    /// streams after earlier allocations have been consumed or refunded.
    pub fn mint_sender(&self, amount: i128) {
        self.token_admin.mint(&self.sender, &amount);
    }

    /// Advance the ledger sequence number by `by`.
    pub fn bump_sequence(&self, by: u32) {
        bump_sequence(&self.env, by);
    }

    /// Generate a fresh `Address` bound to this environment.
    pub fn new_address(&self) -> Address {
        Address::generate(&self.env)
    }
}

/// Contract state before `initialize` is called, for testing
/// `NotInitialized` guards.
pub struct UninitializedSetup<'a> {
    pub env: Env,
    pub contract: StreamPayContractClient<'a>,
}

impl<'a> UninitializedSetup<'a> {
    pub fn new() -> UninitializedSetup<'a> {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(StreamPayContract, ());
        let contract = StreamPayContractClient::new(&env, &contract_id);
        UninitializedSetup { env, contract }
    }

    /// Generate a fresh `Address` bound to this environment.
    pub fn new_address(&self) -> Address {
        Address::generate(&self.env)
    }
}

// --- Utilities --------------------------------------------------------------

/// Sets the ledger timestamp used by time-based view functions.
pub fn set_time(env: &Env, ts: u64) {
    env.ledger().with_mut(|l| l.timestamp = ts);
}

/// Advances the ledger sequence number by `by`.
pub fn bump_sequence(env: &Env, by: u32) {
    env.ledger().with_mut(|l| l.sequence_number += by);
}
