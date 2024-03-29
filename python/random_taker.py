"""Fire a random market order for 100 contracts every 2 seconds."""
import requests
import time
import random

AUTH = ("user", "pass")

while True:
    data = {"book": 123, "size": 100, "is_buy": random.random() < 0.5}
    resp = requests.post("http://localhost:3000/orders", json=data, auth=AUTH)
    print(resp.status_code, resp.text)
    time.sleep(2)
