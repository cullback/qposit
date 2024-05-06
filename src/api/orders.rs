use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use exchange::Action;
use orderbook::OrderId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    actors::matcher_request::MatcherRequest, app_state::AppState, auth::BasicAuthExtractor, models,
};

use super::feed::BookEvent;

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

/// Request for a new order.
#[derive(Debug, Deserialize, ToSchema)]
pub struct OrderRequest {
    /// The id of the book to submit the order to.
    pub book: u32,
    /// The number of contracts to buy or sell.
    #[schema(minimum = 1)]
    pub quantity: u32,
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
            quantity: req.quantity,
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

/// Handles an order POST request.
#[utoipa::path(
    post,
    path = "/orders",
    responses(
        (status = 200, description = "Order successfully submitted", body = [OrderRequest])
    )
)]
pub async fn post(
    State(state): State<AppState>,
    BasicAuthExtractor(user): BasicAuthExtractor,
    Json(order): Json<OrderRequest>,
) -> Response {
    let (req, recv) = MatcherRequest::submit(user.id, order.into());
    state.cmd_send.send(req).await.expect("Receiver dropped");
    let response = recv.await.expect("Sender dropped");

    match response {
        Ok(event) => Json(BookEvent::from(event)).into_response(),
        Err(err) => Json(json!({"error": format!("{err:?}")})).into_response(),
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    id: i64,
    created_at: i64,
    book: u32,
    user: u32,
    quantity: u32,
    remaining: u32,
    price: u16,
    is_buy: bool,
}

impl From<models::order::Order> for OrderResponse {
    fn from(value: models::order::Order) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            book: value.book_id,
            user: value.user_id,
            quantity: value.quantity,
            remaining: value.remaining,
            price: value.price,
            is_buy: value.is_buy,
        }
    }
}

/// Gets the open orders.
#[utoipa::path(
    get,
    path = "/orders",
    responses(
        (status = 200, description = "Get an active order")
    )
)]
pub async fn get(
    BasicAuthExtractor(user): BasicAuthExtractor,
    State(state): State<AppState>,
) -> Response {
    let Ok(orders) = models::order::Order::get_for_user(&state.db, user.id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let resp = orders
        .into_iter()
        .map(OrderResponse::from)
        .collect::<Vec<_>>();
    Json(resp).into_response()
}

/// Delete all open orders.
#[utoipa::path(
    delete,
    path = "/orders",
    responses(
        (status = 200, description = "Deleted all orders")
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    BasicAuthExtractor(user): BasicAuthExtractor,
) -> impl IntoResponse {
    let Ok(orders) = models::order::Order::get_for_user(&state.db, user.id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let mut deleted = vec![];

    for order in orders {
        let (req, recv) = MatcherRequest::cancel(user.id, order.id);
        state.cmd_send.send(req).await.expect("Receiver dropped");
        let resp = recv.await.expect("Sender dropped");
        if let Ok(exchange::BookEvent {
            time: _,
            tick: _,
            book: _,
            user: _,
            action: Action::Remove { id },
        }) = resp
        {
            deleted.push(id);
        };
    }

    Json(json!({"deleted": deleted})).into_response()
}

#[utoipa::path(
    delete,
    path = "/orders/:order_id",
    responses(
        (status = 200, description = "Deleted all orders")
    )
)]
pub async fn delete_by_id(
    State(state): State<AppState>,
    BasicAuthExtractor(user): BasicAuthExtractor,
    Path(order_id): Path<OrderId>,
) -> impl IntoResponse {
    let mut deleted = vec![];

    let (req, recv) = MatcherRequest::cancel(user.id, order_id);
    state.cmd_send.send(req).await.expect("Receiver dropped");
    let resp = recv.await.expect("Sender dropped");

    if let Ok(exchange::BookEvent {
        time: _,
        tick: _,
        book: _,
        user: _,
        action: Action::Remove { id },
    }) = resp
    {
        deleted.push(id);
    };

    Json(json!({"deleted": deleted})).into_response()
}
