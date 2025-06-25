from invoke import task
from invoke import Collection

from scripts.macro.graph import graph_single, graph_comparison

@task
def single(ctx, name, input_path, strategy, display=False, warmup=0):
    """Generate a single graph."""

    display = str(display).lower() in ("1", "true", "yes", "on")
    warmup = int(warmup)

    graph_single(name, input_path, strategy, display, warmup)

@task
def comparison(ctx, name1, input_path1, name2, input_path2, 
                strategy, display=False, warmup=0):

    display = str(display).lower() in ("1", "true", "yes", "on")
    warmup = int(warmup)

    """Compare two workloads via graphs."""
    graph_comparisson(name1, input_path1, name2, input_path2, 
                      strategy, display, warmup)

ns = Collection("graph")
ns.add_task(single)
ns.add_task(comparison)
