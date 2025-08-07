import sys
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import textwrap
from .graph_utils import *

width = 0.3

def plot_boxplot_single(dataframe, group_percentage, sample_n):
    """
    Generates grouped box plots for each operation type from a single dataset.

    Inputs:
    dataframe : pandas.DataFrame
        The input data containing 'id', 'time', and 'operation' columns.
    group_percentage : float
        Fraction (0 < value <= 1) representing the percentage of IDs per group.
    sample_n : int
        Number of samples per group (used for x-axis label formatting, especially for "query" operations).

    Outputs: A dictionary from operation names to matplotlib plot object.
    """
    color = "lightblue"
    all_operations = dataframe["operation"].unique()

    final = {}
    for operation in all_operations:
        plt.figure(figsize=(14, 6))
        legend_handles = []
        base_positions = None

        if 'id' not in dataframe.columns or 'time' not in dataframe.columns or 'operation' not in dataframe.columns:
            print(f"Error: DataFrames must contain 'id', 'time', and 'operation' columns.")
            return

        filtered_df = dataframe[dataframe['operation'] == operation].copy()
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

        # X-tick formatting
        if operation == "query":
            xticks = [f"{i*n}" for i in range(len(grouped_df))] if group_size == 1 else \
                [textwrap.fill(f"{i*sample_n*group_size}-{(i+1)*sample_n*group_size}", 5) for i in range(len(grouped_df))]
        else:
            xticks = [f"{i}" for i in range(len(grouped_df))] if group_size == 1 else \
                [textwrap.fill(f"{i*group_size}-{(i+1)*group_size}", 5) for i in range(len(grouped_df))]

        plt.xticks(base_positions, xticks)
        plt.xlabel("Operation Index (Grouped IDs)")
        plt.ylabel("Time (μs)")
        plt.title(f"Box Plot for Operation: {operation}")
        plt.tight_layout(pad=3)
        
        final[operation] = plt
    return final

def plot_boxplot_dual(df1, df2, label1, label2, group_percentage,sample_n):
    """
    Creates side-by-side box plots to compare two datasets for each operation type.

    Each plot shows grouped execution times for both datasets, allowing direct visual comparison
    across identical operation categories.

    Inputs
    df1 : pandas.DataFrame
        First dataset containing 'id', 'time', and 'operation' columns.
    df2 : pandas.DataFrame
        Second dataset, structured the same as df1.
    label1 : str
        Label for the first dataset to be shown in the plot legend.
    label2 : str
        Label for the second dataset.
    group_percentage : float
        Fraction (0 < value <= 1) representing the percentage of IDs per group for each dataset.
    sample_n : int
        Number of samples per group (used for x-axis label formatting, especially for "query" operations).

    Outputs: A dictionary from operation names to matplotlib plot object.
    """
    datasets = [
        {"label": label1, "dfs": df1, "color": "lightblue"},
        {"label": label2, "dfs": df2, "color": "lightcoral"}
    ]

    all_operations = set()
    for ds in datasets:
            all_operations.update(ds["dfs"]["operation"].unique())

    final ={}
    for operation in all_operations:
        plt.figure(figsize=(14, 6))
        legend_handles = []
        base_positions = None

        for idx, ds in enumerate(datasets):
            label, dfs, color = ds["label"], ds["dfs"], ds["color"]

            if 'id' not in dfs.columns or 'time' not in dfs.columns or  'operation' not in dfs.columns:
                print(f"Error: DataFrames in '{label}' must contain 'id', 'time', and 'operation'.")
                return

            filtered_df = dfs[dfs['operation'] == operation].copy()
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
        
        final[operation] = plt
    return final

