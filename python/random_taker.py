import requests
import time
import random

AUTH = ("demoaccount", "password123")

while True:
    is_buy = random.random() < 0.5
    data = {"book": 123, "quantity": 100, "is_buy": is_buy}
    resp = requests.post("http://localhost:3000/api/orders", json=data, auth=AUTH)
    print(resp.status_code, resp.text)
    time.sleep(2)
