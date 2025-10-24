from invoke import task, Collection

from .benchmark import benchmark as run_benchmark
from tasks.utils import notify_on_finish

def make_benchmark_task(benchmark_name, benchmark_type, clients, operations):
    @task
    @notify_on_finish("Benchmark Finished",f"Macro Benchmark {benchmark_name} just finished.")
    def benchmark_task(c,
            name=benchmark_name, clients=clients, operations=operations, 
            workload_seed=None,
            graph_strategy="all", graph_remove_outliers=0.5, graph_use_dag_ops=True,
            table_latex=True, graph_show=False,
            repetitions=5, graph_include_front=False):
        return run_benchmark(c,
            name=name, clients=clients, operations=operations, 
            workload_seed=workload_seed, 
            benchmark_logs_dir=f"logs/macro/{benchmark_name}-r{repetitions}xc{clients}xo{operations}",
            graph_strategy=graph_strategy, graph_remove_outliers=graph_remove_outliers,
            graph_use_dag_ops=graph_use_dag_ops, repetitions=repetitions,
            table_latex=table_latex, 
            graph_output_dir=f"plots/macro/{benchmark_name}-{graph_strategy}-r{repetitions}xc{clients}xo{operations}",
            graph_show=graph_show,graph_include_front=graph_include_front
        )
    benchmark_task.__doc__ = f"Run {benchmark_type} {benchmark_name} benchmark"
    return benchmark_task


from . import graph, benchmark

ns_simple = Collection()
ns_simple.add_task(make_benchmark_task("authdag", "simple", 2, 10000), name="authdag")
ns_simple.add_task(make_benchmark_task("stateless", "simple", 2, 10000), name="stateless")
ns_simple.add_task(make_benchmark_task("baseline", "simple", 2, 10000), name="baseline")
ns_simple.add_task(make_benchmark_task("authless", "simple", 2, 10000), name="authless")
ns_simple.add_task(make_benchmark_task("matrix", "simple", 2, 10000), name="matrix")

ns_full = Collection()
ns_full.add_task(make_benchmark_task("authdag", "full", 32, 10000), name="authdag")
ns_full.add_task(make_benchmark_task("stateless", "full", 32, 100), name="stateless")
ns_full.add_task(make_benchmark_task("authless", "full",32, 10000), name="authless")
ns_full.add_task(make_benchmark_task("matrix", "full", 32, 10000), name="matrix")

ns = Collection()

ns.add_collection(ns_simple, name="simple")
ns.add_collection(ns_full, name="full")

ns.add_collection(Collection.from_module(benchmark), name="benchmark")
ns.add_collection(Collection.from_module(graph), name="graph")

