from invoke import Task, task, Collection
import os
import shutil

from scripts.micro.benchmarks import run_benchmark
from tasks.utils import notify_on_finish
from .graph import box_single, line_single





@task 
@notify_on_finish
def benchmark(c, sample=2000, apply_n=5, output_dir="plots/micro",
                benchmark="apply", backend_bin="socket", graph_type="box",
                logs_dir="logs/micro/temp",
                save=True, group_percentage=0.05, repetitions=5):
    """Default function to run benchmarks"""

    os.makedirs(logs_dir, exist_ok=True)
    temp_dir="temp12345"
    os.makedirs(temp_dir, exist_ok=True)

    for i in range(repetitions):
        match benchmark:
            case "apply":
                run_benchmark(c, sample=sample, apply_n=apply_n, 
                            query=False, frontend_env="",
                            backend_bin=backend_bin, logs_dir=temp_dir)

            case "query":
                run_benchmark(c, sample=sample, apply_n=apply_n,
                            query=True, frontend_env="",
                            backend_bin=backend_bin, logs_dir=temp_dir)

            case "stateless_query":
                run_benchmark(c, sample=sample, apply_n=apply_n, 
                            query=True, frontend_env="stateless",
                            backend_bin=backend_bin, logs_dir=temp_dir)

        files = [f for f in os.listdir(temp_dir) if os.path.isfile(os.path.join(temp_dir, f))]

        for filename in files:
            name, ext = os.path.splitext(filename)
            new_name = f"{name}-{i}{ext}"

            src_path = os.path.join(temp_dir, filename)
            dest_path = os.path.join(logs_dir, new_name)

            shutil.move(src_path, dest_path)

    match graph_type:
        case "line":
            line_single(c, folder=logs_dir, output_dir=output_dir,
                       save=save,sample_n=apply_n )
        case "box":
            box_single(c, folder=logs_dir, output_dir=output_dir,
                       save=save, group_percentage=group_percentage, sample_n=apply_n)

    shutil.rmtree(temp_dir)



def make_benchmark_task(benchmark_name, backend_bin):

    @task
    def benchmark_task(c, sample=2000, apply_n=5, graph_type="box", repetitions=5, save=True):
        return benchmark(c, sample=sample, apply_n=apply_n,
            graph_type=graph_type, benchmark=benchmark_name, backend_bin=backend_bin,
            save=save, repetitions=repetitions, group_percentage=0.05, 
            output_dir=f"plots/micro/{benchmark_name}-{graph_type}-{repetitions}x{sample}x{apply_n}",
            logs_dir=f"logs/micro/{benchmark_name}-{graph_type}-{repetitions}x{sample}x{apply_n}"
        )

    benchmark_task.__doc__ = f"Run benchmark {benchmark_name} for {backend_bin} binary"
    return benchmark_task


ns_baseline = Collection()
ns_baseline.add_task(make_benchmark_task("apply", backend_bin="baseline"), name="apply")
ns_baseline.add_task(make_benchmark_task("query", backend_bin="baseline"), name="query")
ns_baseline.add_task(make_benchmark_task("stateless_query", backend_bin="baseline"), name="stateless-query")

ns_bm = Collection()
ns_bm.add_task(make_benchmark_task("apply", backend_bin="socket"), name="apply")
ns_bm.add_task(make_benchmark_task("query", backend_bin="socket"), name="query")
ns_bm.add_task(make_benchmark_task("stateless_query", backend_bin="socket"), name="stateless-query")


ns= Collection()
ns.add_collection(ns_baseline, name="baseline")
ns.add_collection(ns_bm, name="authdag")
ns.add_task(benchmark, name="benchmark")
ns.add_collection(Collection.from_module(graph), name="graph")




