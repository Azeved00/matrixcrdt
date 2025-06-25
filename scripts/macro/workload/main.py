import sys
import threading
import random
import logging
import argparse
import numpy as np
import time


from .generator import gen_workload
from .initial_state import gen_initial_state, get_initial_state

logging.basicConfig(level=logging.INFO, format='[%(asctime)s] Client %(client)d - %(message)s', datefmt='%H:%M:%S')

def log_step(client_id, msg):
    logging.info(msg, extra={'client': client_id})


def gen_client_workload(id, time_limit, rng):
    log_step(id, "starting workload")
    gen_workload(id, time_limit, rng=rng)

def run_workload(clients, operations, seed=None):
    if seed is None:
        seed = random.SystemRandom().randint(0, 2**32 - 1)
        print(f"Generated random seed: {seed}")

    rng = np.random.default_rng(seed)

    threads = []

    log_step(0, "Generating initial state")
    gen_initial_state(0)
    log_step(0, "Finished generating initial state")
    time.sleep(1)

    for i in range(1, clients):
        log_step(i, "Querying initial state")
        get_initial_state(i)
        log_step(i, "Finished querying initial state")

    for i in range(clients):
        thread = threading.Thread(target=gen_client_workload, args=(i, operations, rng))
        thread.start()
        threads.append(thread)

    for thread in threads:
        thread.join()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Simulate client workload")
    parser.add_argument("clients", type=int, help="Number of clients")
    parser.add_argument("operations", type=int, help="Number of operations per client")
    parser.add_argument("--seed", "-s", type=int, help="Optional random seed")

    args = parser.parse_args()

    run_workload(
            clients=args.clients,
            operations=args.operations,
            seed=args.seed)

