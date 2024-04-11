use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Form};
use serde::Deserialize;
use sqlx::SqlitePool;
use tracing::info;

use crate::{actors::matcher_request::MatcherRequest, app_state::AppState, auth::SessionExtractor};

#[derive(Debug, Deserialize)]
pub struct PostOrder {
    book: u32,
    quantity: u32,
    price: u16,
    is_buy: bool,
}

// impl From<PostOrder> for exchange::OrderRequest {
//     fn from(req: PostOrder) -> Self {
//         Self {
//             book: req.book,
//             quantity: req.quantity,
//             price: req.price,
//             is_buy: req.is_buy,
//             tif: match req.tif {
//                 TimeInForce::GTC => exchange::TimeInForce::GTC,
//                 TimeInForce::IOC => exchange::TimeInForce::IOC,
//                 TimeInForce::POST => exchange::TimeInForce::POST,
//             },
//         }
//     }
// }

pub async fn post(
    State(state): State<AppState>,
    SessionExtractor(user): SessionExtractor,
    Extension(db): Extension<SqlitePool>,
    Form(form): Form<PostOrder>,
) -> impl IntoResponse {
    let Some(user) = user else {
        return StatusCode::UNAUTHORIZED;
    };
    info!("User {} is trying to post an order {:?}", user.id, form);

    // let (req, recv) = MatcherRequest::submit(user.id, order.into());
    // state.cmd_send.send(req).await.expect("Receiver dropped");
    // let response = recv.await.expect("Sender dropped");

    // match response {
    //     Ok(event) => Json(BookEvent::from(event)).into_response(),
    //     Err(err) => Json(json!({"error": format!("{err:?}")})).into_response(),
    // }

    StatusCode::OK
}
