'''
The AMM strategy seeks to emulate the liquidity available in a concentrated liquidity AMM.
Note that max_collateral only bounds the amount of capital used as liquidity at any given
time. The only way to bound your total losses is to limit the amount of collateral in
your account.
'''
import requests
import time
import random

AUTH = ("testaccount", "password123")

p_min = 0
p_max = 0
spread = 0
delta = 0
depth = 0
max_collateral = 0

while True:
    is_buy = random.random() < 0.5
    data = {"book": 123, "quantity": 100, "is_buy": is_buy}
    resp = requests.post("http://localhost:3000/api/orders", json=data, auth=AUTH)
    print(resp.status_code, resp.text)
    time.sleep(2)
