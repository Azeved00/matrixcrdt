from invoke import Task, task, Collection
import os
import shutil

from scripts.micro.benchmarks import run_benchmark
from tasks.utils import notify_on_finish
from .graph import box_single, line_single



@task 
@notify_on_finish("Benchmark", "Finished running the benchmark")
def benchmark(c, sample=2000, apply_n=5, output_dir="plots/micro",
                benchmark="apply",  graph_type="box",
                logs_dir="logs/micro/temp", label="temp",
                show=True, group_percentage=0.05, repetitions=5):
    """Default function to run benchmarks"""

    os.makedirs(logs_dir, exist_ok=True)
    temp_dir="temp12345"
    os.makedirs(temp_dir, exist_ok=True)
    
    print("Starting Benchmark")
    for i in range(repetitions):
        match benchmark:
            case "authdag":
                run_benchmark(c, sample=sample, apply_n=apply_n, 
                              frontend_env="", backend_bin="socket",
                              logs_dir=temp_dir)

            case "authless":
                run_benchmark(c, sample=sample, apply_n=apply_n,
                              frontend_env="", backend_bin="baseline",
                              logs_dir=temp_dir)

            case "stateless":
                run_benchmark(c, sample=sample, apply_n=apply_n, 
                              frontend_env="stateless", backend_bin="socket",
                              logs_dir=temp_dir)

        files = [f for f in os.listdir(temp_dir) if os.path.isfile(os.path.join(temp_dir, f))]

        for filename in files:
            name, ext = os.path.splitext(filename)
            new_name = f"{name}-{i}{ext}"

            src_path = os.path.join(temp_dir, filename)
            dest_path = os.path.join(logs_dir, new_name)

            shutil.move(src_path, dest_path)

    print("Printing Graphs")
    match graph_type:
        case "line":
            line_single(c, folder=logs_dir, output_dir=output_dir,
                        label=label,
                       show=show,sample_n=apply_n )
        case "box":
            box_single(c, folder=logs_dir, output_dir=output_dir, label=label,
                       show=show, group_percentage=group_percentage, sample_n=apply_n)

    shutil.rmtree(temp_dir)



def make_benchmark_task(benchmark_name):
    @task
    @notify_on_finish("Benchmark", "Finished running the benchmark")
    def benchmark_task(c, sample=2000, apply_n=5, graph_type="box", repetitions=5, show=False):
        return benchmark(c, sample=sample, apply_n=apply_n,
            graph_type=graph_type, benchmark=benchmark_name,
            show=show, repetitions=repetitions, group_percentage=0.05, 
            output_dir=f"plots/micro/",
            label=f"{benchmark_name}-{graph_type}-{repetitions}x{sample}x{apply_n}",
            logs_dir=f"logs/micro/{benchmark_name}-{repetitions}x{sample}x{apply_n}"
        )

    benchmark_task.__doc__ = f"Run benchmark {benchmark_name}"
    return benchmark_task


ns_bench = Collection()
ns_bench.add_task(make_benchmark_task("authdag"), name="authdag")
ns_bench.add_task(make_benchmark_task("authless"), name="authless")
ns_bench.add_task(make_benchmark_task("stateless"), name="stateless")
ns_bench.add_task(benchmark, name="benchmark", default=True)

ns= Collection()
ns.add_collection(ns_bench, name="benchmark")
ns.add_collection(Collection.from_module(graph), name="graph")




