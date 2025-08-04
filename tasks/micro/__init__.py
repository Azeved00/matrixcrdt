from invoke import task, Collection

from .benchmark_apply import run_benchmark as run_bench_1
from .benchmark_query import run_benchmark as run_bench_2
from .benchmark_s_query import run_benchmark as run_bench_3
from . import graph

@task()
def run_baseline_apply(c, sample=1000, apply_n=5):
    run_bench_1(c, sample=sample, apply_n=apply_n, backend_bin="baseline")

@task()
def run_baseline_query(c, sample=1000, apply_n=5):
    run_bench_2(c, sample=sample, apply_n=apply_n, backend_bin="baseline")

@task()
def run_baseline_squery(c, sample=1000, apply_n=5):
    run_bench_3(c, sample=sample, apply_n=apply_n, backend_bin="baseline")


ns_baseline = Collection()
ns_baseline.add_task(run_baseline_apply, name="apply")
ns_baseline.add_task(run_baseline_query, name="query")
ns_baseline.add_task(run_baseline_squery, name="stateless_query")

@task()
def run_benchmark_apply(c, sample=1000, apply_n=5):
    run_bench_1(c, sample=sample, apply_n=apply_n, backend_bin="socket",
                logs_dir="logs/micro/apply")

@task()
def run_benchmark_query(c, sample=1000, apply_n=5):
    run_bench_2(c, sample=sample, apply_n=apply_n, backend_bin="socket")

@task()
def run_benchmark_squery(c, sample=1000, apply_n=5):
    run_bench_3(c, sample=sample, apply_n=apply_n, backend_bin="socket")


ns_bm= Collection()
ns_bm.add_task(run_benchmark_apply, name="apply")
ns_bm.add_task(run_benchmark_query, name="query")
ns_bm.add_task(run_benchmark_squery, name="stateless_query")


ns= Collection()
ns.add_collection(ns_baseline, name="baseline")
ns.add_collection(ns_bm, name="benchmark")
ns.add_collection(Collection.from_module(graph), name="graph")




