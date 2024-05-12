import dataclasses

from typing import Any, Optional
from operator import attrgetter


@dataclasses.dataclass
class Order:
    timestamp: int
    user_id: int
    order_id: int
    size: int
    price: int


class Book:
    def __init__(self, user_id: int, book_id: int) -> None:
        self.user_id = user_id
        self.book_id = book_id
        # the next sequence number we expect to see
        self.next_seq: int = 0
        self.bids: list[Order] = []
        self.asks: list[Order] = []

    def update(self, event: dict[str, Any]) -> None:
        assert event["market"] == self.book_id
        assert event["sequence"] == self.next_seq
        self.next_seq += 1

        if event["type"] == "add":
            order = Order(
                event["timestamp"],
                event["user_id"],
                event["id"],
                event["size"],
                event["price"],
            )
            if event["side"] == "buy":
                self._add_bid(order)
            else:
                self._add_ask(order)

        elif event["type"] == "remove":
            self._remove(event["id"])
        else:
            raise ValueError(f"Unknown event type {event['type']}")

    def _remove(self, order_id: int) -> None:
        for i, order in enumerate(self.bids):
            if order.order_id == order_id:
                self.bids.pop(i)
                return
        for i, order in enumerate(self.asks):
            if order.order_id == order_id:
                self.asks.pop(i)
                return
        raise ValueError(f"Order {order_id} not found")

    def best_bid(self) -> Optional[int]:
        return self.bids[0].price if self.bids else None

    def best_ask(self) -> Optional[int]:
        return self.asks[0].price if self.asks else None

    def _add_bid(self, order: Order) -> None:
        self.bids.append(order)
        # stable sort should preserve FIFO ordering
        self.bids.sort(key=attrgetter("price"), reverse=True)
        self._match()

    def _add_ask(self, order: Order) -> None:
        self.asks.append(order)
        # stable sort should preserve FIFO ordering
        self.asks.sort(key=attrgetter("price"))
        self._match()

    def _match(self) -> None:
        while self.bids and self.asks and self.bids[0].price >= self.asks[0].price:
            print("match")
            bid, ask = self.bids[0], self.asks[0]
            match_size = min(bid.size, ask.size)
            bid.size -= match_size
            ask.size -= match_size
            if bid.size == 0:
                self.bids.pop(0)
            if ask.size == 0:
                self.asks.pop(0)

    def __str__(self) -> str:
        book = "bids | price| asks\n"

        levels = [0] * 100
        for bid in self.bids:
            levels[bid.price] += bid.size
        for ask in self.asks:
            levels[ask.price] += ask.size

        n_levels = 5

        if best_ask := self.best_ask():
            ask_prices = range(best_ask, best_ask + n_levels)
            for px in reversed(ask_prices):
                qty = levels[px]
                if qty != 0:
                    book += f"     | {px:4} | {qty:4}\n"
                else:
                    book += f"     | {px:4} |\n"

        book += "-----+------+-----\n"

        if best_bid := self.best_bid():
            bid_prices = range(best_bid, best_bid - n_levels, -1)
            for px in bid_prices:
                qty = levels[px]
                if qty != 0:
                    book += f"{qty:4} | {px:4} |\n"            
                else:
                    book += f"     | {px:4} |\n"

        return book


def test():
    book = Book(1, 1)
    book.update(
        {
            "type": "add",
            "book_id": 1,
            "sequence": 1,
            "timestamp": 1,
            "user_id": 1,
            "oid": 1,
            "size": 5,
            "price": 10,
            "side": "buy",
        }
    )

    book.update(
        {
            "type": "add",
            "book_id": 1,
            "sequence": 2,
            "timestamp": 1,
            "user_id": 1,
            "oid": 1,
            "size": 7,
            "price": 15,
            "side": "sell",
        }
    )
    print(book)


if __name__ == "__main__":
    test()
