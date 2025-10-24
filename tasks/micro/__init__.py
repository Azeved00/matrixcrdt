from invoke import Task, task, Collection
import os
import shutil

from scripts.micro.benchmarks import run_benchmark
from tasks.utils import notify_on_finish
from .graph import plot_single

@task 
@notify_on_finish("Benchmark", "Finished running the benchmark")
def benchmark(c, sample=1000, apply_n=5, stateful_apply_n=100, output_dir="plots/micro",
                benchmark="apply", strategy="box",
                logs_dir="logs/micro/temp", label="temp", color="lightblue",
                show=True, group_percentage=0.05, repetitions=5):
    """Default function to run benchmarks"""

    os.makedirs(logs_dir, exist_ok=True)
    temp_dir="temp12345"
    os.makedirs(temp_dir, exist_ok=True)
    
    print("Starting Benchmark")
    for i in range(repetitions):
        match benchmark:
            case "authdag":
                run_benchmark(c, sample=sample, apply_n=apply_n, stateful_apply_n=stateful_apply_n,
                              frontend_env="", backend_bin="socket",
                              logs_dir=temp_dir)

            case "authless":
                run_benchmark(c, sample=sample, apply_n=apply_n,
                              frontend_env="", backend_bin="baseline",
                              logs_dir=temp_dir)

            case "stateless":
                run_benchmark(c, sample=sample, apply_n=apply_n,  stateful_apply_n=stateful_apply_n,
                              frontend_env="stateless", backend_bin="socket",
                              logs_dir=temp_dir)

            case "baseline":
                run_benchmark(c, sample=sample, apply_n=apply_n, 
                              frontend_env="stateless", backend_bin="baseline",
                              logs_dir=temp_dir)

        files = [f for f in os.listdir(temp_dir) if os.path.isfile(os.path.join(temp_dir, f))]

        for filename in files:
            name, ext = os.path.splitext(filename)
            new_name = f"{name}-{i}{ext}"

            src_path = os.path.join(temp_dir, filename)
            dest_path = os.path.join(logs_dir, new_name)

            shutil.move(src_path, dest_path)

    print("Printing Graphs")
    plot_single(c, folder=logs_dir, output_dir=output_dir, label=label,
            show=show, group_percentage=group_percentage, sample_n=apply_n,
            color=color, strategy=strategy)

    shutil.rmtree(temp_dir)



def make_benchmark_task(benchmark_name, color):
    @task
    @notify_on_finish("Benchmark", "Finished running the benchmark")
    def benchmark_task(c, sample=2000, apply_n=5, strategy="box", 
                       stateful_apply_n= 10, repetitions=5, show=False):
        return benchmark(c, sample=sample, apply_n=apply_n,
            strategy=strategy, benchmark=benchmark_name,
            show=show, repetitions=repetitions, group_percentage=0.05, 
            stateful_apply_n=stateful_apply_n,
            output_dir=f"plots/micro/", color=color,
            label=f"{benchmark_name}-{strategy}-{repetitions}x{sample}x{apply_n}",
            logs_dir=f"logs/micro/{benchmark_name}-{repetitions}x{sample}x{apply_n}"
        )

    benchmark_task.__doc__ = f"Run benchmark {benchmark_name}"
    return benchmark_task


ns_bench = Collection()
ns_bench.add_task(make_benchmark_task("authdag", "yellow"), name="authdag")
ns_bench.add_task(make_benchmark_task("authless", "steelblue"), name="authless")
ns_bench.add_task(make_benchmark_task("stateless", "salmon"), name="stateless")
ns_bench.add_task(make_benchmark_task("baseline", "steelblue"), name="baseline")
ns_bench.add_task(benchmark, name="benchmark", default=True)

ns= Collection()
ns.add_collection(ns_bench, name="benchmark")
ns.add_collection(Collection.from_module(graph), name="graph")




