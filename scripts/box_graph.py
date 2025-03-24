import sys
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import graph_utils

width = 0.3

def plot_boxplot(datasets, group_percentage, v):
    """
    Plots a separate box plot for each operation with at most 20 x-axis ticks.

    Parameters:
    - datasets: List of dictionaries, each containing:
        - 'label': Name of the dataset
        - 'dataframes': List of Pandas DataFrames
        - 'color': Color for the boxplots
    - operation_group_sizes: Dictionary mapping each operation to its specific group size (optional).
      If not provided, defaults to a group size of 50 for all operations.
    """
    all_operations = set()
    for dataset in datasets:
        for df in dataset["dataframes"]:
            if isinstance(df, pd.DataFrame) and 'operation' in df.columns:
                all_operations.update(df["operation"].unique())

    all_operations = sorted(all_operations)

    for operation in all_operations:
        legend_handles = []
        plt.figure(figsize=(14, 6))
        base_positions = None

        #dataset for bench and dataset for base
        for idx, dataset in enumerate(datasets):
            label, dataframes, color = dataset["label"], dataset["dataframes"], dataset["color"]
            combined_df = pd.concat(dataframes, ignore_index=True)

            if 'id' not in combined_df.columns or 'time' not in combined_df.columns or 'operation' not in combined_df.columns:
                print(f"Error: DataFrames in '{label}' must contain 'id', 'time', and 'operation' columns.")
                return

            filtered_df = combined_df[combined_df['operation'] == operation].copy()

            total_ids = len(filtered_df['id'].unique())
            group_size = max(1, int(total_ids * group_percentage))

            # Use integer division to assign a group number to each row
            filtered_df['group'] = filtered_df['id'] // group_size
            grouped_df= [group for _, group in filtered_df.groupby('group')]

            # Prepare data for the boxplot.
            data_to_plot = [group['time'] for group in grouped_df]
            labels = [str(i * group_size) for i in range(len(grouped_df))]

            # Define x-axis positions
            if base_positions is None:
                base_positions = np.arange(len(grouped_df)) + 1 
            positions = base_positions + (idx * width)  

            legend_handles.append(mpatches.Patch(color=color, label=label))
            plt.boxplot(data_to_plot, patch_artist=True,
                        positions = positions,
                        boxprops=dict(facecolor=color), widths=width,
                        medianprops=dict(color='black'),
                        flierprops=dict(marker='o', markerfacecolor=color, markersize=6, linestyle='none'))


        plt.xticks(base_positions + (width / 2), [f"{i * group_size}-{(i + 1) * group_size}" for i in range(len(grouped_df))])
        plt.xlabel("Operation Index (Grouped IDs)")
        plt.ylabel("Time (μs)")
        plt.title(f"Box Plot by Aggregated Groups for Operation: {operation}")
        plt.legend(handles=legend_handles, loc="upper left")
        plt.tight_layout()
        #plt.show()
        plt.savefig(f"box_bench_{v}.png")

version = int(sys.argv[1]) if len(sys.argv) > 1 else 2

dataset_sources = graph_utils.get_dataset_sources(version)

datasets = []
for source in dataset_sources:
    folder = source["folder"]
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for file in files if (df := graph_utils.load_csv(file)) is not None]
    datasets.append({"dataframes": dfs, "label": source["label"], "color": source["color"]})

plot_boxplot(datasets, 0.05, version)
