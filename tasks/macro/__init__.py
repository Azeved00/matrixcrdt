from invoke import task, Collection

from scripts.macro.workload import run_workload
from scripts.macro.graph import graph_single
from . import workload, graph
from .benchmark import run_benchmark

@task()
def benchmark(c, name, clients, operations):
    log_dir=f"./logs/macro/{name}/"

    print("🔧 Running benchmark...")
    run_benchmark(
        c,
        clients=clients,
        operations=operations,
        logs_dir=log_dir,

        backend_bin=
                 "socket" if name != "authless" 
            else "baseline" if name == "authless" 
            else "matrix",
        frontend_env= "stateless" if name == "stateless" else "",
    )

    print("📈 Graphing results...")
    graph_single(
        name=name,
        input_path=log_dir,
        strategy="mean",
        display=False,
        warmup=0
    )

@task()
def sauthdag(c, clients=2, operations=10000):
    benchmark(c,"authdag", clients, operations)

@task()
def sstateless(c, clients=2, operations=100):
    benchmark(c,"authdag", clients, operations)

@task()
def sauthless(c, clients=2, operations=10000):
    benchmark(c,"authdag", clients, operations)

@task()
def smatrix(c, clients=2, operations=10000):
    benchmark(c,"authdag", clients, operations)
@task()
def sall(c):
    """Run all simple benchmarks in sequence."""
    sauthdag(c,)
    sstateless(c)
    sauthless(c)
    smatrix(c)

ns_simple = Collection()
ns_simple.add_task(sauthdag, name="authdag")
ns_simple.add_task(sstateless, name="stateless")
ns_simple.add_task(sauthless, name="authless")
ns_simple.add_task(smatrix, name="matrix")
ns_simple.add_task(sall, name="all", default=True)


@task()
def fauthdag(c, clients=32, operations=10000):
    benchmark(c,"authdag", clients, operations)

@task()
def fstateless(c, clients=32, operations=100):
    benchmark(c,"authdag", clients, operations)

@task()
def fauthless(c, clients=32, operations=10000):
    benchmark(c,"authdag", clients, operations)

@task()
def fmatrix(c, clients=32, operations=10000):
    benchmark(c,"authdag", clients, operations)

@task()
def fall(c):
    """Run all full benchmarks in sequence."""
    fauthdag(c)
    fstateless(c)
    fauthless(c)
    fmatrix(c)

ns_full = Collection()
ns_full.add_task(fauthdag, name="authdag")
ns_full.add_task(fstateless, name="stateless")
ns_full.add_task(fauthless, name="authless")
ns_full.add_task(fmatrix, name="matrix")
ns_full.add_task(fall, name="all", default=True)


ns = Collection()

ns.add_collection(ns_simple, name="simple")
ns.add_collection(ns_full, name="full")

ns.add_task(workload.workload)
ns.add_task(run_benchmark, name="benchmark")
ns.add_collection(Collection.from_module(graph), name="graph")

