import sys
import threading
from generator import gen_workload
from initial_state import gen_initial_state, get_initial_state
import logging
import time
logging.basicConfig(level=logging.INFO, format='[%(asctime)s] Client %(client)d - %(message)s', datefmt='%H:%M:%S')

def log_step(client_id, msg):
    logging.info(msg, extra={'client': client_id})


def gen_client_workload(id, time_limit):
    log_step(id, "starting workload")
    gen_workload(id, time_limit)
    log_step(id, "done")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python script.py <clients> <operations>")
        exit(1)

    try:
        clients = int(sys.argv[1])
        op_num = int(sys.argv[2])
        threads = []

        log_step(0, "Generating initial state")
        gen_initial_state(0)
        log_step(0, "finished generating initial state")
        time.sleep(1)

        for i in range(1, clients):
            log_step(i, "Querying initial state")
            get_initial_state(i)
            log_step(i, "finished querying initial state")

        for i in range(0, clients):
            thread = threading.Thread(target=gen_client_workload,
                                      args=(i, op_num))
            thread.start()
            threads.append(thread)

        for thread in threads:
            thread.join()

    except ValueError:
        print("Both arguments must be integers.")
        exit(1)
