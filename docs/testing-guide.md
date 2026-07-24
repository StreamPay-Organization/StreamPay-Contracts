# Testing Guide

This note documents the **testing-guide** of the streampay-contract contract.

streampay-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the testing-guide in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.

## Shared Test Fixtures

Before adding a new test, familiarize yourself with the helpers in
`src/test_helpers.rs`. They exist to keep setup duplication low and to make it easy
to spin up a fresh contract environment for a single assertion.

The most common fixtures and methods are:

| Helper / Method | Purpose |
| --- | --- |
| `Setup::new()` | Fully initialized contract with a funded sender (`1_000_000` tokens). |
| `Setup::new_with_stream()` | Initialized setup with a default active stream (`1_000` tokens, start `100`, end `200`) already deployed. Returns `(Setup, stream_id)`. |
| `Setup::new_with_cap(cap)` | Initialized contract with a preset global supply cap. |
| `UninitializedSetup::new()` | Uninitialized contract environment for `NotInitialized` tests. |
| `set_time(&env, ts)` | Advance the ledger timestamp in-place. |
| `bump_sequence(&env, by)` | Advance the ledger sequence number in-place (used to test TTL updates). |

## Conventions

- One logical assertion per test.
- Use `Setup::new()` or `Setup::new_with_stream()` unless the test specifically needs an uninitialized state.
- Use `set_time()` rather than mutating the ledger directly.
