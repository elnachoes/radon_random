import requests
from datetime import datetime
import time
from sys import argv
import random
import json

class GeigerCounterState:
    def __init__(self, last_count : str, count : int, last_reading_cpm : int):
        # this is a hack to make python happy, it will NOT freaking parse a 9 microsecond digit timestamp
        # 2025-04-06T09:39:19.376539056Z
        self.last_count = datetime.strptime(last_count[:-4]+last_count[-1], '%Y-%m-%dT%H:%M:%S.%fZ')
        self.count = count
        self.last_reading_cpm = last_reading_cpm

ip = "127.0.0.1"
if "--ip" in argv:
    ip = argv[argv.index("--ip")+1]

sides = 20
if "--sides" in argv:
    sides = int(argv[argv.index("--sides")+1])

auto_roll = False
if "--watch" in argv:
    auto_roll = True

roll_period_s = 1

service_address = f"http://{ip}:1986"

request_body = requests.get(service_address).text

geiger_state = GeigerCounterState(**json.loads(request_body))
random.seed(geiger_state.last_count.timestamp())

def roll():
    atomic_dice_roll = int(round(random.random() * geiger_state.last_count.timestamp())) % sides
    print(f"atomic dice roll : {atomic_dice_roll}")

if auto_roll:
    while True:
        time.sleep(roll_period_s)
        roll()
else:
    roll()

