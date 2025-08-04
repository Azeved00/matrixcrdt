from invoke import task
from invoke import Collection
import pandas as pd

from scripts.micro.graph.box_graph import plot_boxplot_single, plot_boxplot_dual
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

# proces logs 
ns = Collection()
ns.add_task(box_single, name="box_single")
ns.add_task(box_dual, name="box_dual")

