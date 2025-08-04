import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from .graph_utils import *
import sys

def plot_regression_single(df, n=5, save_fig=False, show_plot=True):
    """
    Plots regression for a single dataframe.

    Args:
        df (pd.DataFrame): Combined DataFrame with 'id', 'time', and 'operation'.
        n (int): X-axis multiplier for 'query' operations.
        save_fig (bool): Whether to save the figure.
        show_plot (bool): Whether to display the plot.
    """
    if 'id' not in df.columns or 'time' not in df.columns or 'operation' not in df.columns:
        print("Error: DataFrame must contain 'id', 'time', and 'operation' columns.")
        return

    operations = sorted(df["operation"].unique())

    for operation in operations:
        plt.figure(figsize=(15, 6))
        op_df = df[df["operation"] == operation].copy()

        avg_time_per_id = op_df.groupby('id')['time'].mean().reset_index()
        avg_time_per_id = avg_time_per_id.sort_values(by="id")

        plt.scatter(avg_time_per_id['id'], avg_time_per_id['time'], color="lightblue", alpha=0.6, label="Data Points")

        coefficients = np.polyfit(avg_time_per_id['id'], avg_time_per_id['time'], deg=1)
        poly_eq = np.poly1d(coefficients)

        x_values = np.linspace(avg_time_per_id['id'].min(), avg_time_per_id['id'].max(), 100)
        y_values = poly_eq(x_values)

        plt.plot(x_values, y_values, color="steelblue", linewidth=2, label="Regression Line")

        plt.xlabel("Operation Index")
        plt.ylabel("Time (\u03BCs)")
        plt.title(f"Linear regression for Operation: {operation}")
        plt.legend(loc="upper left")
        plt.grid(True, linestyle="--", alpha=0.7)

        if operation == "query":
            xticks = plt.xticks()[0][1:-1]
            plt.xticks(xticks, labels=[f"{int(tick * n)}" for tick in xticks])

        plt.tight_layout()

        if save_fig:
            plt.savefig(f"line_single_{operation}.png")
        if show_plot:
            plt.show()

def plot_regression_dual(dataframes1, label1, dataframes2, label2, n=5, save_fig=False, show_plot=True):
    all_operations = set()
    for df in dataframes1 + dataframes2:
        if isinstance(df, pd.DataFrame) and 'operation' in df.columns:
            all_operations.update(df["operation"].unique())

    all_operations = sorted(all_operations)

    for operation in all_operations:
        plt.figure(figsize=(15, 6))

        for dataframes, label, color, secondary in [
            (dataframes1, label1, "lightblue", "blue"),
            (dataframes2, label2, "lightcoral", "darkred")
        ]:
            combined_df = pd.concat(dataframes, ignore_index=True)
            combined_df = combined_df[combined_df['operation'] == operation].copy()

            if 'id' not in combined_df.columns or 'time' not in combined_df.columns:
                print("Error: DataFrames must contain 'id' and 'time' columns.")
                return

            avg_time_per_id = combined_df.groupby('id')['time'].mean().reset_index()
            avg_time_per_id = avg_time_per_id.sort_values(by="id")

            plt.scatter(avg_time_per_id['id'], avg_time_per_id['time'], color=color, alpha=0.6, label=f"{label} Data Points")

            coefficients = np.polyfit(avg_time_per_id['id'], avg_time_per_id['time'], deg=1)
            poly_eq = np.poly1d(coefficients)
            x_values = np.linspace(avg_time_per_id['id'].min(), avg_time_per_id['id'].max(), 100)
            y_values = poly_eq(x_values)

            plt.plot(x_values, y_values, color=secondary, linestyle='-', linewidth=2, label=f"{label} Regression Line")

        plt.xlabel("Operation Index")
        plt.ylabel("Time (\u03BCs)")
        plt.title(f"Linear regression for Operation: {operation}")
        plt.legend(loc="upper left")
        plt.grid(True, linestyle="--", alpha=0.7)
        if operation == "query":
            xticks = plt.xticks()[0][1:-1]
            plt.xticks(xticks, labels=[f"{int(tick * n)}" for tick in xticks])
        plt.tight_layout()

        if save_fig:
            plt.savefig(f"line_pair_{operation}.png")
        if show_plot:
            plt.show()

if __name__ == "__main__":
    version = int(sys.argv[1]) if len(sys.argv) > 1 else 2
    dataset_sources = get_dataset_sources(version)

    datasets = []
    for source in dataset_sources:
        folder = source["folder"]
        files =list_files_in_folder(folder)
        dfs = [df for file in files if (df := load_csv(file)) is not None]
        source["dataframes"] = dfs
        datasets.append(source)

    plot_regression_single(datasets, version, n=5, save_fig=False, show_plot=True)

