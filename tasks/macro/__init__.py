from invoke import task, Collection

from scripts.macro.workload import run_workload
from scripts.macro.graph import graph_single
from . import workload, graph
from .benchmark import run_benchmark

@task()
def authdag(c, clients=2, operations=10000):
    log_dir="./logs/macro/authdag/"

    print("🔧 Running benchmark...")
    run_benchmark(
        c,
        clients=clients,
        operations=operations,
        logs_dir=log_dir,

        backend_bin="socket",
    )

    print("📈 Graphing results...")
    graph_single(
        name="authdag",
        input_path=log_dir,
        strategy="mean",
        display=False,
        warmup=0
    )

@task()
def stateless(c, clients=2, operations=100):
    log_dir="./logs/macro/stateless"

    print("🔧 Running benchmark...")
    run_benchmark(
        c,
        clients=clients,
        operations=operations,
        logs_dir=log_dir,

        backend_bin="socket",
        frontend_env="stateless"
    )

    print("📈 Graphing results...")
    graph_single(
        name="stateless",
        input_path=log_dir,
        strategy="mean",
        display=False,
        warmup=0
    )

@task()
def authless(c, clients=2, operations=10000):
    log_dir="./logs/macro/authless"

    print("🔧 Running benchmark...")
    run_benchmark(
        c,
        clients=clients,
        operations=operations,
        logs_dir=log_dir,

        backend_bin="baseline",
    )

    print("📈 Graphing results...")
    graph_single(
        name="authless",
        input_path=log_dir,
        strategy="mean",
        display=False,
        warmup=0
    )


@task()
def matrix(c, clients=2, operations=5000):
    log_dir="./logs/macro/matrix"

    print("🔧 Running benchmark...")
    run_benchmark(
        c,
        clients=clients,
        operations=operations,
        logs_dir=log_dir,

        backend_bin="matrix",
    )

    print("📈 Graphing results...")
    graph_single(
        name="matrix",
        input_path=log_dir,
        strategy="mean",
        display=False,
        warmup=0
    )


# Create collection and add submodules and tasks
ns = Collection()
ns.add_task(authdag)
ns.add_task(stateless)
ns.add_task(authless)
ns.add_task(matrix)

ns.add_task(workload.workload)
ns.add_task(run_benchmark, name="benchmark")
ns.add_collection(Collection.from_module(graph), name="graph")

