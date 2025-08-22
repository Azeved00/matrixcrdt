from invoke import task
from invoke import Collection
import matplotlib.pyplot as plt
import pandas as pd
import os

from scripts.micro.graph.plot_graph import plot_single as plot
import scripts.micro.graph as graph_utils


@task
def plot_single(c, folder="logs/optimized", output_dir="plots",
            label="Optimized", color="blue", strategy="box",
            group_percentage=0.05, sample_n=5, remove_outliers=5,
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


# proces logs 
ns = Collection()
ns.add_task(plot_single, name="single")

