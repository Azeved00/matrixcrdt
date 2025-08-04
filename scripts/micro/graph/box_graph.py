import sys
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import textwrap
from .graph_utils import *

width = 0.3

def plot_boxplot_single(dataframes, group_percentage, save, n, label):
    color = "lightblue"
    all_operations = set()
    for df in dataframes:
        if isinstance(df, pd.DataFrame) and 'operation' in df.columns:
            all_operations.update(df["operation"].unique())
    all_operations = sorted(all_operations)

    for operation in all_operations:
        plt.figure(figsize=(14, 6))
        legend_handles = []
        base_positions = None

        combined_df = pd.concat(dataframes, ignore_index=True)

        if 'id' not in combined_df.columns or 'time' not in combined_df.columns or 'operation' not in combined_df.columns:
            print(f"Error: DataFrames must contain 'id', 'time', and 'operation' columns.")
            return

        filtered_df = combined_df[combined_df['operation'] == operation].copy()
        total_ids = len(filtered_df['id'].unique())
        group_size = max(1, int(total_ids * group_percentage))

        filtered_df['group'] = filtered_df['id'] // group_size
        grouped_df = [group for _, group in filtered_df.groupby('group')]

        data_to_plot = [group['time'] for group in grouped_df]
        base_positions = np.arange(len(grouped_df)) + 1

        plt.boxplot(data_to_plot, patch_artist=True,
                    positions=base_positions,
                    boxprops=dict(facecolor=color),
                    widths=width,
                    medianprops=dict(color='black'),
                    flierprops=dict(marker='o', markerfacecolor=color, markersize=6, linestyle='none'))

        legend_handles.append(mpatches.Patch(color=color, label=label))

        # X-tick formatting
        if operation == "query":
            xticks = [f"{i*n}" for i in range(len(grouped_df))] if group_size == 1 else \
                [textwrap.fill(f"{i*n*group_size}-{(i+1)*n*group_size}", 5) for i in range(len(grouped_df))]
        else:
            xticks = [f"{i}" for i in range(len(grouped_df))] if group_size == 1 else \
                [textwrap.fill(f"{i*group_size}-{(i+1)*group_size}", 5) for i in range(len(grouped_df))]

        plt.xticks(base_positions, xticks)
        plt.xlabel("Operation Index (Grouped IDs)")
        plt.ylabel("Time (μs)")
        plt.title(f"Box Plot for Operation: {operation}")
        plt.legend(handles=legend_handles, loc="upper left")
        plt.tight_layout(pad=3)

        if save:
            plt.savefig(f"single_boxplot_{operation}.png")
        else:
            plt.show()
        plt.close()

def plot_boxplot_dual(df_group1, df_group2, label1, label2, group_percentage, save, n):
    datasets = [
        {"label": label1, "dfs": df_group1, "color": "lightblue"},
        {"label": label2, "dfs": df_group2, "color": "lightcoral"}
    ]

    all_operations = set()
    for ds in datasets:
        for df in ds["dfs"]:
            if isinstance(df, pd.DataFrame) and 'operation' in df.columns:
                all_operations.update(df["operation"].unique())
    all_operations = sorted(all_operations)

    for operation in all_operations:
        plt.figure(figsize=(14, 6))
        legend_handles = []
        base_positions = None

        for idx, ds in enumerate(datasets):
            label, dfs, color = ds["label"], ds["dfs"], ds["color"]
            combined_df = pd.concat(dfs, ignore_index=True)

            if 'id' not in combined_df.columns or 'time' not in combined_df.columns or 'operation' not in combined_df.columns:
                print(f"Error: DataFrames in '{label}' must contain 'id', 'time', and 'operation'.")
                return

            filtered_df = combined_df[combined_df['operation'] == operation].copy()
            total_ids = len(filtered_df['id'].unique())
            group_size = max(1, int(total_ids * group_percentage))

            filtered_df['group'] = filtered_df['id'] // group_size
            grouped_df = [group for _, group in filtered_df.groupby('group')]
            data_to_plot = [group['time'] for group in grouped_df]

            if base_positions is None:
                base_positions = np.arange(len(grouped_df)) + 1
            positions = base_positions + (idx * width)

            plt.boxplot(data_to_plot, patch_artist=True,
                        positions=positions,
                        boxprops=dict(facecolor=color),
                        widths=width,
                        medianprops=dict(color='black'),
                        flierprops=dict(marker='o', markerfacecolor=color, markersize=6, linestyle='none'))

            legend_handles.append(mpatches.Patch(color=color, label=label))

        # X-ticks
        xticks = []
        if group_size == 1:
            xticks = [f"{i*n}" for i in range(len(grouped_df))] if operation == "query" else [f"{i}" for i in range(len(grouped_df))]
        else:
            if operation == "query":
                xticks = [textwrap.fill(f"{i*n*group_size}-{(i+1)*n*group_size}", 5) for i in range(len(grouped_df))]
            else:
                xticks = [textwrap.fill(f"{i*group_size}-{(i+1)*group_size}", 5) for i in range(len(grouped_df))]

        plt.xticks(base_positions + (width / 2), xticks)
        plt.xlabel("Operation Index (Grouped IDs)")
        plt.ylabel("Time (μs)")
        plt.title(f"Box Plot Comparison for Operation: {operation}")
        plt.legend(handles=legend_handles, loc="upper left")
        plt.tight_layout(pad=3)

        if save:
            plt.savefig(f"dual_boxplot_{operation}.png")
        else:
            plt.show()
        plt.close()

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Plot single or dual benchmark boxplots.")
    parser.add_argument("folders", nargs="+", help="Folder(s) containing CSVs. One for single, two for dual.")
    parser.add_argument("--labels", nargs="+", default=["Dataset A", "Dataset B"], help="Labels for the datasets.")
    parser.add_argument("--save", action="store_true", help="Save the plot instead of displaying it.")
    parser.add_argument("--group-percentage", type=float, default=0.05, help="Percentage of IDs per group.")
    parser.add_argument("-n", type=int, default=5, help="Scaling multiplier for query buckets.")

    args = parser.parse_args()
    folders = args.folders

    if len(folders) == 1:
        files = graph_utils.list_files_in_folder(folders[0])
        dfs = [df for f in files if (df := graph_utils.load_csv(f)) is not None]
        plot_boxplot_single(dfs, group_percentage=args.group_percentage, save=args.save, n=args.n, label=args.labels[0])
    elif len(folders) == 2:
        files1 = graph_utils.list_files_in_folder(folders[0])
        files2 = graph_utils.list_files_in_folder(folders[1])
        dfs1 = [df for f in files1 if (df := graph_utils.load_csv(f)) is not None]
        dfs2 = [df for f in files2 if (df := graph_utils.load_csv(f)) is not None]
        plot_boxplot_dual(dfs1, dfs2, args.labels[0], args.labels[1], group_percentage=args.group_percentage, save=args.save, n=args.n)
    else:
        print("Error: Please specify one or two folders.")

