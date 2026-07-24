# ADR 0026: Prefer saturating math for aggregates

- Status: Accepted
- Deciders: arisu6804

## Context

The StreamPay smart contract needs a clear, documented approach to "prefer saturating math for aggregates" so the codebase stays consistent and auditable.

Batch entrypoints sum multiple escrow amounts and counter increments; cancellation splits a stream's remaining balance into recipient and sender portions. Those paths must never silently clamp or drop operands when a sum would overflow.

## Decision

We prefer saturating math for aggregates as the standard for this contract, in line with Soroban best practices.

Aggregate sums live in [`src/aggregate.rs`](../src/aggregate.rs):

- `add_i128` — batch escrow totals and cancellation payout splits.
- `add_u64` — batch stream-counter bumps.

Each helper computes with `saturating_add` and returns [`Error::Overflow`](../src/error.rs) when the operands would exceed the type's range. Callers must not use ad-hoc `checked_add(...).unwrap_or(...)` fallbacks on aggregate paths.

Per-stream vesting math and single-operand updates continue to use checked arithmetic per [ADR 0009](0009-use-checked-arithmetic-for-all-math.md).

## Consequences

Improves clarity, testability, and maintainability, and gives future contributors a recorded rationale to build on.
