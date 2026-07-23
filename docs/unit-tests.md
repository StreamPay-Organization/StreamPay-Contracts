# Unit Tests

The StreamPay unit tests validate every entrypoint, view function, and
lifecycle edge case. They live in `src/test.rs` and are run with:

```bash
make test
```

## Test Fixtures

All tests share a common set of fixtures defined in `src/test_helpers.rs`
so that new tests can be written quickly and existing tests stay consistent.

### `Setup::new()`

The primary helper for the majority of tests. It returns a [`Setup`] struct
containing:

- `env` — a mocked `Env` with `mock_all_auths()` enabled
- `contract` — an initialized `StreamPayContractClient`
- `token` — a `TokenClient` for the streamed SAC
- `token_admin` — a `StellarAssetClient` for minting/burning tokens
- `admin`, `sender`, `recipient` — pre-generated `Address` values
- The `sender` is pre-funded with `1_000_000` tokens

```rust
let s = Setup::new();
let id = s.contract.create_stream(&s.sender, &s.recipient, &1_000, &100, &200);
```

### `Setup::new_with_stream()`

Convenience helper that initializes a standard setup and deploys a default stream of 1,000 tokens from timestamp 100 to 200. Returns the setup and the stream ID.

```rust
let (s, id) = Setup::new_with_stream();
```

### `Setup::new_with_cap(cap)`

Like `Setup::new()` but pre-loads a global supply cap of `cap`.

```rust
let s = Setup::new_with_cap(500);
assert_eq!(s.contract.get_supply_cap(), 500);
```

### `UninitializedSetup::new()`

Returns an uninitialized environment and a `StreamPayContractClient` client for tests
that need to assert `NotInitialized` errors.

```rust
let u = UninitializedSetup::new();
assert_eq!(u.contract.try_get_admin(), Err(Ok(Error::NotInitialized)));
```

### Time control

Use [`set_time(&env, ts)`](src/test_helpers.rs:set_time) to advance the ledger
timestamp inside a test.

```rust
set_time(&s.env, 150);
assert_eq!(s.contract.streamed_amount(&id), 500);
```

## Writing New Tests

1. Add the `#[test]` attribute and a descriptive name.
2. Use `Setup::new()` or `Setup::new_with_stream()` for a standard initialized contract.
3. Use `UninitializedSetup::new()` only when asserting `NotInitialized`.
4. Use `set_time()` to move the clock forward for vesting calculations.
5. Keep tests focused: one logical assertion per test.
