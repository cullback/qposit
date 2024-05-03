use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use exchange::{Action, BookId};
use orderbook::{OrderId, Price, Quantity};
use serde::Deserialize;
use tracing::info;

use crate::{
    actors::matcher_request::MatcherRequest, app_state::AppState, auth::SessionExtractor,
    pages::templates::order_form::OrderForm,
};

#[derive(Debug, Deserialize)]
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
        );
    };

    let quantity = form.quantity.parse::<Quantity>().ok();
    let price = form
        .price
        .parse::<f32>()
        .ok()
        .and_then(|p| if p < 0.0 { None } else { Some(p) })
        .map(|p| (p * 100.0).round() as Price);

    let quantity_msg = if quantity.is_some() {
        ""
    } else {
        "invalid quantity"
    };
    let price_msg = if price.is_some() { "" } else { "invalid price" };

    let (Some(quantity), Some(price)) = (quantity, price) else {
        println!(
            "AHHH {}, {}, {}, {}",
            form.quantity, form.price, quantity_msg, price_msg
        );
        return OrderForm::with_messages(
            form.book,
            form.quantity,
            form.price,
            quantity_msg.to_owned(),
            price_msg.to_owned(),
            "error".to_owned(),
        );
    };

    let req = exchange::OrderRequest {
        book: form.book,
        quantity,
        price,
        is_buy: form.is_buy,
        tif: match form.order_type {
            OrderType::Market => exchange::TimeInForce::GTC,
            OrderType::GTC => exchange::TimeInForce::GTC,
            OrderType::IOC => exchange::TimeInForce::IOC,
            OrderType::POST => exchange::TimeInForce::POST,
        },
    };

    let (req, recv) = MatcherRequest::submit(user.id, req);
    state.cmd_send.send(req).await.unwrap();
    let response = recv.await.expect("Sender dropped");

    info!("User {} posted an order {:?}", user.id, response);

    let message = match response {
        Ok(event) => format!("success: {event:?}"),
        Err(err) => format!("error: {err:?}"),
    };
    OrderForm::with_messages(
        book,
        form.quantity,
        form.price,
        String::new(),
        String::new(),
        message,
    )
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
