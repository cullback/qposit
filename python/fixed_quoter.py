'''Fixed quoter.

Quotes at a fixed distance away from mid price.
'''
import requests
import sys
import time
import random

AUTH = ("strategy", "password123")

BOOK = 1
URL = "http://localhost:3000/api/v1"

while True:
    # get mid price
    resp = requests.get(URL + f"/trades", params={"book_id": BOOK})
    assert resp.status_code == 200

    trades = resp.json()
    if len(trades) == 0:
        price = 5000
    else:
        print(trades[0])
        price = trades[0]["price"]

    print(f"price {price}")

    bid_price = max(price - 200,  100)
    ask_price = min(price + 200, 9900)
    data = {"book": BOOK, "quantity": 100, "price": bid_price, "is_buy": True}
    resp = requests.post(URL + f"/orders", json=data, auth=AUTH)
    print(resp.status_code, resp.text)
    
    data = {"book": BOOK, "quantity": 100, "price": ask_price, "is_buy": False}
    resp = requests.post(URL + f"/orders", json=data, auth=AUTH)
    print(resp.status_code, resp.text)

    sys.exit()
