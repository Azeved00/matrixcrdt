from invoke import task
from invoke import Collection
import matplotlib.pyplot as plt
import pandas as pd
import os

from scripts.micro.graph.plot_graph import plot_single as plot, plot_multi
import scripts.micro.graph as graph_utils


@task
def plot_single(c, folder="logs/optimized", output_dir="plots",
            label="Optimized", color="blue", strategy="box",
            group_percentage=0.05, sample_n=5, remove_outliers=0.05,
            show=True):
    """ Plot a box plot."""
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for f in files if (df := graph_utils.load_csv(f)) is not None]

    print(f"{len(dfs)} dataframes loaded")
    combined_df = pd.concat(dfs, ignore_index=True)

    plts = plot(
        dataframe=combined_df,
        group_percentage=group_percentage,
        sample_n=sample_n,
        color=color,
        strategy=strategy,
        remove_outliers=remove_outliers,
    )

    for op, fig in plts.items():
        if show:
            fig.show()
        os.makedirs(output_dir, exist_ok=True)
        fig.savefig(f"{output_dir}/{label}-{op}.png")
        plt.close(fig)

@task
def plot_compare(c,
                 folders,
                 output_dir="plots",
                 strategy="box",
                 group_percentage=0.05,
                 sample_n=5,
                 remove_outliers=0.05,
                 show=True, merge_queries=True):
    """
    Compare multiple result folders in the same plots (shared axes) using fixed colors.

    The order of folders determines which color each dataset gets:
        1st folder → yellow
        2nd folder → steelblue
        3rd folder → salmon

    Example:
        invoke plot-compare --folders="logs/optimized,logs/baseline,logs/test"
    """
    # Parse folder list
    folders = [f.strip() for f in folders.split(",") if f.strip()]
    labels = [os.path.basename(f.rstrip("/")) or f for f in folders]

    # Fixed color palette
    colors = ["yellow", "steelblue", "salmon"]
    colors = [colors[i % len(colors)] for i in range(len(folders))]  # cycle if >3 folders

    # Load CSVs from each folder
    dfs = []
    for folder in folders:
        files = graph_utils.list_files_in_folder(folder)
        df_list = [df for f in files if (df := graph_utils.load_csv(f)) is not None]
        if not df_list:
            print(f"Warning: no CSVs found in folder '{folder}'")
            dfs.append(pd.DataFrame())
            continue
        combined_df = pd.concat(df_list, ignore_index=True)
        dfs.append(combined_df)
        print(f"Loaded {len(df_list)} CSVs from '{folder}'")

    # Generate combined plots
    plts = plot_multi(
        dataframes=dfs,
        labels=labels,
        colors=colors,
        group_percentage=group_percentage,
        sample_n=sample_n,
        strategy=strategy,
        remove_outliers=remove_outliers,
        merge_queries=merge_queries
    )

    # Save and optionally show
    os.makedirs(output_dir, exist_ok=True)
    for op, fig in plts.items():
        if show:
            fig.show()
        fig.savefig(f"{output_dir}/Comparison-{op}.png")
        plt.close(fig)

    print(f"All comparison plots saved in: {output_dir}")

ns = Collection()
ns.add_task(plot_single, name="single")
ns.add_task(plot_compare, name="compare")

