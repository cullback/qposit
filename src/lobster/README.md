# Matcher

A matching engine for a prediction market.

## Features

- GTC, IOC, and POST order types
- Market orders emulated using IOCs with min/max price
- position and balance tracking
- order rejection on insufficient funds
- self-matching prevention using reduce oldest
- resolve book to certain price
- entirely deterministic, easy to simulate


## Interface

```rust
submit_order(UserId, OrderRequest) -> Result<Event, RejectReason>
cancel_order(UserId, OrderId) -> Result<Event, RejectReason>
resolve_book(BookId, Price) -> Result<Event, RejectReason>
add_book(BookId, tick_size: Price)
deposit(UserId, i64)
withdraw(UserId, i64)
```

## Notes

### IOC orders

- if an IOC is not marketable, it is not broadcasted over the feed.
- if it is marketable, it will be published with the size that was executed
- this saves some bandwidth since we don't have to send a cancel message

### Position

- Sign of position represents long and short respectively
