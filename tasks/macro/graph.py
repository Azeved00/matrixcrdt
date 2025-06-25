from invoke import task
from invoke import Collection
import pandas as pd

from scripts.macro.graph import plot_graphs, plot_comparison, merge, make_table

@task
def single(ctx, name, input_path, strategy, display=False, warmup=0):
    """Generate a single graph."""

    display = str(display).lower() in ("1", "true", "yes", "on")
    warmup = int(warmup)

    df1 = merge(name, input_path, True)
    plot_graphs(df1, name,
            strategy=strategy, show=display, warmup=warmup)


@task
def comparison(ctx, name1, input_path1, name2, input_path2, 
                strategy, display=False, warmup=0):
    """Compare two workloads via graphs."""
    display = str(display).lower() in ("1", "true", "yes", "on")
    warmup = int(warmup)

    df1 = merge(name1, input_path1, True)
    df2 = merge(name2, input_path2, True)

    path=f"{name1}x{name2}"
    plot_comparison(df1, name1, df2, name2, path)

@task
def table(ctx, input_path, include_front=False, latex=False ):
    """Print The latex ."""
    df = pd.read_csv(input_path)
    make_table(df, include_front, latex)

ns = Collection("graph")
ns.add_task(single)
ns.add_task(comparison)
ns.add_task(table)
