# StreamPay

StreamPay is a real-time payment-streaming smart contract for the
[Stellar](https://stellar.org) network, written in Rust with the
[Soroban SDK](https://developers.stellar.org/docs/build/smart-contracts).

A sender escrows a fixed amount of a token and the contract releases it to a
recipient **linearly over a time window**. The recipient can withdraw the
vested portion at any time, and either party can cancel an active stream to
split the funds fairly between what has and has not yet vested.

## Features

- Linear, time-based vesting of an escrowed token amount.
- On-demand withdrawals of the vested-but-unwithdrawn balance.
- Cancellation that refunds the unstreamed remainder to the sender and pays the
  streamed remainder to the recipient.
- Authorization enforced with `require_auth` on every state-changing call.
- Checked arithmetic throughout to avoid silent overflow.
- Events emitted for stream creation, withdrawal, and cancellation.

## Contract API

| Function | Description |
| --- | --- |
| `initialize(admin, token)` | One-time setup: records the admin and the streamed token (SAC). |
| `create_stream(sender, recipient, total_amount, start_time, end_time) -> u64` | Escrows `total_amount` from `sender` and opens a stream; returns its id. |
| `top_up(id, sender, amount) -> i128` | Sender escrows `amount` more into an active stream; returns the new total. |
| `extend_stream(id, sender, new_end)` | Sender pushes back an active stream's `end` time, slowing vesting. |
| `streamed_amount(id) -> i128` | View: amount vested so far based on the ledger timestamp. |
| `withdrawable_amount(id) -> i128` | View: vested-but-unwithdrawn balance available to the recipient right now. |
| `remaining_amount(id) -> i128` | View: amount not yet vested (the sender's potential refund). |
| `progress_bps(id) -> u32` | View: vesting progress in basis points (0..=10_000) by elapsed time. |
| `percent_withdrawn(id) -> u32` | View: share of the total already withdrawn, in basis points. |
| `duration(id) -> u64` | View: length of the vesting window in seconds. |
| `elapsed(id) -> u64` | View: seconds of the window elapsed so far (clamped to the window). |
| `get_summary(id) -> StreamSummary` | View: total, vested, withdrawn, withdrawable, progress, and status in one call. |
| `get_status(id) -> Status` | View: the stream's lifecycle status. |
| `is_active(id) -> bool` | View: whether the stream is still active. |
| `withdraw(id, recipient) -> i128` | Recipient pulls the vested-but-unwithdrawn balance; returns the amount paid. |
| `cancel(id, caller)` | Sender or recipient cancels; splits funds by vested/unvested. |
| `get_stream(id) -> Stream` | View: the full stream record. |
| `get_admin() -> Address` | View: the configured admin. |
| `get_token() -> Address` | View: the streamed token address. |
| `stream_counter() -> u64` | View: number of streams created so far. |

### Vesting

The vested amount at timestamp `t` is:

```
vested(t) = 0                                              if t <= start
vested(t) = total                                          if t >= end
vested(t) = total * (t - start) / (end - start)            otherwise
```

Integer division truncates, so dust may accrue at the end of the window; it is
always fully released once `t >= end`.

## Events

The contract publishes an event for every lifecycle change so off-chain
indexers can follow streams without polling. Each event's topics carry the
event name and the stream id.

| Topic | Data | Emitted by |
| --- | --- | --- |
| `("created", id)` | `(sender, recipient, total)` | `create_stream` |
| `("toppedup", id)` | `(sender, amount, new_total)` | `top_up` |
| `("extended", id)` | `(sender, old_end, new_end)` | `extend_stream` |
| `("withdrawn", id)` | `(recipient, amount)` | `withdraw` |
| `("cancelled", id)` | `(caller, sender_refund, recipient_paid)` | `cancel` |

## Build

Install the Rust `wasm32-unknown-unknown` target and the
[Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli),
then:

```bash
make build      # compile the optimized release wasm
make test       # run the unit test suite
make fmt        # format the source tree
make clippy     # lint with warnings denied
```

The release artifact is written to:

```
target/wasm32-unknown-unknown/release/streampay_contract.wasm
```

## Deploy

```bash
# Optimize the wasm (optional but recommended).
make optimize

# Deploy to a network. Override SOURCE and NETWORK as needed.
make deploy NETWORK=testnet SOURCE=alice
```

After deploying, initialize the contract once with an admin and the address of
the Stellar Asset Contract (SAC) to stream:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --token <TOKEN_SAC_ADDRESS>
```

## Stream lifecycle

A stream moves through three statuses:

```
Active ──fully withdrawn──▶ Completed
  │
  └──────cancel───────────▶ Cancelled
```

- **Active** — the default after `create_stream`. Funds vest over time, the
  recipient may `withdraw`, and the sender may `top_up` or `extend_stream`.
- **Completed** — set automatically once the recipient has withdrawn the entire
  total. Further withdrawals return `AlreadyCompleted`.
- **Cancelled** — set by `cancel`. The vested-but-unwithdrawn portion is paid to
  the recipient and the unvested remainder is refunded to the sender. Further
  withdrawals return `AlreadyCancelled`.

Once a stream leaves the `Active` status it is terminal: `top_up` and
`extend_stream` return `StreamNotActive`.

## Resource costs

Soroban charges for CPU instructions, storage rent, and network bandwidth on
every transaction. This section summarises what StreamPay costs per operation
so integrators can set fees and TTLs confidently.

### Instruction budget (approximate)

State-changing calls are bounded by their storage and token-transfer work; view
calls are read-only and much cheaper.

| Operation | Relative cost | Dominant factor |
| --- | --- | --- |
| `initialize` | Low | 1 instance write + `extend_ttl` |
| `create_stream` | Medium | 1 `token::transfer` + 1 persistent write |
| `top_up` | Medium | 1 `token::transfer` + 1 persistent read/write |
| `extend_stream` | Low-medium | 1 persistent read/write (no transfer) |
| `withdraw` | Medium | 1 persistent read/write + 1 `token::transfer` |
| `cancel` | Medium-high | 1 persistent read/write + up to 2 `token::transfer`s |
| View functions | Very low | 1 persistent or instance read |

### Storage rent

Soroban charges rent on every byte kept alive in the ledger. StreamPay uses two
storage types:

| Storage class | Keys stored | TTL policy |
| --- | --- | --- |
| Instance | `Admin`, `Token`, `Counter` | Extended on every mutating call |
| Persistent | `Stream(id)` per stream | Extended on every `write_stream` call |

The constants governing TTL are defined in `src/storage.rs`:

| Constant | Value (ledgers) | Approx. real time (5 s/ledger) |
| --- | --- | --- |
| `BUMP_THRESHOLD` | 100,000 | ~6 days |
| `BUMP_EXTEND` | 518,400 | ~30 days |

`BUMP_THRESHOLD` is the *minimum* TTL floor: if the remaining TTL is already
above this value no extension fee is paid. `BUMP_EXTEND` is the *target* TTL
restored on each extension. A stream accessed at least once per month never
expires. An abandoned stream's entry is eventually reclaimed by the network.

### Wasm binary size

The optimised release binary is compiled with `opt-level = "z"`, LTO enabled,
and debug symbols stripped (see `Cargo.toml`). These settings minimize
footprint on the ledger, reducing the one-time upload cost and the ongoing rent
for the wasm entry.

### Practical guidance

- Callers should budget a fee of at least **0.01 XLM** for writes and
  **0.001 XLM** for reads on Testnet; confirm with `stellar contract invoke
  --fee` before mainnet use.
- Run `make simulate` (or `stellar contract simulate`) against Testnet to
  measure actual instruction counts before deploying.
- For detailed per-entrypoint budget numbers, TTL configuration rationale, and
  rent-cost formulas see [`docs/resource-costs.md`](docs/resource-costs.md).

## Invariants

- A stream's escrowed balance is always `total - withdrawn` while active; the
  contract never holds less than the sum of its active streams' balances.
- `streamed_amount + remaining_amount == total` at every timestamp.
- `withdrawn` only ever increases and never exceeds `total`.
- All token math is checked, so an overflow returns `Overflow` rather than
  wrapping.

## License

Licensed under the [MIT License](LICENSE).
