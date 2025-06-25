from invoke import task

from scripts.macro.workload import run_workload

@task
def workload(ctx,clients, operations, seed):
    """Run Workload scripts"""

    clients = int(clients)
    operations = int(operations)
    seed = int(seed)

    run_workload(clients, operations, seed)

