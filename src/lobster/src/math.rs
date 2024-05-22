//! The logic for creating/combining shares, and updating positions/balances.
//! Try to keep as much of the math in here as possible to control for errors.

use crate::{Price, Quantity, Side};
use crate::{Position, RESOLVE_PRICE};

/// Computes max(0, x) as a u32.
fn to_u32(x: i32) -> u32 {
    u32::try_from(x).unwrap_or_default()
}

/// Returns the number of contracts created for a sell order based on position.
/// Safe and correct for all inputs.
/// max value when position=0, size=MAX
/// max(size - max(0, position), 0)
/// 0 <= return <= size
#[must_use]
fn contracts_created(position: Position, quantity: Quantity) -> Quantity {
    quantity.saturating_sub(to_u32(position))
}

/// Returns the number of contracts combined for a buy order based on position.
/// Safe and correct for all inputs.
///
/// min(size, max(-pos, 0))
/// if pos >= 0, returns 0
/// 0 <= return <= size <= 2^31
#[must_use]
fn contracts_combined(position: Position, quantity: Quantity) -> Quantity {
    let pos = position.checked_neg().map_or(1 << 31, to_u32);
    Quantity::min(quantity, pos)
}

/// Computes the amount a balance should change by if this buy order were to be
/// fully executed.
///
/// Safe and correct for all inputs due to i64 promotion.
/// if pos >= 0 => simplifies to -cost]
#[must_use]
pub fn buyer_cost(position: Position, quantity: Quantity, price: Price) -> i64 {
    let combined = contracts_combined(position, quantity);
    let cost = i64::from(quantity).wrapping_mul(i64::from(price));
    cost.wrapping_sub(i64::from(combined) * i64::from(RESOLVE_PRICE))
}

/// Computes the amount a balance should change by if this sell order were to be
/// fully executed.
#[must_use]
pub fn seller_cost(position: Position, quantity: Quantity, price: Price) -> i64 {
    let created = contracts_created(position, quantity);
    let cost = i64::from(quantity).wrapping_mul(i64::from(price));
    (i64::from(created) * i64::from(RESOLVE_PRICE)).wrapping_sub(cost)
}

#[must_use]
pub fn trade_cost(position: Position, quantity: Quantity, price: Price, side: Side) -> i64 {
    match side {
        Side::Buy => buyer_cost(position, quantity, price),
        Side::Sell => seller_cost(position, quantity, price),
    }
}

#[cfg(test)]
mod tests {
    use super::Quantity;
    use super::{contracts_combined, contracts_created};

    #[test]
    fn test_shares_created() {
        assert_eq!(contracts_created(-1, 2), 2);
        assert_eq!(contracts_created(-2, 0), 0);
        assert_eq!(contracts_created(-2, 1), 1);
        assert_eq!(contracts_created(-2, 2), 2);
        assert_eq!(contracts_created(0, 2), 2);
        assert_eq!(contracts_created(1, 2), 1);
        assert_eq!(contracts_created(2, 2), 0);

        assert_eq!(contracts_created(i32::MIN, Quantity::MAX), Quantity::MAX);
        assert_eq!(contracts_created(0, Quantity::MAX), Quantity::MAX);
        assert_eq!(contracts_created(i32::MAX, Quantity::MAX), 2147483648);
        assert_eq!(contracts_created(i32::MAX, 2147483647), 0);
    }

    #[test]
    fn test_shares_combined() {
        assert_eq!(contracts_combined(-1, 2), 1);
        assert_eq!(contracts_combined(-2, 0), 0);
        assert_eq!(contracts_combined(-2, 1), 1);
        assert_eq!(contracts_combined(-2, 2), 2);
        assert_eq!(contracts_combined(0, 2), 0);
        assert_eq!(contracts_combined(1, 2), 0);
        assert_eq!(contracts_combined(2, 2), 0);
    }
}
