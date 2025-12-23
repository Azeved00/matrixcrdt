from invoke import task
from invoke import Collection
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import pandas as pd
import os

from scripts.micro.graph.plot_graph import plot_single as plot, plot_multi, plot_merged_queries
import scripts.micro.graph as graph_utils
from scripts.micro.graph import make_table, table_to_latex

@task
def table(c, folder="logs/optimized",
                           label="authdag", group_percentage=0.05, sample_n=5,
                           remove_outliers=0.05, boxes=5, latex=False):
    """Task to generate a summary table from CSV files in a folder.
       Prints as LaTeX if latex=True, otherwise prints normally.
    """

    # Load CSV files
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for f in files if (df := graph_utils.load_csv(f)) is not None]

    print(f"{len(dfs)} dataframes loaded")
    combined_df = pd.concat(dfs, ignore_index=True)

    # Generate the table
    summary_df = make_table(
        dataframe=combined_df,
        group_percentage=group_percentage,
        sample_n=sample_n,
        boxes=boxes
    )

    # Print table
    if latex:
        latex_str = table_to_latex(summary_df, label=label)
        print(latex_str)
    else:
        print(summary_df)

@task
def plot_single(c, folder="logs/optimized", output_dir="plots",
            label="Optimized", color="blue", strategy="box",
            group_percentage=0.05, sample_n=5, remove_outliers=0.5,
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
        ax = fig.axes[0]
        patch = mpatches.Patch(color=color, label=label)
        ax.legend(handles=[patch])

        os.makedirs(output_dir, exist_ok=True)
        fig.savefig(f"{output_dir}/{label}-{op}.png")

    if show:
        plt.show()
    plt.close("all")

@task
def plot_merged(c, folder="logs/optimized", output_dir="plots",
                label="Optimized", colors=("blue", "red"), strategy="line",
                group_percentage=0.05, sample_n=5, remove_outliers=0.05,
                show=True):
    """ Plot merged queries (stateful_query vs stateless_query). """
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for f in files if (df := graph_utils.load_csv(f)) is not None]

    print(f"{len(dfs)} dataframes loaded")
    combined_df = pd.concat(dfs, ignore_index=True)

    fig = plot_merged_queries(
        dataframe=combined_df,
        group_percentage=group_percentage,
        sample_n=sample_n,
        colors=colors,
        strategy=strategy,
        remove_outliers=remove_outliers,
    )

    ax = fig.axes[0]
    patches = [mpatches.Patch(color=colors[i], label=label + f" ({name})")
               for i, name in enumerate(["Stateless", "Stateful"])]
    ax.legend(handles=patches)

    os.makedirs(output_dir, exist_ok=True)
    fig.savefig(f"{output_dir}/{label}-merged_queries.png")

    if show:
        plt.show()
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

    # Fixed color palette
    labels = ["Cached", "Baseline" , "Authenticated"]
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
        fig.savefig(f"{output_dir}/Comparison-{op}.png")

    if show:
        plt.show()
    plt.close("all")

    print(f"All comparison plots saved in: {output_dir}")

ns = Collection()
ns.add_task(plot_single, name="single")
ns.add_task(plot_compare, name="compare")
ns.add_task(table, name="table")
ns.add_task(plot_merged, name="merged")

