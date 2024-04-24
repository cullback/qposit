use exchange::{BookEvent, BookId, OrderRequest, RejectReason, UserId};
use orderbook::OrderId;
use tokio::sync::oneshot;

/// A request to the database engine.
/// These messages are used to communicate from the endpoints to the matching engine service.
#[derive(Debug)]
pub enum MatcherRequest {
    SubmitOrder {
        user: UserId,
        order: OrderRequest,
        /// Response to the client
        response: oneshot::Sender<Result<BookEvent, RejectReason>>,
    },
    CancelOrder {
        user: UserId,
        order: OrderId,
        /// Response to the client
        response: oneshot::Sender<Result<BookEvent, RejectReason>>,
    },
    AddBook {
        book_id: BookId,
    },
}

impl MatcherRequest {
    pub fn submit(
        user: UserId,
        order: OrderRequest,
    ) -> (Self, oneshot::Receiver<Result<BookEvent, RejectReason>>) {
        let (response, recv) = oneshot::channel();
        let req = Self::SubmitOrder {
            user,
            order,
            response,
        };
        (req, recv)
    }

    pub fn cancel(
        user: UserId,
        order: OrderId,
    ) -> (Self, oneshot::Receiver<Result<BookEvent, RejectReason>>) {
        let (response, recv) = oneshot::channel();
        let req = Self::CancelOrder {
            user,
            order,
            response,
        };
        (req, recv)
    }
}
