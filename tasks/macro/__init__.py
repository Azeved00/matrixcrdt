from invoke import task, Collection

from .benchmark import benchmark as run_benchmark
def make_benchmark_task(benchmark_name, benchmark_type, clients, operations):
    @task()
    def benchmark_task(c,
            name=benchmark_name, clients=clients, operations=operations, 
            workload_seed=None, benchmark_logs_dir=None,
            graph_strategy="mean", graph_warmup=0, graph_use_dag_ops=True,
            table_latex=True, graph_output_dir="./plots", graph_save=True,
            graph_include_dir=False):
        return benchmark(
            name=name, clients=clients, operations=operations, 
            workload_seed=workload_seed, benchmark_logs_dir=benchmark_logs_dir,
            graph_strategy=graph_strategy, graph_warmup=graph_warmup,
            graph_use_dag_ops=graph_use_dag_ops,
            table_latex=table_latex, graph_output_dir=graph_output_dir,
            graph_save=graph_save,graph_include_dir=graph_include_dir
        )
    benchmark_task.__doc__ = f"Run {benchmark_name} {benchmark_type} benchmark"
    return benchmark_task


from . import graph, benchmark

ns_simple = Collection()
ns_simple.add_task(make_benchmark_task("authdag", "simple", 2, 10000), name="authdag")
ns_simple.add_task(make_benchmark_task("stateless", "simple", 2, 1000), name="stateless")
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

