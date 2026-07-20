# Resource Costs

This document provides a detailed breakdown of the on-chain resource consumption
of the StreamPay contract. Soroban charges for three distinct resources on every
transaction: **CPU instructions**, **storage rent** (byte-ledger fees), and
**network bandwidth** (read/write bytes). Understanding each is necessary to set
transaction fees and TTL budgets correctly.

---

## 1. CPU instruction budget

Soroban enforces a per-transaction instruction limit (10 000 000 instructions on
Stellar mainnet at the time of writing). The table below ranks StreamPay
entrypoints by their relative instruction cost and identifies the dominant
driver.

| Entrypoint | Relative cost | Dominant factor |
| --- | --- | --- |
| `initialize` | Low | 1 instance write + `extend_ttl` call |
| `get_admin` / `get_token` / `stream_counter` | Very low | 1 instance read |
| `get_stream` / `get_status` / `is_active` / `duration` | Very low | 1 persistent read |
| `streamed_amount` / `withdrawable_amount` / `remaining_amount` / `elapsed` / `progress_bps` / `percent_withdrawn` | Very low | 1 persistent read + integer arithmetic |
| `get_summary` | Very low | 1 persistent read + several arithmetic ops |
| `extend_stream` | Low-medium | 1 persistent read/write + `extend_ttl` |
| `top_up` | Medium | 1 `token::transfer` + 1 persistent read/write |
| `create_stream` | Medium | 1 `token::transfer` + 1 persistent write |
| `withdraw` | Medium | 1 persistent read/write + 1 `token::transfer` |
| `cancel` | Medium-high | 1 persistent read/write + up to 2 `token::transfer`s |

**Notes:**

- Every `token::transfer` crosses a contract boundary and carries the bulk of
  the instruction cost for a call. The exact count depends on the SAC
  implementation, but it typically accounts for 60–80 % of a write's
  instruction budget.
- `cancel` can issue two transfers (sender refund + recipient payment) and is
  therefore the most expensive entrypoint. When one side receives zero tokens
  (e.g. the stream is cancelled before it starts or after it has fully vested)
  the corresponding transfer is skipped, reducing cost.
- View functions (those that do not mutate storage or call `token::transfer`)
  cost a fraction of write calls and are safe to call frequently.

### Measuring actual counts

Run a simulation against Testnet before deploying to mainnet:

```bash
stellar contract simulate \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- create_stream \
  --sender <SENDER> \
  --recipient <RECIPIENT> \
  --total_amount 1000 \
  --start_time 1700000000 \
  --end_time   1700086400
```

The response includes `cpu_insns` and `mem_bytes` consumed. Use these numbers to
set the `--fee` flag with an appropriate margin.

---

## 2. Storage layout

StreamPay divides its state across two Soroban storage classes.

### 2.1 Instance storage

Instance storage is cheap (shared TTL with the wasm entry) and is used for
contract-wide singletons.

| Key | Type | Written by | Size (approx.) |
| --- | --- | --- | --- |
| `Admin` | `Address` | `initialize` | 32–64 bytes |
| `Token` | `Address` | `initialize` | 32–64 bytes |
| `Counter` | `u64` | `create_stream` | 8 bytes |

Instance storage TTL is extended on every mutating call via `extend_instance`.

### 2.2 Persistent storage

Each stream is stored under a unique key `Stream(id)` in persistent storage.
Persistent entries have an independent TTL and are the main ongoing rent cost.

| Key | Type | Written by | Size (approx.) |
| --- | --- | --- | --- |
| `Stream(id)` | `Stream` struct | `create_stream`, `top_up`, `extend_stream`, `withdraw`, `cancel` | ~200 bytes |

The `Stream` struct contains two `Address` fields (sender, recipient), three
`i128` fields (total, withdrawn, and internally vested totals are derived),
two `u64` timestamps (start, end), and a `Status` enum.

---

## 3. TTL and rent strategy

### Constants (`src/storage.rs`)

| Constant | Value (ledgers) | Approx. real time (5 s/ledger) |
| --- | --- | --- |
| `BUMP_THRESHOLD` | 100 000 | ~6 days |
| `BUMP_EXTEND` | 518 400 | ~30 days |

### How TTL extension works

Every call to `write_stream` invokes:

```rust
env.storage()
    .persistent()
    .extend_ttl(&key, BUMP_THRESHOLD, BUMP_EXTEND);
```

- If the entry's **remaining TTL** is already **≥ BUMP_THRESHOLD** the network
  does **not** charge for an extension — no rent fee is paid.
- If the remaining TTL has fallen **below BUMP_THRESHOLD**, the network restores
  it to **BUMP_EXTEND** and charges the pro-rated rent for the bump.

The same threshold and target are used for instance storage via
`extend_instance`.

### Rent-cost formula

Soroban charges rent per byte per ledger:

```
rent_fee = size_bytes × (BUMP_EXTEND − current_ttl) × fee_per_byte_ledger
```

`fee_per_byte_ledger` is a network-level parameter; query it with:

```bash
stellar network fee-stats --network testnet
```

For a 200-byte stream entry bumped from 0 to 518 400 ledgers the rent cost at
the current testnet rate (~`1 stroop` per 1 024 byte-ledgers) is roughly
**0.01 XLM**. Streams that are accessed at least once per month never pay a
full-window bump and effectively cost far less.

### Abandoned streams

A stream that is never touched after creation will have its persistent entry
reclaimed by the network once the TTL elapses (~30 days after the last write).
The token funds escrowed in the contract are **not** automatically returned when
an entry expires; callers should monitor streams and withdraw or cancel before
TTL expiry if they want to recover funds.

---

## 4. Wasm binary size

The release profile in `Cargo.toml` is configured for minimal binary size:

```toml
[profile.release]
opt-level       = "z"    # size-optimised
lto             = true   # link-time optimisation across crates
codegen-units   = 1      # single codegen unit, better dead-code elimination
debug           = 0      # no debug info
strip           = "symbols"
panic           = "abort"
overflow-checks = true   # retain safety; cost is negligible at opt-level z
```

The compiled wasm is uploaded once and stored under the ledger's `ContractCode`
entry. Its one-time upload fee and ongoing rent depend on its size. The above
settings typically produce a binary under **10 kB** for a contract of this
complexity. Run `make optimize` (which invokes `stellar contract optimize`) to
further compress the binary with Binaryen's `wasm-opt`.

---

## 5. Fee estimation guidance

### Testnet quick-reference

| Operation | Suggested minimum fee |
| --- | --- |
| View call | 0.001 XLM |
| Single-transfer write (`create_stream`, `top_up`, `withdraw`) | 0.01 XLM |
| Double-transfer write (`cancel`) | 0.02 XLM |

These are conservative estimates. Use `stellar contract invoke --fee` to specify
a custom fee, and run simulations first to confirm actual resource usage.

### Simulation workflow

```bash
# 1. Simulate to get the resource envelope.
stellar contract simulate \
  --id <CONTRACT_ID> --source alice --network testnet -- <ENTRYPOINT> [args]

# 2. Deploy with an explicit fee (in stroops; 1 XLM = 10 000 000 stroops).
stellar contract invoke \
  --id <CONTRACT_ID> --source alice --network testnet \
  --fee 100000 \
  -- <ENTRYPOINT> [args]
```

### Mainnet considerations

- Mainnet instruction and fee limits are typically stricter than Testnet; always
  validate simulation results against the mainnet network parameters.
- Storage rent rates on mainnet can differ from Testnet; re-run fee-stats
  against `--network mainnet` before launching.

---

## 6. Related documents

- [`docs/ttl-and-rent.md`](ttl-and-rent.md) — deeper discussion of Soroban TTL
  mechanics.
- [`docs/gas-and-fees.md`](gas-and-fees.md) — transaction fee structure and
  fee-bump transactions.
- [`docs/performance-notes.md`](performance-notes.md) — general performance
  notes and optimisation decisions.
- [`docs/adr/0034-budget-resources-and-instructions.md`](adr/0034-budget-resources-and-instructions.md) —
  ADR documenting the decision to track and document resource budgets.
- [`docs/adr/0005-define-a-ttl-bump-strategy-for-persistent-entries.md`](adr/0005-define-a-ttl-bump-strategy-for-persistent-entries.md) —
  ADR documenting the choice of `BUMP_THRESHOLD` and `BUMP_EXTEND` values.
