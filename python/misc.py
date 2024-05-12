import order_book
import requests
import json

USERNAME = "alice"
PASSWORD = "password"
AUTH = requests.auth.HTTPBasicAuth("alice", "password")
MARKET_ID = 1

def main():
    from websockets.sync.client import connect

    book = order_book.Book(1, MARKET_ID)

    URI = f"ws://{USERNAME}:{PASSWORD}@localhost:3000/ws"
    URI = f"ws://localhost:3000/ws"
    with connect(URI) as websocket:
        print("listening...")
        while True:
            message = websocket.recv()
            event = json.loads(message)
            book.update(event)
            print(book)




if __name__ == "__main__":
    main()


import json
from typing import Any, Iterator


def parse_mid_point(message: dict[str, Any]) -> float:
    bid = float(message["b"])
    ask = float(message["a"])
    return (bid + ask) / 2


async def get_message(url: str) -> Iterator[dict[str, Any]]:
    async with connect(url) as websocket:
        while True:
            message = await websocket.recv()
            data = json.loads(message)
            yield data


async def book_update(url: str) -> Iterator[dict[str, Any]]:
    async with connect("ws://localhost:3000/ws") as websocket:
        while True:
            message = await websocket.recv()
            data = json.loads(message)
            yield data


async def main():
    # url = "wss://stream.binance.com:9443/ws/btcusdt@trade"
    # url = "wss://stream.binance.com:9443/ws/!miniTicker@arr"
    # url = "wss://stream.binance.com:9443/ws/btcusdt@aggTrade"
    url = "wss://stream.binance.us:9443/ws/btcusdt@ticker"

    while True:
        done, pending = asyncio.wait(
            [get_message(url), book_update(url)], return_when=asyncio.FIRST_COMPLETED
        )

        if done.get("action") == "add":
            print("yay")

    # with connect(url) as websocket:
    #     print("Connected, waiting for message...")
    #     while True:
    #         message = websocket.recv()
    #         message = json.loads(message)

    #         timestamp = message["E"]
    #         mid = parse_mid_point(message)
    #         print(f"t={timestamp}, mid={mid}")


if __name__ == "__main__":
    asyncio.run(main())
