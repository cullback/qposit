import tomllib
import pathlib
import requests

URL = "http://localhost:3000/api/v1"
AUTH = requests.auth.HTTPBasicAuth("admin", "berlopletrople")

def seed_markets():
    markets = tomllib.loads(pathlib.Path("scripts/markets.toml").read_text())
    for market in markets["markets"]:
        market_slug = market.pop("slug")
        print(f"Posting market: {market_slug}")
        response = requests.post(f"{URL}/markets/{market_slug}", json=market, auth=AUTH)
        print(response.status_code, response.text)


def main():
    seed_markets()

    # data = {"value": None}
    # response = requests.patch(f"{URL}/books/2", json=data, auth=AUTH)
    # print(response.status_code, response.text)

    # data = {"amount": 15000}
    # response = requests.post(f"{URL}/deposit/2", json=data, auth=AUTH)
    # print(response.status_code, response.text)

if __name__ == "__main__":
    main()