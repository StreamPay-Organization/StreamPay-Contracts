# Authorization

Every state-changing entrypoint on the StreamPay contract requires
authorization from a specific party. Calls that do not carry the required
authorization are rejected with `Error::Unauthorized` (code 6) or a Soroban
auth failure before any state is modified.

## Auth matrix

| Entrypoint | Required signer | Notes |
|---|---|---|
| `initialize` | — | Permissionless; can only succeed once. |
| `set_admin` | current admin | Immediate transfer; use `schedule_admin_transfer` for governance-safe handoffs. |
| `schedule_admin_transfer` | current admin | Queues a timelocked transfer. |
| `execute_admin_transfer` | — | Permissionless after the timelock elapses. |
| `cancel_admin_transfer` | current admin | Cancels a pending transfer. |
| `set_supply_cap` | current admin | |
| `upgrade` | current admin | |
| `create_stream` | `sender` | The account funding the stream. |
| `create_stream_batch` | `sender` | One auth covers all streams in the batch. |
| `top_up` | stream's `sender` | Only the original stream funder may top up. |
| `extend_stream` | stream's `sender` | Only the original stream funder may extend. |
| `withdraw` | stream's `recipient` | Only the designated recipient may pull funds. |
| `cancel` | stream's `sender` **or** `recipient` | Either party may cancel. |
| `emergency_withdraw` | current admin | Break-glass path — see below. |

## Emergency withdrawal

`emergency_withdraw(admin, stream_id, recipient)` is an admin-only entrypoint
that drains a stream's remaining balance to an admin-chosen `recipient`,
bypassing normal vesting rules.

The admin must call `require_auth()` on the `admin` argument, and the contract
additionally verifies that `admin == storage::read_admin(&env)`. A call where
the supplied address is not the stored admin is rejected with
`Error::Unauthorized`.

This entrypoint is intended for break-glass situations only (e.g., a
vulnerability is discovered, or a stream's parties are unresponsive and funds
are at risk). Normal operational use of `cancel` or `withdraw` is always
preferred.

## View entrypoints

All view (`get_*`, `is_*`, `streamed_amount`, etc.) entrypoints are read-only
and require no authorization.

See the README and the sources under `src/` for the authoritative implementation.
