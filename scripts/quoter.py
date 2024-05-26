"""Simple fixed-price quoting strategy.

1. Compute desired orders.
2. Compare to open orders.
3. Cancel differences.
4. Submit new orders.
"""
import requests
import time
import typing

USER_ID = 1
USERNAME = "admin"
PASSWORD = "berlopletrople"
AUTH = (USERNAME, PASSWORD)
BOOKS = {
    3: 0.55, # donald trump
    # 4: 0.46, # joe biden
    # 9: 0.23, # tim scott
    # 10: 0.13, # elise stefanik
    # 11: 0.15, # doug bourgum
    # 13: 0.05, # J.D. Vance
    # 12: 0.05, # marco rubio
    # 19: 0.25, # new york rangers
}
URL = "http://localhost:3000/api/v1"
# URL = "http://qposit.com/api/v1"

def probability_to_price(probability: float) -> int:
    return round(probability * 10_000)


class Order(typing.NamedTuple):
    order_id: int
    quantity: int
    price: int
    is_buy: bool


def compute_desired_orders(price: int) -> list[Order]:
    """Compute ladder of prices."""
    N_LEVELS = 3  # number of levels
    DELTA = 200  # difference between price levels
    SPREAD = 250  # difference between bid and ask
    QUANTITY = 100
    QTY_DIFF = 20  # increase quantity as we go deeper into the book

    orders = []
    for i in range(N_LEVELS):
        qty = QUANTITY + i * QTY_DIFF

        ask_price = price + SPREAD + i * DELTA
        if ask_price > 0 and ask_price < 10_000:
            orders.append(
                Order(order_id=0, quantity=qty, price=ask_price, is_buy=False)
            )
        bid_price = price - SPREAD - i * DELTA
        if bid_price > 0 and bid_price < 10_000:
            orders.append(Order(order_id=0, quantity=qty, price=bid_price, is_buy=True))

    return orders


def compare_orders(
    desired: list[Order], open: list[Order]
) -> tuple[list[Order], list[Order]]:
    """Returns a list of orders to cancel and place."""

    # (price, side) -> quantity
    orders_to_place: dict[tuple[int, bool], int] = {}
    for order in desired:
        key = (order.price, order.is_buy)
        orders_to_place[key] = order.quantity

    orders_to_cancel = []
    for order in open:
        key = (order.price, order.is_buy)
        if key in orders_to_place and orders_to_place[key] == order.quantity:
            # the order exists and has the correct quantity so don't place it
            del orders_to_place[key]
        else:
            orders_to_cancel.append(order.order_id)

    orders_to_place = [
        Order(order_id=0, quantity=qty, price=price, is_buy=side)
        for (price, side), qty in orders_to_place.items()
    ]

    return orders_to_cancel, orders_to_place


def get_open_orders() -> list[Order]:
    params = {"user_id": USER_ID}
    resp = requests.get(f"{URL}/orders", params=params)
    print("open orders:", resp.status_code, resp.text)
    assert resp.status_code == 200

    orders = []
    for order in resp.json():
        orders.append(
            Order(
                order_id=order["id"],
                quantity=order["remaining"],
                price=order["price"],
                is_buy=order["is_buy"],
            )
        )

    return orders


def step_for_book(book_id: int) -> None:
    probability = BOOKS[book_id]
    price = probability_to_price(probability)
    desired_orders = compute_desired_orders(price)

    open_orders = get_open_orders()
    orders_to_cancel, orders_to_place = compare_orders(desired_orders, open_orders)

    for order_id in orders_to_cancel:
        resp = requests.delete(f"{URL}/orders/{order_id}", auth=AUTH)
        print("deleted order:", order_id, resp.status_code, resp.text)
        assert resp.status_code == 200

    for order in orders_to_place:
        data = {
            "book": book_id,
            "quantity": order.quantity,
            "price": order.price,
            "is_buy": order.is_buy,
        }
        print("placing order:", data)
        resp = requests.post(f"{URL}/orders", json=data, auth=AUTH)
        print("placed order:", resp.status_code, resp.json())
        assert resp.status_code == 200
        assert "error" not in resp.json()


def main():
    while True:
        print("stepping...")
        for book in BOOKS:
            step_for_book(book)
        time.sleep(10)


if __name__ == "__main__":
    main()
