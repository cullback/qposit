import requests

URL = "http://localhost:3000/api/v1"
AUTH = requests.auth.HTTPBasicAuth("testaccount", "password123")

def post_market(market: dict[str, str|int]):
    market_slug = market.pop("slug")
    print(f"Posting market: {market_slug}")
    response = requests.post(f"{URL}/markets/{market_slug}", json=market, auth=AUTH)
    print(response.status_code, response.text)

def query_books():
    response = requests.get(f"{URL}/books")
    print(response.status_code, response.text)



def main():
    
    market = {
        "slug": "eglinton-crosstown-open-to-public",
        "title": "Eglinton Crosstown Open to Public",
        "description": "not in totality",
        "created_at": 1697838953000000,
        "expires_at": 1730764800000000,
        "books": ["2024 Q2", "2024 Q3", "2024 Q4", "2025 Q1"],
    }
    post_market(market)


if __name__ == "__main__":
    main()