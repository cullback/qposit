use lobster::{Balance, MarketId, MatcherResult, OrderRequest, UserId};
use lobster::{OrderId, Price};
use tokio::sync::oneshot;

/// A request to the database engine.
/// These messages are used to communicate from the endpoints to the matching engine service.
#[derive(Debug)]
pub enum MatcherRequest {
    SubmitOrder {
        user: UserId,
        order: OrderRequest,
        /// Response to the client
        response: oneshot::Sender<MatcherResult>,
    },
    CancelOrder {
        user: UserId,
        order: OrderId,
        /// Response to the client
        response: oneshot::Sender<MatcherResult>,
    },
    AddEvent {
        market_id: MarketId,
    },
    Deposit {
        user: UserId,
        amount: Balance,
    },
    Resolve {
        market_id: MarketId,
        price: Price,
        response: oneshot::Sender<MatcherResult>,
    },
}

impl MatcherRequest {
    pub fn submit(user: UserId, order: OrderRequest) -> (Self, oneshot::Receiver<MatcherResult>) {
        let (response, recv) = oneshot::channel();
        let req = Self::SubmitOrder {
            user,
            order,
            response,
        };
        (req, recv)
    }

    pub fn cancel(user: UserId, order: OrderId) -> (Self, oneshot::Receiver<MatcherResult>) {
        let (response, recv) = oneshot::channel();
        let req = Self::CancelOrder {
            user,
            order,
            response,
        };
        (req, recv)
    }

    pub fn deposit(user: UserId, amount: Balance) -> Self {
        let req = Self::Deposit { user, amount };
        req
    }

    pub fn resolve(market_id: MarketId, price: Price) -> (Self, oneshot::Receiver<MatcherResult>) {
        let (response, recv) = oneshot::channel();
        let req = Self::Resolve {
            market_id,
            price,
            response,
        };
        (req, recv)
    }
}
