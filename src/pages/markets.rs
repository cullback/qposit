use super::templates::market;
use crate::models;
use crate::models::market::Market;
use crate::{auth::SessionExtractor, models::book::Book};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::{
    response::{Html, IntoResponse},
    Extension,
};
use orderbook::{Price, Quantity};
use sqlx::SqlitePool;
use tracing::info;

type PriceLevel = (Price, Quantity, Quantity);

fn price_level_to_string(pl: PriceLevel) -> (String, String, String) {
    let (price, quantity, value) = pl;

    let mut price_str = price.to_string();
    price_str.insert(price_str.len() - 2, '.');

    let mut value_str = value.to_string();
    value_str.insert(value_str.len() - 4, '.');
    value_str.pop();
    value_str.pop();
    (price_str, quantity.to_string(), value_str)
}

fn build_price_levels(orders: impl Iterator<Item = models::order::Order>) -> Vec<PriceLevel> {
    let mut price_levels: Vec<PriceLevel> = Vec::new();
    let mut current_price: Option<Price> = None;
    let mut total_quantity = 0;
    let mut cumulative_value = 0;

    for order in orders {
        if current_price == Some(order.price) {
            total_quantity += order.remaining;
            cumulative_value += order.remaining * Quantity::from(order.price);
        } else {
            if let Some(price) = current_price {
                price_levels.push((price, total_quantity, cumulative_value));
            }
            total_quantity = order.remaining;
            cumulative_value += order.remaining * Quantity::from(order.price);
            current_price = Some(order.price);
        }
    }

    if let Some(price) = current_price {
        price_levels.push((price, total_quantity, cumulative_value));
    }
    price_levels
}

pub struct OrderBook {
    pub bids: Vec<(String, String, String)>,
    pub asks: Vec<(String, String, String)>,
}

impl OrderBook {
    pub fn from_orders(orders: Vec<models::order::Order>) -> Self {
        let (bids, asks): (Vec<_>, Vec<_>) = orders.into_iter().partition(|order| order.is_buy);

        Self {
            bids: build_price_levels(bids.into_iter().rev())
                .into_iter()
                .map(price_level_to_string)
                .collect(),
            asks: build_price_levels(asks.into_iter())
                .into_iter()
                .rev()
                .map(price_level_to_string)
                .collect(),
        }
    }
}

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    Path(slug): Path<String>,
    Extension(db): Extension<SqlitePool>,
) -> impl IntoResponse {
    let Ok(market) = Market::get_by_slug(&db, &slug).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Some(market) = market else {
        return Redirect::to("/404").into_response();
    };

    let books = Book::get_all_for_market(&db, market.id).await.unwrap();
    let mut orderbooks = Vec::new();
    for book in books.iter() {
        let orders = models::order::Order::get_open_for_book(&db, book.id)
            .await
            .unwrap();
        orderbooks.push(OrderBook::from_orders(orders));
    }

    match user {
        Some(user) => {
            Html(market::build(&user.username, market, books, orderbooks)).into_response()
        }
        None => Html(market::build("", market, books, orderbooks)).into_response(),
    }
}
