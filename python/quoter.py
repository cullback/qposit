import json
import requests
from typing import Iterator
import time


USER_ID = 3
USERNAME = "strategy2"
PASSWORD = "password123"
AUTH = (USERNAME, PASSWORD)
BOOK = 2
URL = "http://localhost:3000/api/v1"
SPREAD = 250

def step():
    
    # cancel open orders
    params = {"user_id": USER_ID}
    resp = requests.get(f"{URL}/orders", params=params, auth=AUTH)
    print("open orders", resp.status_code, resp.text)

    for order in resp.json():
        params = {"id": order["id"]}
        resp = requests.delete(f"{URL}/orders", params=params, auth=AUTH)
        print("deleted order:", order, resp.status_code, resp.json())

    time.sleep(1.5)

    valuation = 5000

    # submit two-sided quote
    bid_price = valuation - SPREAD
    bid = {"book": BOOK, "quantity": 100, "price": bid_price, "is_buy": True}
    resp = requests.post(f"{URL}/orders", json=bid, auth=AUTH)
    print(resp.status_code, resp.text)

    ask_price = valuation + SPREAD
    ask = {"book": BOOK, "quantity": 100, "price": ask_price, "is_buy": False}
    resp = requests.post(f"{URL}/orders", json=ask, auth=AUTH)
    print(resp.status_code, resp.text)

def main():

    while True:
        print("stepping...")
        step()
        time.sleep(8)

if __name__ == "__main__":
    main()