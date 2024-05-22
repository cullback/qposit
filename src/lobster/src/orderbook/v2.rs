// use crate::{
//     event::{Action, Event},
//     math::{buyer_cost, seller_cost, trade_cost},
//     order_request::OrderRequest,
//     reject_reason::RejectReason,
//     Balance, BookId, OrderId, Price, Size, Tick, TimeInForce, Timestamp, UserId, MAX_PRICE,
// };

// pub struct Order {
//     user: UserId,
//     book: BookId,
//     size: Size,
//     price: Price,
//     is_buy: bool,
// }

// pub struct BookInfo {
//     expire_time: Timestamp,
//     next_tick: Tick,
//     bid: Price,
//     ask: Price,
//     levels: [VecDeque<OrderId>; MAX_PRICE as usize + 1],
// }

// #[derive(Default)]
// pub struct Engine {
//     balances: HashMap<UserId, Balance>,
//     positions: HashMap<(UserId, BookId), i32>,
//     /// Map of open orders.
//     orders: HashMap<OrderId, Order>,
//     books: HashMap<BookId, BookInfo>,
//     /// Globally unique order id.
//     next_order_id: OrderId,
// }

// impl Engine {
//     pub fn new(next_order_id: OrderId) -> Self {
//         Self {
//             next_order_id,
//             ..Default::default()
//         }
//     }

//     pub fn add_book(&mut self, book: BookId) {
//         let book = BookInfo {
//             expire_time: 0,
//             next_tick: 0,
//             bid: 0,
//             ask: MAX_PRICE,
//             levels: std::array::from_fn(|_| VecDeque::new()),
//         };
//     }

//     pub fn init_position(&mut self, user_id: UserId, book_id: BookId, position: i32) {
//         assert!(self
//             .positions
//             .insert((user_id, book_id), position)
//             .is_none());
//     }

//     /// Function for initializing the book. Should only be called on startup.
//     pub fn init_order(&mut self, id: OrderId, user_id: UserId, order: OrderRequest) {
//         let book = self
//             .books
//             .get_mut(&order.book)
//             .expect("expected book to exist");

//         self.orders.insert(
//             id,
//             Order {
//                 user: user_id,
//                 book: order.book,
//                 size: order.size,
//                 price: order.price,
//                 is_buy: order.is_buy,
//             },
//         );
//     }

//     pub fn deposit(&mut self, user: UserId, amount: Balance) {
//         let balance = self.balances.entry(user).or_default();
//         *balance += amount;
//     }

//     /// Returns true if the order is marketable.
//     /// i.e. if placing the order would produce at least one fill.
//     fn is_marketable(&self, order: OrderRequest) -> bool {
//         let book = self.books.get(&order.book).unwrap();
//         if order.is_buy {
//             if book.levels[usize::from(order.price)..usize::from(book.ask)]
//                 .iter()
//                 .flatten()
//                 .any(|oid| self.orders.get(oid).is_some())
//             {
//                 return true;
//             }
//         } else {
//             if book.levels[usize::from(book.bid)..usize::from(order.price)]
//                 .iter()
//                 .flatten()
//                 .any(|oid| self.orders.get(oid).is_some())
//             {
//                 return true;
//             }
//         }
//         false
//     }

//     fn pre_trade_check(&self, user: UserId, order: OrderRequest) -> Result<(), RejectReason> {
//         if order.price == 0 || order.price > MAX_PRICE {
//             return Err(RejectReason::InvalidPrice);
//         }
//         if order.size == 0 {
//             return Err(RejectReason::InvalidSize);
//         }
//         if !self.books.contains_key(&order.book) {
//             return Err(RejectReason::BookNotFound);
//         }

//         let balance = *self.balances.get(&user).unwrap_or(&0);
//         let position = *self.positions.get(&(user, order.book)).unwrap_or(&0);
//         let trade_cost = trade_cost(position, order.size, order.price, order.is_buy);
//         if balance < trade_cost {
//             return Err(RejectReason::InsufficientFunds);
//         }

//         if matches!(order.tif, TimeInForce::IOC) {
//             let marketable = self.is_marketable(order);
//             if order.tif == TimeInForce::IOC && !marketable
//                 || order.tif == TimeInForce::POST && marketable
//             {
//                 return Err(RejectReason::OrderNotMarketable);
//             }
//         }
//         Ok(())
//     }

//     fn match_with_asks(&mut self, taker: UserId, order: &mut OrderRequest) {
//         let mut balance = *self.balances.get(&taker).unwrap_or(&0);
//         let mut position = *self.positions.get(&(taker, order.book)).unwrap_or(&0);
//         let book = self.books.get_mut(&order.book).unwrap();

//         while book.ask <= order.price {
//             let level = &mut book.levels[usize::from(book.ask)];
//             let mut i = 0;
//             for &maker_id in level.iter() {
//                 match self.orders.get_mut(&maker_id) {
//                     Some(maker) => {
//                         let traded_size = maker.size.min(order.size);
//                         if taker != maker.user {
//                             let maker_pos =
//                                 self.positions.entry((maker.user, order.book)).or_default();
//                             let maker_bal = self.balances.entry(maker.user).or_default();

//                             balance -= buyer_cost(position, traded_size, book.ask);
//                             *maker_bal -= seller_cost(*maker_pos, traded_size, book.ask);
//                             position += traded_size as i32;
//                             *maker_pos -= traded_size as i32;
//                         }
//                         order.size -= traded_size;
//                         if traded_size == maker.size {
//                             self.orders.remove(&maker_id);
//                             i += 1;
//                         } else {
//                             maker.size -= traded_size;
//                             break;
//                         }
//                     }
//                     None => i += 1,
//                 }
//             }
//             level.drain(..i);

//             if order.size == 0 {
//                 break;
//             }
//             book.ask += 1;
//         }
//         if order.size != 0 {
//             book.bid = book.bid.max(order.price);
//         }
//         self.balances.insert(taker, balance);
//         self.positions.insert((taker, order.book), position);
//     }

//     fn match_with_bids(&mut self, taker: UserId, order: &mut OrderRequest) {
//         let mut balance = *self.balances.get(&taker).unwrap_or(&0);
//         let mut position = *self.positions.get(&(taker, order.book)).unwrap_or(&0);
//         let book = self.books.get_mut(&order.book).unwrap();

//         while order.price <= book.bid {
//             let level = &mut book.levels[usize::from(book.bid)];
//             let mut i = 0;
//             for &maker_id in level.iter() {
//                 match self.orders.get_mut(&maker_id) {
//                     Some(maker) => {
//                         let traded_size = maker.size.min(order.size);
//                         if taker != maker.user {
//                             let maker_pos =
//                                 self.positions.entry((maker.user, order.book)).or_default();
//                             let maker_bal = self.balances.entry(maker.user).or_default();

//                             *maker_bal -= buyer_cost(*maker_pos, traded_size, book.bid);
//                             balance -= seller_cost(position, traded_size, book.bid);
//                             *maker_pos += traded_size as i32;
//                             position -= traded_size as i32;
//                         }
//                         order.size -= traded_size;
//                         if traded_size == maker.size {
//                             self.orders.remove(&maker_id);
//                             i += 1;
//                         } else {
//                             maker.size -= traded_size;
//                             break;
//                         }
//                     }
//                     None => i += 1,
//                 }
//             }
//             level.drain(..i);

//             if order.size == 0 {
//                 break;
//             }
//             book.bid -= 1;
//         }
//         if order.size != 0 {
//             book.ask = book.ask.max(order.price);
//         }
//         self.balances.insert(taker, balance);
//         self.positions.insert((taker, order.book), position);
//     }

//     pub fn submit_order(
//         &mut self,
//         time: Timestamp,
//         user: UserId,
//         mut order: OrderRequest,
//     ) -> Result<Event, RejectReason> {
//         self.pre_trade_check(user, order)?;

//         let original_size = order.size;

//         if order.is_buy {
//             self.match_with_asks(user, &mut order);
//         } else {
//             self.match_with_bids(user, &mut order);
//         }

//         let id = self.next_order_id;
//         self.next_order_id += 1;

//         let book = self.books.get_mut(&order.book).unwrap();
//         if order.size != 0 && order.tif != TimeInForce::IOC {
//             book.levels[usize::from(order.price)].push_back(id);
//             let order_entry = Order {
//                 book: order.book,
//                 user,
//                 size: order.size,
//                 price: order.price,
//                 is_buy: order.is_buy,
//             };
//             self.orders.insert(id, order_entry);
//         }

//         let event = Event::add(
//             time,
//             book.next_tick,
//             order.book,
//             user,
//             id,
//             original_size,
//             order.price,
//             order.is_buy,
//         );

//         book.next_tick += 1;
//         Ok(event)
//     }

//     pub fn cancel_order(
//         &mut self,
//         time: Timestamp,
//         user: UserId,
//         order_id: OrderId,
//     ) -> Result<Event, RejectReason> {
//         if self
//             .orders
//             .get(&order_id)
//             .ok_or(RejectReason::OrderNotFound)?
//             .user
//             != user
//         {
//             return Err(RejectReason::OrderNotFound);
//         }
//         let order = self
//             .orders
//             .remove(&order_id)
//             .ok_or(RejectReason::OrderNotFound)?;
//         let book = self.books.get_mut(&order.book).unwrap();
//         let event = Event::remove(time, book.next_tick, order.book, order.user, order_id);
//         book.next_tick += 1;
//         Ok(event)
//     }
// }
