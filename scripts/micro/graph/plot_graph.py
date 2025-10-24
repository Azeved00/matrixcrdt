import sys
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import textwrap
from .graph_utils import *

WIDTH = 0.3
OPERATION_TEXT = {
    "apply": "Update",
    "query": "Query",
    "stateful_query": "Query",
    "stateless_query": "Query"
}

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
        total_ids = len(filtered_df['id'].unique())
        if remove_outliers is not None and 0 < remove_outliers < 100:
            threshold = filtered_df['time'].quantile((100 - remove_outliers)/100)
            
            removed_rows = filtered_df[filtered_df['time'] > threshold]
            print(f"Removing top {len(removed_rows[['id', 'time']])}({remove_outliers}%) outliers for operation '{operation}'")
            filtered_df = filtered_df[filtered_df['time'] <= threshold]  
            print(f"Kept {len(filtered_df)} outliers for operation '{operation}'")

        match strategy:
            case "box":
                group_size = max(1, int(total_ids * group_percentage))
                filtered_df['group'] = (filtered_df['id'] - 1) // group_size
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

        ax.set_xlabel("Operation Index (Grouped IDs)")
        ax.set_ylabel("Time (μs)")
        ax.set_title(f"{strategy.capitalize()} Plot for {OPERATION_TEXT[operation]}")
        fig.tight_layout(pad=3)

        final[operation] = fig   

    return final

def plot_multi(dataframes, labels, group_percentage=0.05, sample_n=5, strategy="box", 
               colors=None, remove_outliers=0.0, merge_queries=False):
    """
    Generates grouped plots for multiple datasets. Boxes are plotted side by side.
    Merges 'stateful_query' and 'stateless_query' if merge_queries=True.

    Args:
        dataframes (list of pd.DataFrame): List of DataFrames to plot.
        labels (list of str): Labels corresponding to each DataFrame.
        group_percentage (float): Fraction of IDs to include in each group.
        sample_n (int): Number of samples per ID.
        strategy (str): "box" or "line".
        colors (list of str): Colors for each DataFrame (default: ['yellow', 'steelblue', 'salmon']).
        remove_outliers (float): Fraction of top outliers to remove (0-1).
        merge_queries (bool): If True, merge 'stateful_query' and 'stateless_query' into 'query'.
    
    Returns:
        dict: Dictionary mapping operation names to matplotlib Figure objects.
    """
    if len(dataframes) != len(labels):
        raise ValueError("Length of dataframes and labels must match.")
    
    # Default colors
    default_colors = ["yellow", "steelblue", "salmon"]
    if colors is None:
        colors = default_colors
    colors = [colors[i % len(colors)] for i in range(len(dataframes))]

    # Merge queries if needed
    if merge_queries:
        merged_dataframes = []
        for df in dataframes:
            df_copy = df.copy()
            df_copy.loc[df_copy['operation'].isin(['stateful_query', 'stateless_query']), 'operation'] = 'query'
            merged_dataframes.append(df_copy)
        dataframes = merged_dataframes

    # Collect all operations across all dataframes
    all_operations = sorted(set(op for df in dataframes for op in df["operation"].unique()))
    final = {}

    for operation in all_operations:
        fig, ax = plt.subplots(figsize=(14, 6))

        # Filter each dataframe by operation and handle outliers
        filtered_dfs = []
        max_ids = 0
        for df in dataframes:
            filtered_df = df[df['operation'] == operation].copy()
            
            if remove_outliers and 0 < remove_outliers < 100:
                threshold = filtered_df['time'].quantile((100 - remove_outliers)/100)
                filtered_df = filtered_df[filtered_df['time'] <= threshold]

            filtered_dfs.append(filtered_df)
            max_ids = max(max_ids, len(filtered_df['id'].unique()))

        # Determine group size based on the largest dataframe
        group_size = max(1, int(max_ids * group_percentage))
        num_groups = max_ids // group_size + (1 if max_ids % group_size else 0)
        base_positions = np.arange(num_groups) + 1

        # Width adjustment for side-by-side boxes
        num_dfs = len(dataframes)
        box_width = WIDTH  
        offsets = np.linspace(-((num_dfs-1)/2)*WIDTH, ((num_dfs-1)/2)*WIDTH, num_dfs)

        # Collect proxy artists for legend
        proxy_artists = []

        for df_idx, filtered_df in enumerate(filtered_dfs):
            label = labels[df_idx]
            color = colors[df_idx]

            match strategy:
                case "box":
                    filtered_df['group'] = filtered_df['id'] // group_size
                    grouped = filtered_df.groupby('group')
                    grouped_df = [group for _, group in grouped]
                    data_to_plot = [group['time'] for group in grouped_df]

                    group_nums = sorted(grouped.groups.keys())
                    positions = np.arange(len(group_nums)) + 1 + offsets[df_idx]

                    ax.boxplot(
                        data_to_plot,
                        patch_artist=True,
                        positions=positions,
                        boxprops=dict(facecolor=color),
                        widths=box_width,
                        medianprops=dict(color='black'),
                        flierprops=dict(marker='o', markerfacecolor=color, markersize=6, linestyle='none'),
                        manage_ticks=False
                    )

                    # add proxy patch for legend
                    proxy_artists.append(mpatches.Patch(color=color, label=label))

                case "line":
                    avg_time_per_id = filtered_df.groupby('id')['time'].mean().reset_index()
                    avg_time_per_id = avg_time_per_id.sort_values(by="id")

                    ax.scatter(avg_time_per_id['id'], avg_time_per_id['time'], color=color, alpha=0.6, label=label)

                    coefficients = np.polyfit(avg_time_per_id['id'], avg_time_per_id['time'], deg=1)
                    poly_eq = np.poly1d(coefficients)

                    x_values = np.linspace(avg_time_per_id['id'].min(), avg_time_per_id['id'].max(), 100)
                    y_values = poly_eq(x_values)
                    ax.plot(x_values, y_values, color=color, linewidth=2, label=label)

        # Add legend once
        ax.legend(handles=proxy_artists if strategy == "box" else None)

        # X-tick formatting
        xticks = [
            textwrap.fill(f"{i*group_size}-{(i+1)*group_size}", 5)
            for i in range(len(base_positions))
        ]
        ax.set_xticks(base_positions)
        ax.set_xticklabels(xticks)

        ax.set_xlabel("Operation Index (Grouped IDs)")
        ax.set_ylabel("Time (μs)")
        ax.set_title(f"{strategy.capitalize()} Plot for {OPERATION_TEXT[operation]}")
        fig.tight_layout(pad=3)

        final[operation] = fig

    return final


def plot_merged_queries(dataframe, group_percentage, sample_n, strategy,
                        colors=("lightblue", "salmon"), remove_outliers=0.0):
    """
    Plots stateful_query and stateless_query together on the same figure,
    by creating a new column `query` and coloring by it.

    Supports strategies: "box", "line", and "cumulative".

    - "box": grouped boxplots of times by IDs.
    - "line": scatter + regression lines.
    - "cumulative": plots **cumulative time** of stateless queries between stateful queries,
      showing how total time evolves until the next stateful reset.

    Outputs: A matplotlib Figure object.
    """
    queries = ["stateful_query", "stateless_query"]
    missing = [q for q in queries if q not in dataframe["operation"].unique()]
    if missing:
        raise ValueError(f"Missing required operations: {missing}")

    # Create a unified query column
    df = dataframe[dataframe["operation"].isin(queries)].copy()
    df["query"] = df["operation"].replace({"stateful_query": "Stateful", "stateless_query": "Stateless"})
    df["operation"] = "query"

    # Multiply id values by sample_n
    df["id"] = df["id"] * sample_n

    fig, ax = plt.subplots(figsize=(14, 6))

    match strategy:
        case "box" | "line":
            for i, query in enumerate(df["query"].unique()):
                filtered_df = df[df['query'] == query].copy()
                total_ids = len(filtered_df['id'].unique())

                # Handle outliers
                if remove_outliers is not None and 0 < remove_outliers < 100:
                    threshold = filtered_df['time'].quantile((100 - remove_outliers)/100)
                    filtered_df = filtered_df[filtered_df['time'] <= threshold]

                match strategy:
                    case "box":
                        group_size = max(1, int(total_ids * group_percentage))
                        filtered_df['group'] = (filtered_df['id'] - 1) // group_size
                        grouped_df = [group for _, group in filtered_df.groupby('group')]
                        data_to_plot = [group['time'] for group in grouped_df]
                        base_positions = np.arange(len(grouped_df)) + 1 + (i * 0.3)

                        ax.boxplot(
                            data_to_plot,
                            patch_artist=True,
                            positions=base_positions,
                            boxprops=dict(facecolor=colors[i]),
                            widths=0.25,
                            medianprops=dict(color='black'),
                            flierprops=dict(marker='o', markerfacecolor=colors[i], markersize=6, linestyle='none')
                        )

                    case "line":
                        avg_time_per_id = filtered_df.groupby('id')['time'].mean().reset_index()
                        avg_time_per_id = avg_time_per_id.sort_values(by="id")

                        ax.scatter(avg_time_per_id['id'], avg_time_per_id['time'], color=colors[i], alpha=0.6, label=f"{query} points")

                        coefficients = np.polyfit(avg_time_per_id['id'], avg_time_per_id['time'], deg=1)
                        poly_eq = np.poly1d(coefficients)

                        x_values = np.linspace(avg_time_per_id['id'].min(), avg_time_per_id['id'].max(), 100)
                        y_values = poly_eq(x_values)

                        ax.plot(x_values, y_values, color=colors[i], linewidth=2, label=f"{query} regression")

        case "cumulative":
            # Compute cumulative time for stateless queries between stateful queries
            df = df.sort_values(by="id").reset_index(drop=True)
            cum_time = []
            total = 0
            for _, row in df.iterrows():
                if row["query"] == "Stateful":
                    total = 0  # reset cumulative time
                    continue
                else:
                    total += row["time"]
                    cum_time.append((row["id"], total))

            if cum_time:
                ids, times = zip(*cum_time)
                ax.scatter(ids, times, color=colors[1], alpha=0.6, label="Cumulative Stateless")

                # Regression line
                coefficients = np.polyfit(ids, times, deg=1)
                poly_eq = np.poly1d(coefficients)

                x_values = np.linspace(min(ids), max(ids), 100)
                y_values = poly_eq(x_values)

                ax.plot(x_values, y_values, color=colors[1], linewidth=2, label="Regression")

            ax.set_xlabel("Query ID")

    ax.set_ylabel("Cumulative Time (μs)")
    ax.set_title(f"{strategy.capitalize()} Plot for Queries (Stateless Cumulative)")
    ax.legend()
    fig.tight_layout(pad=3)

    return fig

