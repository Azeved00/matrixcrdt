import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import sys
import os
import seaborn as sns
import matplotlib.patches as mpatches
import textwrap

strategies = ["scatter", "box", "mean"]
WIDTH = 0.3
OPERATION_TEXT = {
    "apply": "Update",
    "query": "Query",
    "stateful_query": "Query",
    "stateless_query": "Query"
}

def plot_graphs(df, strategy: str = "box", 
                color="steelblue", front_color="orange", 
                remove_outliers=0.0, num_groups=20,
                include_front=True, use_dag_ops=False):
    """
    Plots benchmarking data using the specified visualization strategy.

    Args:
        df (pd.DataFrame): The benchmarking data.
        strategy (str): Visualization strategy to use. Options are:
                        "scatter", "mean", or "box".
        color (str): Color for backend plots.
        front_color (str): Color for frontend plots.
        remove_outliers (float): Fraction of top entries to remove
                                 (based on front_time if include_front=True,
                                 otherwise back_time).
        num_groups (int): Number of groups (bins) for "box" plots.
        include_front (bool): Whether to include frontend times in plots.
        use_dag_ops (bool): If True, use 'dag_operation' column instead of 'client_operation'.

    Outputs:
        A dictionary from operations to matplotlib figures.
    """
    # Ensure numeric
    df['total_time'] = pd.to_numeric(df['total_time'], errors='coerce')
    df['front_time'] = pd.to_numeric(df['front_time'], errors='coerce')
    df['back_time'] = pd.to_numeric(df['back_time'], errors='coerce')  

    df = df.fillna(0)
    df['front_time'] = df['front_time'] + df['back_time']

    final = {}
    op_field = 'dag_operation' if use_dag_ops else 'client_operation'

    for op_name_x in df[op_field].unique():
        op_name = op_name_x.strip()
        if op_name == "":
            continue

        subset = df[df[op_field].str.strip() == op_name]

        # Outlier removal
        if remove_outliers > 0 and len(subset) > 0:
            target_col = 'front_time' if include_front else 'back_time'
            cutoff = subset[target_col].quantile((100 - remove_outliers)/100)
            before_count = len(subset)
            subset = subset[subset[target_col] <= cutoff]
            removed_count = before_count - len(subset)
            print(f"[INFO] Removed {removed_count} entries ({remove_outliers}%) "
                  f"based on {target_col} outliers.")



        fig, ax = plt.subplots(figsize=(14, 6))   

        match strategy:
            case "scatter":
                ax.scatter(subset['front_id'], subset['back_time'], marker="o", 
                           label='Backend Time', 
                           alpha=0.7, color=color)

                if include_front:
                    ax.scatter(subset['front_id'], subset['front_time'], marker="o",
                               label='Frontend Time',
                               alpha=0.7, color=front_color)

            case "mean":
                back = subset.groupby('front_id')['back_time'].mean().reset_index()
                front = subset.groupby('front_id')['front_time'].mean().reset_index()

                ax.plot(back['front_id'], back['back_time'],
                        marker='o', linestyle='-',
                        label='Backend Time', 
                        alpha=0.7, color=color)
                if include_front:
                    ax.plot(front['front_id'], front['front_time'],
                            marker='o', linestyle='-',
                            label='Frontend Time',
                            alpha=0.7, color=front_color)

            case "box":
                # Sort by front_id first
                box_df = subset.copy().sort_values('front_id')

                min_val = 0
                max_val = box_df['front_id'].max() + 1
                bins = np.linspace(min_val, max_val, num_groups + 1)

                box_df['id_qbin'] = pd.cut(box_df['front_id'], bins=bins, include_lowest=True)
                box_df['id_qbin'] = box_df['id_qbin'].apply(
                    lambda x: f"{int(x.left)+1}–{int(x.right)}" if pd.notna(x) else None
                )

                # Group by bin (will now preserve order)
                grouped = [g['back_time'].values for _, g in box_df.groupby('id_qbin')]
                positions = np.arange(len(grouped)) + 1

                # Backend boxes
                ax.boxplot(
                    grouped,
                    positions=positions - 0.15,
                    widths=0.3,
                    patch_artist=True,
                    boxprops=dict(facecolor=color),
                    medianprops=dict(color='black'),
                    flierprops=dict(marker='o', markerfacecolor=color, markersize=6, linestyle='none')
                )

                if include_front:
                    grouped_front = [g['front_time'].values for _, g in box_df.groupby('id_qbin')]
                    ax.boxplot(
                        grouped_front,
                        positions=positions + 0.15,
                        widths=0.3,
                        patch_artist=True,
                        boxprops=dict(facecolor=front_color),
                        medianprops=dict(color='black'),
                        flierprops=dict(marker='o', markerfacecolor=front_color, markersize=6, linestyle='none')
                    )

                ax.set_xticks(positions)
                ax.set_xticklabels(box_df['id_qbin'].unique(), rotation=45)
                            
            case _:
                print("invalid strategy")
                return

        ax.set_title(f"{strategy.capitalize()} Plot for {OPERATION_TEXT[op_name]}")
        ax.set_xlabel("Operation Index (Grouped IDs)")
        ax.set_ylabel("Time (μs)")
        fig.tight_layout(pad=3)
        final[op_name] = fig

    return final        

def plot_comparison(
    dataframes,
    labels,
    group_percentage=0.05,
    strategy="box",
    colors=None,
    remove_outliers=0.5,
    merge_queries=True,
    use_dag_ops=True,
    include_front=False
):
    """
    Generates grouped plots for multiple datasets. Boxes are plotted side by side.
    Merges 'stateful_query' and 'stateless_query' if merge_queries=True.

    Args:
        dataframes (list of pd.DataFrame): List of DataFrames to plot.
        labels (list of str): Labels corresponding to each DataFrame.
        group_percentage (float): Fraction of IDs to include in each group.
        strategy (str): "box" or "line".
        colors (list of str): Colors for each DataFrame 
                              (default: ['yellow', 'steelblue', 'salmon', 'limegreen']).
        remove_outliers (float): Fraction of top outliers to remove (0-100).
        merge_queries (bool): If True, merge 'stateful_query' and 'stateless_query' into 'query'.
        use_dag_ops (bool): If True, use 'dag_operation' column, else 'client_operation'.

    Returns:
        dict: Dictionary mapping operation names to matplotlib Figure objects.
    """
    if len(dataframes) != len(labels):
        raise ValueError("Length of dataframes and labels must match.")

    operation_col = "dag_operation" if use_dag_ops else "client_operation"

    # --- Remove invalid front_id entries (<0) ---
    dataframes = [df[df['front_id'] >= 0].copy() for df in dataframes]

    # Default colors
    default_colors = ["yellow", "steelblue", "salmon", "limegreen"]
    if colors is None:
        colors = default_colors
    colors = [colors[i % len(colors)] for i in range(len(dataframes))]

    # Merge queries if needed
    if merge_queries:
        merged_dataframes = []
        for df in dataframes:
            df_copy = df.copy()
            df_copy.loc[df_copy[operation_col].isin(
                ['stateful_query', 'stateless_query']
            ), operation_col] = 'query'
            merged_dataframes.append(df_copy)
        dataframes = merged_dataframes

    # Collect all operations across all dataframes
    all_operations = sorted(
        set(op for df in dataframes for op in df[operation_col].unique())
    )

    # --- Global group sizing (done once) ---
    global_max_ids = max(
        len(df['front_id'].unique())
        for df in dataframes
    )
    group_size = max(1, int(global_max_ids * group_percentage))
    num_groups = global_max_ids // group_size + (1 if global_max_ids % group_size else 0)

    # Box width settings
    WIDTH = 0.25
    num_dfs = len(dataframes)
    offsets = np.linspace(-((num_dfs-1)/2)*WIDTH, ((num_dfs-1)/2)*WIDTH, num_dfs)

    # Add spacing: shift each group by +WIDTH relative to the baseline
    base_positions = np.arange(num_groups) * (1 + WIDTH) + 1  

    final = {}

    for operation in all_operations:
        fig, ax = plt.subplots(figsize=(14, 6))

        # Filter each dataframe by operation and handle outliers
        filtered_dfs = []
        for df in dataframes:
            filtered_df = df[df[operation_col] == operation].copy()

            if remove_outliers and 0 < remove_outliers < 100:
                threshold = filtered_df['back_time'].quantile(
                    (100 - remove_outliers) / 100
                )
                filtered_df = filtered_df[filtered_df['back_time'] <= threshold]

            filtered_dfs.append(filtered_df)

        # Collect proxy artists for legend
        proxy_artists = []

        for df_idx, filtered_df in enumerate(filtered_dfs):
            label = labels[df_idx]
            color = colors[df_idx]

            match strategy:
                case "box":
                    if not filtered_df.empty:
                        filtered_df['group'] = filtered_df['front_id'] // group_size
                        grouped = filtered_df.groupby('group')
                        grouped_df = [group for _, group in grouped]
                        data_to_plot = [group['back_time'] for group in grouped_df]

                        group_nums = sorted(grouped.groups.keys())
                        positions = base_positions[group_nums] + offsets[df_idx]

                        ax.boxplot(
                            data_to_plot,
                            patch_artist=True,
                            positions=positions,
                            boxprops=dict(facecolor=color),
                            widths=WIDTH,
                            medianprops=dict(color='black'),
                            flierprops=dict(
                                marker='o',
                                markerfacecolor=color,
                                markersize=6,
                                linestyle='none'
                            ),
                            manage_ticks=False
                        )

                        proxy_artists.append(mpatches.Patch(color=color, label=label))

                case "line":
                    if not filtered_df.empty:
                        avg_time_per_id = (
                            filtered_df.groupby('id')['time']
                            .mean()
                            .reset_index()
                            .sort_values(by="id")
                        )

                        ax.scatter(
                            avg_time_per_id['id'],
                            avg_time_per_id['time'],
                            color=color,
                            alpha=0.6,
                            label=label
                        )

                        coefficients = np.polyfit(
                            avg_time_per_id['id'],
                            avg_time_per_id['time'],
                            deg=1
                        )
                        poly_eq = np.poly1d(coefficients)

                        x_values = np.linspace(
                            avg_time_per_id['id'].min(),
                            avg_time_per_id['id'].max(),
                            100
                        )
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
        ax.set_title(f"{strategy.capitalize()} Plot for {operation}")
        fig.tight_layout(pad=3)

        final[operation] = fig

    return final
