// use askama::Template;
// use orderbook::{Price, Quantity};

// #[derive(Template)]
// #[template(path = "orderbook.html")]
// pub struct OrderBook<'a> {
//     bids: &'a [(Price, Quantity, Quantity)],
//     asks: &'a [(Price, Quantity, Quantity)],
// }

// impl<'a> OrderBook<'a> {
//     pub async fn build(
//         bids: &'a [(Price, Quantity, Quantity)],
//         asks: &'a [(Price, Quantity, Quantity)],
//     ) -> Self {
//         OrderBook { bids, asks }
//     }
// }
