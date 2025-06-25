import requests
import sys
from .stats import * 


def gen_initial_state(client_id):
    path = calc_server_addr(client_id) + OPERATIONS["create_prescription"]["path"]
    for i in range(0, INITIAL_STATE_SIZE):
        data = {
            "patient": 1,
            "doctor": 1,
            "pharmacy":1,
            "id": i + 99_000_000,
        }
        response = requests.post(path, json=data)

def get_initial_state(client_id):
    path = calc_server_addr(client_id) + OPERATIONS["get_staff_prescriptions"]["path"]
    response = requests.get(path)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python script.py <clients>")
        exit(1)

    try:
        clients = int(sys.argv[1])

        print("Generating initial state")
        gen_initial_state(0)
        print("finished generating initial state")

        for i in range(1, clients):
            print("Querying initial state", i)
            get_initial_state(i)
            print("finished querying initial state", i)

    except ValueError:
        print("Both arguments must be integers.")
        exit(1)
