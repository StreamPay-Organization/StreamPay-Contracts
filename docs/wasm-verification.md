# WASM Hash Verification

## Overview
This document explains how to verify the deployed WASM hash for the StreamPay smart contract.

## Why Verify?
Verifying the WASM hash ensures:
- The deployed contract matches the expected version
- No unauthorized modifications
- Consistent deployments across environments

## Prerequisites
- Rust toolchain installed
- `wasm32-unknown-unknown` target installed
- `sha256sum` or `shasum` utility

## Quick Start

### Verify Hash
```bash
# Verify for testnet (default)
./scripts/verify-wasm-hash.sh

# Verify for mainnet
./scripts/verify-wasm-hash.sh mainnet

# Verify for devnet
./scripts/verify-wasm-hash.sh devnet