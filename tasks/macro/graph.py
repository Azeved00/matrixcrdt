from invoke import task
from invoke import Collection
import pandas as pd

from scripts.macro.graph import plot_graphs, plot_comparison, make_table, merge_and_validate as merge

@task
def single(ctx, name, input_path, strategy="box", 
            display=False, warmup=0, use_dag_ops=True,
            output_path="plots/", include_front=False):
    """Generate a single graph."""

    display = str(display).lower() in ("1", "true", "yes", "on")
    warmup = int(warmup)

    df1 = merge(input_path)
    plots = plot_graphs(df1, strategy=strategy, warmup=warmup, 
                include_front=include_front, use_dag_ops=use_dag_ops)

    for op, plt in plts:
        if display:
            plt.show()
        else:
            plt.savefig(f'{output_path}/{name}-{op_name}.png')
        plt.close()
    print("Plots saved in 'plots' directory (or shown).")


@task
def comparison(ctx, name1, input_path1, name2, input_path2, 
                strategy="box", include_front=False, warmup=0, use_dag_ops=True,
                output_path="plots/", display=False):
    """Compare two workloads via graphs."""
    display = str(display).lower() in ("1", "true", "yes", "on")
    warmup = int(warmup)

    df1 = merge(input_path1)
    df2 = merge(input_path2)

    path=f"{name1}x{name2}"
    plot_comparison(df1, name1, df2, name2, use_dag_ops=use_dag_ops,
                    include_front=include_front)

    for op, plt in plts:
        if display:
            plt.show()
        else:
            plt.savefig(f'{output_path}/{name1}x{name2}-{op_name}.png')
        plt.close()

    print("Plots saved in 'plots' directory (or shown).")

@task
def table(ctx, input_path, include_front=False, latex=False, parts=4):
    """Print a table summary of the csv at `input_path`."""
    df = pd.read_csv(input_path)
    make_table(df, include_front=include_front, latex=latex, parts=parts)

ns = Collection("graph")
ns.add_task(single)
ns.add_task(comparison)
ns.add_task(table)
