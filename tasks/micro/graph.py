from invoke import task
from invoke import Collection
import pandas as pd

from scripts.micro.graph.box_graph import plot_boxplot_single, plot_boxplot_dual
from scripts.micro.graph.line_graph import plot_regression_single, plot_regression_dual
import scripts.micro.graph as graph_utils


#box graph
@task
def box_single(c, folder="logs/optimized", label="Optimized", save=True, group_percentage=0.05, n=5):
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for f in files if (df := graph_utils.load_csv(f)) is not None]
    plot_boxplot_single(
        dataframes=dfs,
        group_percentage=group_percentage,
        save=save,
        n=n,
        label=label
    )

@task
def box_dual(c,
              folder1="data/optimized",
              folder2="data/baseline",
              label1="Optimized",
              label2="Baseline",
              save=True,
              group_percentage=0.05,
              n=5):
    files1 = graph_utils.list_files_in_folder(folder1)
    dfs1 = [df for f in files1 if (df := graph_utils.load_csv(f)) is not None]

    files2 = graph_utils.list_files_in_folder(folder2)
    dfs2 = [df for f in files2 if (df := graph_utils.load_csv(f)) is not None]

    plot_boxplot_dual(
        df_group1=dfs1,
        df_group2=dfs2,
        label1=label1,
        label2=label2,
        group_percentage=group_percentage,
        save=save, n=n)

#line graph
@task
def line_single(c, folder, n=5, save=False, show=True):
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for file in files if (df := graph_utils.load_csv(file)) is not None]

    if not dfs:
        print("No valid CSV files found.")
        return

    combined_df = pd.concat(dfs, ignore_index=True)
    plot_regression_single(combined_df, n=int(n), save_fig=bool(save), show_plot=bool(show))

@task
def line_dual(c, label1, folder1, label2, folder2, n=5, save=False, show=True):
    files1 = graph_utils.list_files_in_folder(folder1)
    files2 = graph_utils.list_files_in_folder(folder2)

    dfs1 = [df for file in files1 if (df := graph_utils.load_csv(file)) is not None]
    dfs2 = [df for file in files2 if (df := graph_utils.load_csv(file)) is not None]

    plot_regression_pair(dfs1, label1, dfs2, label2, n=int(n), save_fig=bool(save), show_plot=bool(show))

# proces logs 
ns = Collection()
ns.add_task(box_single, name="box_single")
ns.add_task(box_dual, name="box_dual")
ns.add_task(line_single, name="line_single")
ns.add_task(line_dual, name="line_dual")

