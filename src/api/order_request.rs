use lobster::Side;
use serde::Deserialize;
use utoipa::ToSchema;

/// Request for a new order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
pub struct OrderRequest {
    /// The id of the book to submit the order to.
    #[schema(minimum = 1)]
    pub market: u32,
    /// The number of contracts to buy or sell.
    #[schema(minimum = 1)]
    pub quantity: u32,
    /// The price to buy or sell at. If not present, order will be a event order.
    #[schema(minimum = 1, maximum = 9999)]
    pub price: u16,
    /// Whether to buy or sell.
    pub is_buy: bool,
    /// The time in force of the order. Defaults to good-till-closed ("GTC")
    #[serde(default = "TimeInForce::gtc")]
    pub tif: TimeInForce,
}

impl From<OrderRequest> for lobster::OrderRequest {
    fn from(req: OrderRequest) -> Self {
        Self {
            market: req.market,
            quantity: req.quantity,
            price: req.price,
            side: Side::new(req.is_buy),
            tif: match req.tif {
                TimeInForce::GTC => lobster::TimeInForce::GTC,
                TimeInForce::IOC => lobster::TimeInForce::IOC,
                TimeInForce::POST => lobster::TimeInForce::POST,
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
    pub const fn ioc() -> Self {
        Self::IOC
    }

    pub const fn gtc() -> Self {
        Self::GTC
    }
}
