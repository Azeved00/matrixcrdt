import sys
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import textwrap
from .graph_utils import *

WIDTH = 0.3

def plot_single(dataframe, group_percentage, sample_n, strategy, 
                color="lightblue", remove_outliers=0.0):
    """
    Generates grouped box plots for each operation type from a single dataset.

    Outputs: A dictionary from operation names to matplotlib Figure objects.
    """
    all_operations = dataframe["operation"].unique()
    final = {}

    if not all(col in dataframe.columns for col in ['id', 'time', 'operation']):
        raise ValueError("DataFrame must contain 'id', 'time', and 'operation' columns.")

    for operation in all_operations:
        fig, ax = plt.subplots(figsize=(14, 6))  

        filtered_df = dataframe[dataframe['operation'] == operation].copy()
        if remove_outliers is not None and 0 < remove_outliers < 1:
            threshold = filtered_df['time'].quantile(1 - remove_outliers)
            
            removed_rows = filtered_df[filtered_df['time'] > threshold]
            print(f"Removing top {len(removed_rows[['id', 'time']])}({remove_outliers*100}%) outliers for operation '{operation}'")
            filtered_df = filtered_df[filtered_df['time'] <= threshold]  

        match strategy:
            case "box":
                total_ids = len(filtered_df['id'].unique())
                group_size = max(1, int(total_ids * group_percentage))
                filtered_df['group'] = filtered_df['id'] // group_size
                grouped_df = [group for _, group in filtered_df.groupby('group')]
                data_to_plot = [group['time'] for group in grouped_df]
                base_positions = np.arange(len(grouped_df)) + 1

                ax.boxplot(
                    data_to_plot,
                    patch_artist=True,
                    positions=base_positions,
                    boxprops=dict(facecolor=color),
                    widths=WIDTH,  
                    medianprops=dict(color='black'),
                    flierprops=dict(marker='o', markerfacecolor=color, markersize=6, linestyle='none')
                )

                # X-tick formatting
                if operation == "query":
                    if group_size == 1:
                        xticks = [f"{i*sample_n}" for i in range(len(grouped_df))]
                    else:
                        xticks = [
                            textwrap.fill(f"{i*sample_n*group_size}-{(i+1)*sample_n*group_size}", 5)
                            for i in range(len(grouped_df))
                        ]
                else:
                    if group_size == 1:
                        xticks = [f"{i}" for i in range(len(grouped_df))]
                    else:
                        xticks = [
                            textwrap.fill(f"{i*group_size}-{(i+1)*group_size}", 5)
                            for i in range(len(grouped_df))
                        ]

                ax.set_xticks(base_positions)
                ax.set_xticklabels(xticks)

            case "line":
                avg_time_per_id = filtered_df.groupby('id')['time'].mean().reset_index()
                avg_time_per_id = avg_time_per_id.sort_values(by="id")

                ax.scatter(avg_time_per_id['id'], avg_time_per_id['time'], color=color, alpha=0.6, label="Data Points")

                coefficients = np.polyfit(avg_time_per_id['id'], avg_time_per_id['time'], deg=1)
                poly_eq = np.poly1d(coefficients)

                x_values = np.linspace(avg_time_per_id['id'].min(), avg_time_per_id['id'].max(), 100)
                y_values = poly_eq(x_values)

                ax.plot(x_values, y_values, color=color, linewidth=2, label="Regression Line")

                if operation in {"stateful_query", "stateless_query"}:
                    xticks = plt.xticks()[0][1:-1]
                    ax.set_xticks(xticks, labels=[f"{int(tick * sample_n)}" for tick in xticks])

                ax.legend()

        ax.set_xlabel("Operation Index (Grouped IDs)")
        ax.set_ylabel("Time (μs)")
        ax.set_title(f"{strategy} Plot for Operation: {operation}")
        fig.tight_layout(pad=3)

        final[operation] = fig   

    return final

def plot_multi(dataframes, labels, colors=None, group_percentage=0.05,
               sample_n=5, strategy="box", remove_outliers=0.0,
               merge_queries=False):
    """
    Generate comparison plots for multiple datasets in the same figure.
    Shorter datasets simply stop plotting when they run out of groups.

    :param merge_queries: If True, merges 'stateful_query' and 'stateless_query' into 'query'
    """
    import matplotlib.pyplot as plt
    import numpy as np
    import textwrap

    WIDTH = 0.3
    if colors is None:
        base_colors = ["yellow", "steelblue", "salmon"]
        colors = [base_colors[i % len(base_colors)] for i in range(len(dataframes))]

    # Optionally merge queries
    processed_dfs = []
    for df in dataframes:
        df_copy = df.copy()
        if merge_queries:
            df_copy.loc[df_copy['operation'].isin(['stateful_query', 'stateless_query']), 'operation'] = 'query'
        processed_dfs.append(df_copy)

    # Collect all unique operations
    all_operations = sorted({op for df in processed_dfs for op in df['operation'].unique()})
    final = {}

    for operation in all_operations:
        fig, ax = plt.subplots(figsize=(14, 6))

        for idx, (df, color, label) in enumerate(zip(processed_dfs, colors, labels)):
            filtered_df = df[df['operation'] == operation].copy()
            if remove_outliers and 0 < remove_outliers < 1:
                threshold = filtered_df['time'].quantile(1 - remove_outliers)
                filtered_df = filtered_df[filtered_df['time'] <= threshold]

            if filtered_df.empty:
                continue  # nothing to plot for this dataset

            total_ids = len(filtered_df['id'].unique())
            group_size = max(1, int(total_ids * group_percentage))
            filtered_df['group'] = filtered_df['id'] // group_size
            grouped_df = [group['time'].values for _, group in filtered_df.groupby('group')]

            # Positions relative to dataset only
            positions = np.arange(1, len(grouped_df) + 1)
            offset = (idx - (len(dataframes)-1)/2) * WIDTH * 1.1
            positions = positions + offset

            if strategy == "box":
                ax.boxplot(
                    grouped_df,
                    patch_artist=True,
                    positions=positions,
                    widths=WIDTH,
                    boxprops=dict(facecolor=color),
                    medianprops=dict(color='black'),
                    flierprops=dict(marker='o', markerfacecolor=color, markersize=6, linestyle='none')
                )
            elif strategy == "line":
                # line plotting logic here if needed
                pass

        ax.set_xlabel("Operation Index (Grouped IDs)")
        ax.set_ylabel("Time (μs)")
        ax.set_title(f"{strategy} Plot for Operation: {operation}")
        fig.tight_layout(pad=3)
        final[operation] = fig

    return final

