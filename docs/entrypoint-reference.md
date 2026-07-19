# Entrypoint Reference

This note documents the **entrypoint-reference** of the streampay-contract contract.

streampay-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the entrypoint-reference in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.

## Batch creation

`create_stream_batch(sender, requests)` creates multiple streams from a single
sender. `requests` is a Soroban `Vec<StreamRequest>`; each request contains a
recipient, amount, start time, and end time. The operation requires the
sender's authorization once, validates the complete batch before escrow, and
returns the consecutive stream IDs. An empty or invalid batch fails atomically.
