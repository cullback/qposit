#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RejectReason {
    /// Invalid order id or already cancelled.
    OrderNotFound,
    /// Outside [tick_size, RESOLVE_PRICE - tick_size].
    InvalidPrice,
    /// Size of 0.
    InvalidQuantity,
    /// Book does not exist or already resolved.
    BookNotFound,
    InsufficientFunds,
    IOCNotMarketable,
}
