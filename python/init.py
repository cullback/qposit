import requests
import tomllib
import json
import datetime as dt
import pathlib

URL = "http://localhost:3000"
AUTH = requests.auth.HTTPBasicAuth("admin", "pass123")
BOB = requests.auth.HTTPBasicAuth("bob", "pass123")

def post_markets():
    print("####### Posting markets #######")
    text = pathlib.Path("markets.toml").read_text()
    data = tomllib.loads(text)
    for market in data["markets"]:
        market_slug = market.pop("slug")
        print(f"Posting market: {market_slug}")
        response = requests.post(f"{URL}/markets/{market_slug}", json=market, auth=AUTH)
        print(response.status_code, response.text)

def query_books():
    print("####### Querying books #######")
    response = requests.get(f"{URL}/books")
    print(response.status_code, response.text)
    '''
    [
    {
        "id": 1,
        "market_id": "2024-us-presidential-election",
        "name": "Donald Trump",
        "status": "active"
    },
    ]
    '''

def post_order():
    print("####### Posting order #######")
    data = {
        "book": 1,
        "size": 50,
        "price": 10,
        "is_buy": True,
    }
    response = requests.post(f"{URL}/orders", json=data, auth=BOB)
    print(response.status_code, response.text)

    resp = response.json()
    print(resp)
    order_id = resp["id"]
    return order_id

def main():
    post_markets()

    query_books()
    order_id = post_order()

    # print('-----')
    # response = requests.get(f"{URL}/orders", auth=BOB)
    # print(response.status_code, response.text)

    # requests.delete(f"{URL}/orders/{order_id}", auth=BOB)


    # trade against two people
    # query active orders, positions, balance, trades
    # resolve market




if __name__ == "__main__":
    main()
