import sys
import glob
import subprocess
import threading
import random
import logging
import argparse
import numpy as np
import time
import signal
import os
import shutil


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


def wait_for_port(port, delay=0.1, timeout=60):
    start = time.time()
    while True:
        result = subprocess.run(["netstat", "-an"], capture_output=True, text=True)
        if f"LISTEN" in result.stdout and str(port) in result.stdout:
            break
        if time.time() - start > timeout:
            raise TimeoutError(f"Port {port} did not open within {timeout} seconds")
        time.sleep(delay)

def move_logs(logs_dir, pattern, prefix):
    for file in glob.glob(pattern):
        name = os.path.splitext(os.path.basename(file))[0]
        dest = os.path.join(logs_dir, f"{prefix}_{name}.csv")
        shutil.move(file, dest)



def run_benchmark(backend_bin, clients=2, operations=10000, logs_dir="./logs/default", frontend_env="",  seed=None):
    ""
    procs = []
    try:
        print(f"🔧 Starting backend binary: {backend_bin}")
        backend = subprocess.Popen(
            ["cargo", "run", "--manifest-path", "./backend/Cargo.toml", 
             "--bin", backend_bin, "--features", "bench"]
        )
        procs.append(backend)
        wait_for_port(20076)

        print("🖥️ Starting frontends...")
        for i in range(1, int(clients) + 1):
            port = 3000 + i
            env = os.environ.copy()
            env["NODE_ENV"] = "bench"
            env["STATE_ENV"] = frontend_env
            proc = subprocess.Popen(
                ["npm", "--prefix", "./frontend", "run", "macro", str(port)],
                env=env
            )
            procs.append(proc)
            wait_for_port(port)

        time.sleep(1)
        print("🚀 Starting workload script...")
        run_workload(clients, operations, seed)


    except KeyboardInterrupt:
        print("⚠️ Interrupted by user (Ctrl+C)")
    except subprocess.CalledProcessError as e:
        print(f"❌ A subprocess failed: {e}")
    except Exception as e:
        print(f"❌ An unexpected error occurred: {e}")
    finally:
        print("🧹 Cleaning up processes...")
        for proc in procs:
            try:
                proc.send_signal(signal.SIGTERM)
            except Exception as e:
                print(f"⚠️ Failed to terminate process: {e}")

        print("📦 Collecting logs...")
        os.makedirs(logs_dir, exist_ok=True)
        move_logs(logs_dir, "./*.csv", "script")
        move_logs(logs_dir, "./frontend/src/*.csv", "front")
        move_logs(logs_dir, "./backend/*.csv", "back")

        print("✅ Benchmark complete.")

