# Initialization

This document describes the initialization process for `streampay-contract`.

## `initialize(admin, token)`

The contract is a one-shot, single-instance deployment. Initialization must be called exactly once after deployment; any subsequent call returns `AlreadyInitialized`.

### What it sets up

| Storage key | Type | Initial value | Description |
|---|---|---|---|
| `Admin` | `Address` | `admin` param | The account that can perform privileged operations (e.g. `set_supply_cap`) |
| `Token` | `Address` | `token` param | The SAC address of the token to be streamed |
| `Counter` | `u64` | `0` | Monotonically increasing stream ID counter |
| `TotalSupply` | `i128` | `0` | Running total of tokens currently held in escrow |
| `SupplyCap` | `i128` | `i128::MAX` | Global cap on `TotalSupply`; effectively unlimited until the admin tightens it |
| `OperationLimit` | `u32` | `u32::MAX` | Maximum concurrent active streams per sender; effectively unlimited until the admin tightens it |

All keys are stored in instance storage and have their TTL extended on every write.

Per-sender active stream counts are stored in persistent storage under
`AccountOpCount(address)` and are incremented on stream creation and
decremented when a stream is cancelled or fully withdrawn.

## Guard checks

All state-changing entrypoints check for the presence of the `Admin` key using `has_admin`. If the contract has not been initialized, they return `NotInitialized`.

## Supply cap defaults

The `supply_cap` is initialized to `i128::MAX` so the cap is disabled by default. The admin can call `set_supply_cap` at any time to impose a concrete limit on the total tokens that may be simultaneously escrowed.

## Operation limit defaults

The `operation_limit` is initialized to `u32::MAX` so per-account limits are disabled by default. The admin can call `set_operation_limit` to cap how many concurrent active streams any single sender may hold. Slots are released when a stream is cancelled or fully withdrawn.
