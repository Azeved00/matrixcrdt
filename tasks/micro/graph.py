from invoke import task
from invoke import Collection
import pandas as pd
import os

from scripts.micro.graph.box_graph import plot_boxplot_single, plot_boxplot_dual
from scripts.micro.graph.line_graph import plot_regression_single, plot_regression_dual
import scripts.micro.graph as graph_utils


#box graph
@task
def box_single(c, folder="logs/optimized", output_dir="plots", label="Optimized", group_percentage=0.05, sample_n=5,
               show=True):
    """ Plot a box plot."""
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for f in files if (df := graph_utils.load_csv(f)) is not None]

    print(f"{len(dfs)} dataframes loaded")
    combined_df = pd.concat(dfs, ignore_index=True)

    plts = plot_boxplot_single(
        dataframe=combined_df,
        group_percentage=group_percentage,
        sample_n=sample_n,
    )

    for op,plt in plts.items():
        if show:
            plt.show()
        os.makedirs(output_dir, exist_ok=True)
        plt.savefig(f"{output_dir}/{label}-{op}.png")
        plt.close()
    
@task
def box_dual(c,
              folder1="data/optimized",label1="Optimized",
              folder2="data/baseline", label2="Baseline",
              show=True, output_dir="plots",
              group_percentage=0.05, sample_n=5):
    """ Plot a comparisson box plot."""

    files1 = graph_utils.list_files_in_folder(folder1)
    dfs1 = [df for f in files1 if (df := graph_utils.load_csv(f)) is not None]
    combined_df1 = pd.concat(dfs1, ignore_index=True)

    files2 = graph_utils.list_files_in_folder(folder2)
    dfs2 = [df for f in files2 if (df := graph_utils.load_csv(f)) is not None]
    combined_df2 = pd.concat(dfs2, ignore_index=True)

    plts = plot_boxplot_dual(
        df1=combined_df1, label1=label1,
        df2=combined_df2, label2=label2,
        group_percentage=group_percentage,sample_n= sample_n,)

    for op,plt in plts.items():
        if show:
            plt.show()
        os.makedirs(output_dir, exist_ok=True)
        plt.savefig(f"{output_dir}/{label1}x{label2}-{op}.png")
        plt.close()

#line graph
@task
def line_single(c, folder, show=False, sample_n=5, output_dir="plots"):
    """ Plot a line plot."""
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for file in files if (df := graph_utils.load_csv(file)) is not None]

    if not dfs:
        print("No valid CSV files found.")
        return

    combined_df = pd.concat(dfs, ignore_index=True)
    plts = plot_regression_single(combined_df, sample_n=int(sample_n))
    for op,plt in plts.items():
        if show:
            plt.show()
        os.makedirs(output_dir, exist_ok=True)
        plt.savefig(f"{output_dir}/{label}-{op}.png")
        plt.close()

@task
def line_dual(c, label1, folder1, label2, folder2,show=False, sample_n=5, output_dir="plots"):
    """ Plot a comparisson line plot."""
    files1 = graph_utils.list_files_in_folder(folder1)
    files2 = graph_utils.list_files_in_folder(folder2)

    dfs1 = [df for file in files1 if (df := graph_utils.load_csv(file)) is not None]
    dfs2 = [df for file in files2 if (df := graph_utils.load_csv(file)) is not None]
    combined_df1 = pd.concat(dfs1, ignore_index=True)
    combined_df2 = pd.concat(dfs2, ignore_index=True)

    plts = plot_regression_pair(dfs1, label1, dfs2, label2, sample_n=int(sample_n))
    for op,plt in plts.items():
        if show:
            plt.show()
        os.makedirs(output_dir, exist_ok=True)
        plt.savefig(f"{output_dir}/{label1}x{label2}-{op}.png")
        plt.close()

# proces logs 
ns = Collection()
ns.add_task(box_single, name="box_single")
ns.add_task(box_dual, name="box_dual")
ns.add_task(line_single, name="line_single")
ns.add_task(line_dual, name="line_dual")

