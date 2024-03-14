use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{headers::{authorization::{Basic, Bearer}, Authorization}, TypedHeader};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::app_state::AppState;

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

// /// Handles an order POST request.
// #[utoipa::path(
//     post,
//     path = "/orders",
//     responses(
//         (status = 200, description = "Post an order", body = [OrderRequest])
//     )
// )]
// pub async fn post(
//     State(state): State<AppState>,
//     TypedHeader(auth): TypedHeader<Bearer>,
//     Json(payload): Json<OrderRequest>,
// ) -> Response {
//     let Some(user_id) = state.authenticate(&auth).await else {
//         return StatusCode::UNAUTHORIZED.into_response();
//     };

//     let (req, recv) = EngineRequest::submit(user_id, payload);
//     state.matcher.send(req).await.expect("Receiver dropped");
//     let response = recv.await.expect("Sender dropped");

//     match response {
//         Ok(event) => Json(event).into_response(),
//         Err(err) => Json(json!({"error": format!("{err:?}")})).into_response(),
//     }
// }

#[utoipa::path(
    get,
    path = "/orders",
    responses(
        (status = 200, description = "Get an active order")
    )
)]
pub async fn get(
    State(state): State<AppState>,
    // TypedHeader(auth): TypedHeader<Bearer>,
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,

) -> Response {
    return StatusCode::UNAUTHORIZED.into_response();
}
