from invoke import task
import subprocess
import time
import os
import shutil
import glob
import signal
import sys

from scripts.macro.workload import run_benchmark
from scripts.macro.graph import plot_graphs, merge_and_validate as merge, make_table
from . import graph, benchmark
from .benchmark import benchmark as run_benchmark

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


@task()
def benchmark(c, name, clients, operations, 
              workload_seed=76,
              benchmark_logs_dir=None,
              graph_strategy="mean", graph_warmup=0, graph_use_dag_ops=True,
              table_latex=True, graph_output_dir="./plots", graph_save=True,
              graph_include_dir=False):
    """Default function to run benchmarks"""

    if benchmark_logs_dir == None:
        benchmark_logs_dir=f"./logs/macro/{name}/"

    print("🔧 Running benchmark...")
    run_benchmark(
        clients=clients,
        operations=operations,
        logs_dir=benchmark_logs_dir,

        backend_bin=
                 "socket" if name != "authless" 
            else "baseline" if name == "authless" 
            else "matrix",
        frontend_env= "stateless" if name == "stateless" else "",
        seed=workload_seed,
    )

    print("📈 Graphing results...")
    df1 = merge(log_dir)
    plts = plot_graphs(df1, strategy=graph_strategy,warmup=graph_warmup, use_dag_ops=graph_use_dag_ops, include_front=graph_include_front)
    for op, plt in plts:
        if graph_save:
            plt.savefig(f'{graph_output_dir}/{args[0]}-{op_name}.png')
        else:
            plt.show()
        plt.close()
    print("Plots saved in 'plots' directory.")
    make_table(df1, include_front=graph_include_front, latex=table_latex)
    notify_user()
