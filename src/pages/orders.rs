//! Order entry form component.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use exchange::{Action, BookEvent, BookId, RejectReason};
use orderbook::{OrderId, Price, Quantity, Side};
use serde::Deserialize;
use tracing::warn;

use crate::{
    actors::matcher_request::MatcherRequest, app_state::AppState, auth::SessionExtractor,
    pages::templates::order_form::OrderForm,
};

#[derive(Debug, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum OrderType {
    Market,
    GTC,
    IOC,
    POST,
}

#[derive(Debug, Deserialize)]
pub struct PostOrder {
    book: BookId,
    quantity: String,
    // may be empty on market orders
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
    let book = form.book;
    let Some(user) = user else {
        return OrderForm::with_messages(
            form.book,
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
            form.book,
            form.quantity,
            form.price,
            quantity.err().unwrap_or_default().to_owned(),
            price.err().unwrap_or_default().to_owned(),
            "".to_owned(),
        )
        .into_response();
    };

    let req = exchange::OrderRequest {
        book: form.book,
        quantity,
        price,
        side: Side::new(form.is_buy),
        tif: match form.order_type {
            OrderType::Market | OrderType::IOC => exchange::TimeInForce::IOC,
            OrderType::GTC => exchange::TimeInForce::GTC,
            OrderType::POST => exchange::TimeInForce::POST,
        },
    };

    let (req, recv) = MatcherRequest::submit(user.id, req);
    state.cmd_send.send(req).await.unwrap();
    let response = recv.await.expect("Sender dropped");

    match response {
        Ok(BookEvent {
            action: Action::Add(order),
            ..
        }) => OrderForm::with_messages(
            book,
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
                RejectReason::BookNotFound => "Error: Invalid book",
                RejectReason::IOCNotMarketable => "Error: Order not marketable",
                RejectReason::InvalidQuantity => "Error: Invalid quantity",
                RejectReason::InsufficientFunds => "Error: Insufficient funds",
                RejectReason::OrderNotFound => "Error: Order not found",
            };
            OrderForm::with_messages(
                book,
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
