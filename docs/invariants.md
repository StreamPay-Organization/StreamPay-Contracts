# Invariants

This document describes the key invariants enforced by the `streampay-contract`.

## Amount invariants

- `stream.total >= MIN_STREAM_AMOUNT (1)` at creation.
- `stream.total` can only grow (via `top_up`) and never decreases.
- `stream.withdrawn <= stream.total` always.
- `total_amount` and `top_up` amounts must satisfy `is_valid_amount`, i.e. be `>= 1`.

## Time invariants

- `stream.end > stream.start` always; the window is always positive.
- `stream.end` can only move forward (via `extend_stream`), never backward.
- A new stream's `end` must be strictly in the future relative to the ledger timestamp at creation.

## Lifecycle invariants

- A `Cancelled` or `Completed` stream is immutable — no further `withdraw`, `top_up`, `cancel`, or `extend_stream` calls are accepted.
- A stream transitions to `Completed` only when `stream.withdrawn >= stream.total`.
- A stream transitions to `Cancelled` only via an explicit `cancel` call by the sender or recipient.

## Token conservation

At cancellation, the entire remaining escrow is always distributed with no tokens lost or created:

```
sender_refund + recipient_paid == stream.total - stream.withdrawn
```

The contract's token balance always equals the sum of unreleased escrow across all streams:

```
contract_balance == Σ (stream.total - stream.withdrawn)  for all Active/partial streams
```

## Global supply cap invariant

The **total supply** tracked in instance storage is the sum of all tokens currently held in escrow:

```
total_supply == Σ stream.total - Σ stream.withdrawn  (across all non-terminal streams)
```

- `create_stream` increments `total_supply` by `total_amount` and fails with `SupplyCapExceeded` if `total_supply + total_amount > supply_cap`.
- `top_up` increments `total_supply` by `amount` and fails with `SupplyCapExceeded` if `total_supply + amount > supply_cap`.
- `withdraw` decrements `total_supply` by the withdrawn amount because tokens have left the contract.
- `cancel` decrements `total_supply` by `recipient_paid + sender_refund` (i.e., the entire released escrow).
- The `supply_cap` is set to `i128::MAX` on initialization (effectively unlimited) and can be lowered or raised by the admin at any time via `set_supply_cap`.
- Tightening the cap below the current `total_supply` does not disturb existing streams but blocks new escrowing until supply drops back below the cap.

## Per-account operation limit invariant

Each sender account tracks how many streams it currently holds in the `Active`
status:

```
account_op_count(sender) == count of Active streams where stream.sender == sender
```

- `create_stream` and `create_stream_batch` reserve one slot per new stream for
  the sender and fail with `OperationLimitExceeded` when
  `account_op_count + new_streams > operation_limit`.
- `cancel` and a completing `withdraw` release one slot for the stream's sender.
- The `operation_limit` defaults to `u32::MAX` on initialization (effectively
  unlimited) and can be adjusted by the admin via `set_operation_limit`.
- Lowering the limit below a sender's current count does not cancel existing
  streams but blocks that sender from opening new ones until slots are released.
