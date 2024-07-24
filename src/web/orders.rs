//! Order entry form component.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use lobster::{Action, MarketId, MarketUpdate, RejectReason};
use lobster::{OrderId, Price, Quantity, Side};
use serde::Deserialize;

use crate::{
    app_state::AppState, services::matcher_request::MatcherRequest,
    web::templates::order_form::OrderForm,
};

use super::auth::SessionExtractor;

#[derive(Debug, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Deserialize)]
pub struct PostOrder {
    market: MarketId,
    quantity: String,
    // may be empty on event orders
    #[serde(default)]
    price: String,
    is_buy: bool,
    order_type: OrderType,
}

pub async fn post(
    State(state): State<AppState>,
    SessionExtractor(user): SessionExtractor,
    Form(form): Form<PostOrder>,
) -> impl IntoResponse {
    let market_id = form.market;
    let Some(user) = user else {
        return OrderForm::with_messages(
            form.market,
            form.quantity,
            form.price,
            String::new(),
            String::new(),
            "Error: must be logged in to place order.".to_string(),
        )
        .into_response();
    };

    let quantity = form
        .quantity
        .parse::<Quantity>()
        .map_err(|_| "Invalid quantity");

    let price = match form.order_type {
        OrderType::Market => Ok(if form.is_buy { 9999 } else { 1 }),
        _ => form
            .price
            .parse::<f32>()
            .ok()
            .filter(|&p| p > 0.0 || p < 100.0)
            .map(|p| (p * 100.0).round() as Price)
            .ok_or("Invalid price"),
    };

    let (Ok(quantity), Ok(price)) = (quantity, price) else {
        return OrderForm::with_messages(
            form.market,
            form.quantity,
            form.price,
            quantity.err().unwrap_or_default().to_owned(),
            price.err().unwrap_or_default().to_owned(),
            "".to_owned(),
        )
        .into_response();
    };

    let req = lobster::OrderRequest {
        market: form.market,
        quantity,
        price,
        side: Side::new(form.is_buy),
        tif: match form.order_type {
            OrderType::Market => lobster::TimeInForce::IOC,
            OrderType::Limit => lobster::TimeInForce::GTC,
        },
    };

    let (req, recv) = MatcherRequest::submit(user.id, req);
    state.cmd_send.send(req).await.unwrap();
    let response = recv.await.expect("Sender dropped");

    match response {
        Ok(MarketUpdate {
            action: Action::Add(order),
            ..
        }) => OrderForm::with_messages(
            market_id,
            form.quantity,
            form.price,
            String::new(),
            String::new(),
            format!("Order accepted! id: {}", order.id),
        )
        .into_response(),
        Err(err) => {
            let msg = match err {
                RejectReason::InvalidPrice => "Error: Invalid price",
                RejectReason::MarketNotFound => "Error: Invalid market",
                RejectReason::IOCNotMarketable => "Error: Order not marketable",
                RejectReason::InvalidQuantity => "Error: Invalid quantity",
                RejectReason::InsufficientFunds => "Error: Insufficient funds",
                RejectReason::OrderNotFound => "Error: Order not found",
                RejectReason::MarketAlreadyExists => "Error: Market already exists",
            };
            OrderForm::with_messages(
                market_id,
                form.quantity,
                form.price,
                String::new(),
                String::new(),
                msg.to_owned(),
            )
            .into_response()
        }
        _ => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}

pub async fn delete_by_id(
    State(state): State<AppState>,
    Path(order_id): Path<OrderId>,
    SessionExtractor(user): SessionExtractor,
) -> impl IntoResponse {
    let Some(user) = user else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let (req, recv) = MatcherRequest::cancel(user.id, order_id);
    state.cmd_send.send(req).await.expect("Receiver dropped");
    let _ = recv.await.expect("Sender dropped");
    Html("").into_response()
}
