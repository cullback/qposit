use super::templates::market_page::MarketPage;
use super::templates::orderbook::{OrderBook, PriceLevel};
use crate::app_state::AppState;
use crate::models;
use crate::models::market::Market;
use crate::models::order::Order;
use crate::{auth::SessionExtractor, models::book::Book};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;
use orderbook::{Price, Quantity};

/// Computes the price levels for an order book.
///
/// # Arguments
///
/// - `orders`: iterable of orders sorted from best price to worst price.
fn do_side<'a>(orders: impl IntoIterator<Item = &'a Order>) -> Vec<PriceLevel> {
    let mut price_levels: Vec<PriceLevel> = Vec::new();
    let mut current_price: Option<Price> = None;
    let mut level_quantity = 0;
    let mut cumulative_value = 0;

    for order in orders {
        if current_price == Some(order.price) {
            level_quantity += order.remaining;
            cumulative_value += order.remaining * Quantity::from(order.price);
        } else {
            if let Some(price) = current_price {
                price_levels.push(PriceLevel {
                    price: format!("{:.2}", f32::from(price) / 100.0),
                    quantity: level_quantity.to_string(),
                    value: format!("{:.2}", f64::from(cumulative_value) / 10000.0),
                });
            }
            level_quantity = order.remaining;
            cumulative_value += order.remaining * Quantity::from(order.price);
            current_price = Some(order.price);
        }
    }

    if let Some(price) = current_price {
        price_levels.push(PriceLevel {
            price: format!("{:.2}", f32::from(price) / 100.0),
            quantity: level_quantity.to_string(),
            value: format!("{:.2}", f64::from(cumulative_value) / 10000.0),
        });
    }

    price_levels
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

        let orderbook = OrderBook {
            book_id: book.id,
            bids: do_side(orders.iter().filter(|order| order.is_buy).rev()),
            asks: do_side(orders.iter().filter(|order| !order.is_buy)),
        };
        orderbooks.push(orderbook);
    }

    match user {
        Some(user) => MarketPage::new(user.username, market, books, orderbooks).into_response(),
        None => MarketPage::new(String::new(), market, books, orderbooks).into_response(),
    }
}
