import tomllib
import pathlib
import requests

URL = "http://localhost:3000/api/v1"
AUTH = requests.auth.HTTPBasicAuth("testaccount", "password123")

def seed_markets():
    markets = tomllib.loads(pathlib.Path("python/markets.toml").read_text())
    for market in markets["markets"]:
        market_slug = market.pop("slug")
        print(f"Posting market: {market_slug}")
        response = requests.post(f"{URL}/markets/{market_slug}", json=market, auth=AUTH)
        print(response.status_code, response.text)


def main():
    seed_markets()

if __name__ == "__main__":
    main()