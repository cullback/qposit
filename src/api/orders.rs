use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use lobster::Action;
use lobster::OrderId;
use serde::Deserialize;
use serde_json::json;
use sqlx::QueryBuilder;
use tracing::error;
use utoipa::{IntoParams, ToSchema};

use crate::api::order_request::OrderRequest;
use crate::{
    services::matcher_request::MatcherRequest,
    app_state::AppState,
    models::{self, order::Order},
};

use super::{
    api_error::{ApiError, ApiJson},
    auth::{BasicAuthExtractor, OptionalBasicAuth},
    feed::BookUpdate,
};

const fn default_limit() -> u32 {
    100
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetOrderParams {
    pub market_id: Option<u32>,
    pub user_id: Option<u32>,
    pub before: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Get orders
///
/// Get orders according to query parameters.
#[utoipa::path(
    get,
    path = "/api/v1/orders",
    params(GetOrderParams),
    responses(
        (status = 200, description = "List of orders", body = [Order])
    ),
    security(
        (),
        ("basic_auth" = [])
    )
)]
pub async fn get(
    OptionalBasicAuth(user): OptionalBasicAuth,
    State(state): State<AppState>,
    Query(params): Query<GetOrderParams>,
) -> Response {
    let mut query = QueryBuilder::new("SELECT * from 'order' WHERE status = 'open'");

    if let Some(market_id) = params.market_id {
        query.push(" AND market_id = ");
        query.push_bind(market_id);
    }
    if let Some(user_id) = params.user_id {
        query.push(" AND user_id = ");
        query.push_bind(user_id);
    }
    if let Some(after) = params.before {
        query.push(" AND created_at < ");
        query.push_bind(after);
    }
    query.push(" ORDER BY created_at DESC LIMIT ");
    query.push_bind(params.limit);

    let orders = match query.build_query_as::<Order>().fetch_all(&state.pool).await {
        Ok(orders) => orders,
        Err(err) => {
            error!(?err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let resp = orders.into_iter().collect::<Vec<_>>();
    Json(resp).into_response()
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

/// Submit order
///
/// Submit an order to the matching engine.
#[utoipa::path(
    post,
    path = "/api/v1/orders",
    request_body = OrderRequest,
    responses(
        (status = 200, description = "Order successfully submitted", body = BookUpdate)
    ),
    security(
        ("basic_auth" = [])
    )
)]
pub async fn post(
    State(state): State<AppState>,
    BasicAuthExtractor(user): BasicAuthExtractor,
    ApiJson(order): ApiJson<OrderRequest>,
) -> Response {
    let (req, recv) = MatcherRequest::submit(user.id, order.into());
    state.cmd_send.send(req).await.expect("Receiver dropped");
    let response = recv
        .await
        .expect("Sender dropped")
        .map_err(|err| ApiError::MatcherRequest(err));

    match response {
        Ok(market) => Json(BookUpdate::from(market)).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Cancel all orders
///
/// Submit cancel request for all orders
#[utoipa::path(
    delete,
    path = "/api/v1/orders",
    responses(
        (status = 200, description = "Selected orders cancelled succesfully")
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    BasicAuthExtractor(user): BasicAuthExtractor,
) -> impl IntoResponse {
    let Ok(orders) = models::order::Order::get_for_user(&state.pool, user.id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let mut deleted = vec![];

    for order in orders {
        let (req, recv) = MatcherRequest::cancel(user.id, order.id);
        state.cmd_send.send(req).await.expect("Receiver dropped");
        let resp = recv.await.expect("Sender dropped");
        if let Ok(lobster::MarketUpdate {
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

/// Cancel order
///
/// Submit cancel request for specified order.
#[utoipa::path(
    delete,
    path = "/api/v1/orders/:id",
    params(
        ("id" = i64, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Deleted all orders")
    ),
    security(
        ("basic_auth" = [])
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

    if let Ok(lobster::MarketUpdate {
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
