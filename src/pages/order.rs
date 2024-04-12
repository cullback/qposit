use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension, Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tracing::info;

use crate::{actors::matcher_request::MatcherRequest, app_state::AppState, auth::SessionExtractor};

#[derive(Debug, Deserialize)]
pub enum OrderType {
    Market,
    GTC,
    IOC,
    POST,
}

#[derive(Debug, Deserialize)]
pub struct PostOrder {
    book: u32,
    quantity: u32,
    price: u16,
    is_buy: bool,
    order_type: OrderType,
}

impl From<PostOrder> for exchange::OrderRequest {
    fn from(req: PostOrder) -> Self {
        Self {
            book: req.book,
            quantity: req.quantity,
            price: req.price,
            is_buy: req.is_buy,
            tif: match req.order_type {
                OrderType::Market => exchange::TimeInForce::GTC,
                OrderType::GTC => exchange::TimeInForce::GTC,
                OrderType::IOC => exchange::TimeInForce::IOC,
                OrderType::POST => exchange::TimeInForce::POST,
            },
        }
    }
}

pub async fn post(
    State(state): State<AppState>,
    SessionExtractor(user): SessionExtractor,
    Extension(db): Extension<SqlitePool>,
    Form(form): Form<PostOrder>,
) -> impl IntoResponse {
    let Some(user) = user else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    info!("User {} is trying to post an order {:?}", user.id, form);

    let (req, recv) = MatcherRequest::submit(user.id, form.into());
    state.cmd_send.send(req).await.expect("Receiver dropped");
    let response = recv.await.expect("Sender dropped");

    info!("User {} posted an order {:?}", user.id, response);

    match response {
        Ok(event) => Html::from(format!("<p>success: {event:?}</p>")).into_response(),
        Err(err) => Html::from(format!("<p>error: {err:?}</p>")).into_response(),
    }
}
