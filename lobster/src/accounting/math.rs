//! The logic for creating/combining shares, and updating positions/balances.
//! Try to keep as much of the math in here as possible to control for errors.

use crate::{Balance, Position};
use crate::{Price, Quantity, Side};

/// The maximum value a contract can resolve to.
pub const RESOLVE_PRICE: Price = 10_000;

/// Computes max(0, x) as a u32. AKA rectified linear unit activation function.
fn relu(x: i32) -> u32 {
    u32::try_from(x).unwrap_or_default()
}

/// Returns the number of contracts created for a sell order based on position.
/// Safe and correct for all inputs.
///
/// max value for `contracts_created(0, Quantity::MAX)`
/// Equivalent to max(quantity - max(0, position), 0)
/// or relu(quantity - relu(position))
/// `0 <= return <= quantity`
#[must_use]
pub fn contracts_created(position: Position, quantity: Quantity) -> Quantity {
    quantity.saturating_sub(relu(position))
}

/// Returns the number of contracts combined for a buy order based on position.
/// Safe and correct for all inputs.
///
/// Equivalent to min(quantity, relu(-position))
/// Max value attained for `contracts_combined(Position::MIN, Quantity::MAX)`
/// if position >= 0, returns 0
/// 0 <= return <= size <= 2^31
#[must_use]
pub fn contracts_combined(position: Position, quantity: Quantity) -> Quantity {
    let pos = position.checked_neg().map_or(1 << 31, relu);
    Quantity::min(quantity, pos)
}

/// Computes the amount a balance should change by if this buy order were to be
/// fully executed.
///
/// Safe and correct for all inputs due to i64 promotion.
/// if pos >= 0 => simplifies to -cost]
#[must_use]
fn buyer_cost(position: Position, quantity: Quantity, price: Price) -> Balance {
    let combined = contracts_combined(position, quantity);
    let cost = Balance::from(quantity).wrapping_mul(Balance::from(price));
    cost.wrapping_sub(Balance::from(combined) * Balance::from(RESOLVE_PRICE))
}

/// Computes the amount a balance should change by if this sell order were to be
/// fully executed.
#[must_use]
fn seller_cost(position: Position, quantity: Quantity, price: Price) -> Balance {
    let created = contracts_created(position, quantity);
    let cost = Balance::from(quantity).wrapping_mul(Balance::from(price));
    (Balance::from(created) * Balance::from(RESOLVE_PRICE)).wrapping_sub(cost)
}

/// Computes the change in balance if an order with these arguments were to be filled.
#[must_use]
pub fn trade_cost(position: Position, quantity: Quantity, price: Price, side: Side) -> Balance {
    match side {
        Side::Buy => buyer_cost(position, quantity, price),
        Side::Sell => seller_cost(position, quantity, price),
    }
}

#[cfg(test)]
mod tests {
    use crate::Position;

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

        assert_eq!(
            contracts_created(Position::MIN, Quantity::MAX),
            Quantity::MAX
        );
        assert_eq!(contracts_created(0, Quantity::MAX), Quantity::MAX);
        assert_eq!(contracts_created(Position::MAX, Quantity::MAX), 2147483648);
        assert_eq!(contracts_created(Position::MAX, 2147483647), 0);
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

        assert_eq!(contracts_combined(Position::MIN, Quantity::MAX), 1 << 31);
    }
}
