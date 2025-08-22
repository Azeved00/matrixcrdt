import sys
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import textwrap
from .graph_utils import *

WIDTH = 0.3

def plot_single(dataframe, group_percentage, sample_n, strategy, color="lightblue"):
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
        group_size = max(1, int(total_ids * group_percentage))

        filtered_df['group'] = filtered_df['id'] // group_size
        grouped_df = [group for _, group in filtered_df.groupby('group')]

        data_to_plot = [group['time'] for group in grouped_df]
        base_positions = np.arange(len(grouped_df)) + 1


        match strategy:
            case "box":
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

                if operation == "stateful_query" or operation == "stateless_query":
                    xticks = plt.xticks()[0][1:-1]
                    ax.set_xticks(xticks, labels=[f"{int(tick * sample_n)}" for tick in xticks])

                ax.legend()

        ax.set_xlabel("Operation Index (Grouped IDs)")
        ax.set_ylabel("Time (μs)")
        ax.set_title(f"{strategy} Plot for Operation: {operation}")
        fig.tight_layout(pad=3)

        final[operation] = fig   

    return final

