from invoke import task
import subprocess
import time
import os
import shutil
import glob
import signal
import sys

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

def notify_user():
    try:
        if shutil.which("notify-send"):
            subprocess.run(["notify-send", "Benchmark Complete", "Your benchmark has finished."])
        elif shutil.which("osascript"):
            subprocess.run([
                "osascript", "-e",
                'display notification "Your benchmark has finished." with title "Benchmark Complete"'
            ])
        else:
            print("📢 Notification not supported on this OS.")
    except Exception as e:
        print(f"⚠️ Notification failed: {e}")

@task
def run_benchmark(c, backend_bin, frontend_env="",
                  sample=1000, apply_n=5,
                  logs_dir="./logs/default"):
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
        env = os.environ.copy()
        env["NODE_ENV"] = "bench"
        env["STATE_ENV"] = frontend_env
        proc = subprocess.Popen(
                ["npm", "--prefix", "./frontend", "run", "micro", str(3001)],
            env=env
        )
        procs.append(proc)
        wait_for_port(port)

        env = os.environ.copy()
        env["NODE_ENV"] = "bench"
        env["STATE_ENV"] = frontend_env
        proc = subprocess.Popen(
            ["npm", "--prefix", "./frontend", "run", "micro_stateless", str(3002)],
            env=env
        )
        procs.append(proc)
        wait_for_port(port)

        time.sleep(1)
        print("🚀 Starting Requests...")
        headers = {'Content-Type': 'application/json'}
        for _ in range(sample):
            for i in range(1, apply_n+ 1):
                requests.post(
                    "http://localhost:3000/map", headers=headers,
                    json={"key": "123", "value": i},
                )
                requests.post("http://localhost:3000/save",)

            requests.post("http://localhost:3001/query",)

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
        notify_user()

