import requests
import time
from sys import argv
pi_ipv4 = argv[1]
service_address = f"http://{pi_ipv4}:1986"
while True:
    time.sleep(1)
    print(requests.get(service_address).text)
