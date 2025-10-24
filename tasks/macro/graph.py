from invoke import task
from invoke import Collection
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import pandas as pd
import os


from scripts.macro.graph import plot_graphs, plot_comparison, make_table, make_box_table, merge_folders as merge, strategies

@task
def single(ctx, name, input_path, strategy="box", 
            color="steelblue", front_color="orange", 
            remove_outliers=0.0, num_groups=20,
            use_dag_ops=True,include_front=False,
            output_path="./plots", show=False):
    """Generate a single graph."""

    strategies_list = strategies if strategy == "all" else [strategy]

    df = merge(input_path)
    os.makedirs(output_path, exist_ok=True)
    df = df[df["front_id"] >= 0]

    all_figs = []
    for t in strategies_list:
        plots = plot_graphs(df,
                            strategy=t, 
                            color=color, 
                            front_color=front_color,
                            remove_outliers=remove_outliers, 
                            num_groups=num_groups,
                            include_front=include_front, 
                            use_dag_ops=use_dag_ops)

        for op, fig in plots.items():
            ax = fig.axes[0]

            legend_patches = [mpatches.Patch(color=color, label=name.capitalize())]
            if include_front:
                legend_patches = [mpatches.Patch(color=color, label="Backend Time")]
                legend_patches.append(mpatches.Patch(color=front_color, label="Frontend Time"))
            ax.legend(handles=legend_patches)

            fig.savefig(f"{output_path}/{name}-{t}-{op}.png")

            if show:
                all_figs.append(fig)
            else:
                plt.close(fig)

    if show and all_figs:
        plt.show()

    print(f"Plots saved in '{output_path}' (or shown).")


@task
def comparison(ctx,folders,
                strategy="box", include_front=False, remove_outliers=0.5,
               use_dag_ops=True, show=False,
                output_dir="plots/macro/"):
    """Compare two workloads via graphs."""
    show = str(show).lower() in ("1", "true", "yes", "on")

    folders = [f.strip() for f in folders.split(",") if f.strip()]
    labels = ["Authdag", "Baseline" , "Stateless", "Matrix"]
    colors = ["yellow", "steelblue", "salmon", "limegreen"]

    dfs = []
    for folder in folders:
        dfs.append(merge(folder))
        print(f"Loaded CSVs from '{folder}'")

    plts = plot_comparison(dfs, use_dag_ops=use_dag_ops, remove_outliers=remove_outliers,
                    labels=labels, colors=colors,
                    include_front=include_front)

    # Save and optionally show
    os.makedirs(output_dir, exist_ok=True)
    for op, fig in plts.items():
        fig.savefig(f"{output_dir}/Comparison-{op}.png")

    if show:
        plt.show()
    plt.close("all")

    print(f"All comparison plots saved in: {output_dir}")

@task
def value_table(ctx, input_path, include_front=False, latex=False, parts=4):
    """Print a table summary of the csv at `input_path`."""
    df = pd.read_csv(input_path)
    make_table(df, include_front=include_front, latex=latex, parts=parts)

@task
def box_table(ctx, input_path, boxes=20, rows=5, latex=False):
    """
    Print a box-based table summary of the csv at `input_path`.

    B = number of boxes (default=20)
    R = number of rows to report (default=5)
    """
    df = merge(input_path)
    print(df.head())
    make_box_table(df, B=boxes, R=rows, latex=latex)

ns = Collection("graph")
ns.add_task(single)
ns.add_task(comparison)
ns.add_task(value_table)
ns.add_task(box_table)
