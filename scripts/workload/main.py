import sys
import threading
from generator import gen_workload
from initial_state import gen_initial_state


def client_thread(id, time):
    set_initial_state(id)
    gen_workload(id, time)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python script.py <clients> <operations>")
        exit(1)

    try:
        clients = int(sys.argv[1])
        time = int(sys.argv[2])
        threads = []

        for i in range(0, clients):
            thread = threading.Thread(target=client_thread,
                                      args=(i, time))
            thread.start()
            threads.add(thread)

        for thread in threads:
            thread.join()

    except ValueError:
        print("Both arguments must be integers.")
        exit(1)
