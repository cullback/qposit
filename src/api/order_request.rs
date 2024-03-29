use serde::Deserialize;
use utoipa::ToSchema;

/// Request for a new order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
pub struct OrderRequest {
    /// The id of the book to submit the order to.
    pub book: u32,
    /// The number of contracts to buy or sell.
    #[schema(minimum = 1)]
    pub size: u32,
    /// The price to buy or sell at. If not present, order will be a market order.
    #[schema(minimum = 1, maximum = 99)]
    pub price: u16,
    /// Whether to buy or sell.
    pub is_buy: bool,
    #[serde(default = "TimeInForce::gtc")]
    pub tif: TimeInForce,
}

impl From<OrderRequest> for exchange::OrderRequest {
    fn from(req: OrderRequest) -> Self {
        Self {
            book: req.book,
            size: req.size,
            price: req.price,
            is_buy: req.is_buy,
            tif: match req.tif {
                TimeInForce::GTC => exchange::TimeInForce::GTC,
                TimeInForce::IOC => exchange::TimeInForce::IOC,
                TimeInForce::POST => exchange::TimeInForce::POST,
            },
        }
    }
}

/// The time in force of an order.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[allow(clippy::upper_case_acronyms)]
pub enum TimeInForce {
    /// Good until canceled.
    #[default]
    GTC,
    /// Immediate or cancel.
    IOC,
    /// Don't take liquidity.
    POST,
}

impl TimeInForce {
    pub fn ioc() -> Self {
        Self::IOC
    }

    pub fn gtc() -> Self {
        Self::GTC
    }
}
