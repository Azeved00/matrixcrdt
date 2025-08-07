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
