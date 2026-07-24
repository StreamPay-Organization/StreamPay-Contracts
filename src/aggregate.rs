//! Aggregate arithmetic helpers for the StreamPay contract.
//!
//! Batch entrypoints and cancellation paths sum multiple non-negative values
//! (escrow totals, counter bumps, payout splits). Per
//! [ADR 0026](../docs/adr/0026-prefer-saturating-math-for-aggregates.md) those
//! aggregates use saturating addition internally and then verify the result did
//! not clamp, returning [`Error::Overflow`] when the true sum would exceed the
//! type's range.

use crate::error::Error;

/// Adds two `i128` values for aggregate escrow totals and payout splits.
///
/// Uses [`i128::saturating_add`] and returns [`Error::Overflow`] when the
/// operands would sum outside `i128` range.
pub fn add_i128(lhs: i128, rhs: i128) -> Result<i128, Error> {
    let sum = lhs.saturating_add(rhs);
    if sum.saturating_sub(rhs) != lhs {
        return Err(Error::Overflow);
    }
    Ok(sum)
}

/// Adds two `u64` values for aggregate counter bumps.
///
/// Uses [`u64::saturating_add`] and returns [`Error::Overflow`] when the
/// operands would sum past `u64::MAX`.
pub fn add_u64(lhs: u64, rhs: u64) -> Result<u64, Error> {
    let sum = lhs.saturating_add(rhs);
    if sum.saturating_sub(rhs) != lhs {
        return Err(Error::Overflow);
    }
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_i128_sums_positive_values() {
        assert_eq!(add_i128(1_000, 2_000), Ok(3_000));
    }

    #[test]
    fn add_i128_overflows_on_max_plus_one() {
        assert_eq!(add_i128(i128::MAX, 1), Err(Error::Overflow));
    }

    #[test]
    fn add_i128_overflows_on_large_pair() {
        let half = i128::MAX / 2 + 1;
        assert_eq!(add_i128(half, half), Err(Error::Overflow));
    }

    #[test]
    fn add_u64_sums_values() {
        assert_eq!(add_u64(10, 5), Ok(15));
    }

    #[test]
    fn add_u64_overflows_on_max_plus_one() {
        assert_eq!(add_u64(u64::MAX, 1), Err(Error::Overflow));
    }
}
