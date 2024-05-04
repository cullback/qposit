use super::templates::market_page::MarketPage;
use super::templates::order_form::OrderForm;
use crate::app_state::AppState;
use crate::models;
use crate::models::market::Market;
use crate::{auth::SessionExtractor, models::book::Book};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;
use exchange::BookId;
use orderbook::{Price, Quantity};

type PriceLevel = (Price, Quantity, Quantity);

fn price_level_to_string(pl: PriceLevel) -> (String, String, String) {
    let (price, quantity, value) = pl;

    let price_str = format!("{:.2}", f32::from(price) / 100.0);
    let quantity_str = quantity.to_string();
    let value_str = format!("{:.2}", f64::from(value) / 10000.0);

    (price_str, quantity_str, value_str)
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
    pub order_form: OrderForm,
}

impl OrderBook {
    pub fn from_orders(book_id: BookId, orders: Vec<models::order::Order>) -> Self {
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
            order_form: OrderForm::new(book_id),
        }
    }
}

pub async fn get(
    SessionExtractor(user): SessionExtractor,
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Ok(market) = Market::get_by_slug(&state.db, &slug).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Some(market) = market else {
        return Redirect::to("/404").into_response();
    };

    let books = Book::get_all_for_market(&state.db, market.id)
        .await
        .unwrap();
    let mut orderbooks = Vec::new();
    for book in &books {
        let orders = models::order::Order::get_open_for_book(&state.db, book.id)
            .await
            .unwrap();
        orderbooks.push(OrderBook::from_orders(book.id, orders));
    }

    match user {
        Some(user) => MarketPage::new(user.username, market, books, orderbooks).into_response(),
        None => MarketPage::new(String::new(), market, books, orderbooks).into_response(),
    }
}
