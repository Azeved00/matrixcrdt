import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import graph_utils
import sys

def plot_regression(datasets, v):
    """
    Plots a scatter plot of 'time' vs 'id' for two datasets with regression lines.
    
    Parameters:
    - sets_of_dataframes: A tuple of two lists, each containing DataFrames.
    - labels: Tuple containing labels for the two datasets.
    """

    all_operations = set()
    for dataset in datasets:
        for df in dataset["dataframes"]:
            if isinstance(df, pd.DataFrame) and 'operation' in df.columns:
                all_operations.update(df["operation"].unique())

    all_operations = sorted(all_operations)

    for operation in all_operations:
        plt.figure(figsize=(15, 6))

        for idx, dataset in enumerate(datasets):
            label, dataframes, color = dataset["label"], dataset["dataframes"], dataset["color"]
            combined_df = pd.concat(dataframes, ignore_index=True)
            combined_df = combined_df[combined_df['operation'] == operation].copy()
            
            # Ensure required columns exist
            if 'id' not in combined_df.columns or 'time' not in combined_df.columns:
                print("Error: DataFrames must contain 'id' and 'time' columns.")
                return

            # Compute the mean 'time' per unique ID
            avg_time_per_id = combined_df.groupby('id')['time'].mean().reset_index()

            # Sort by ID
            avg_time_per_id = avg_time_per_id.sort_values(by="id")

            # Scatter plot
            plt.scatter(avg_time_per_id['id'], avg_time_per_id['time'], color=color, alpha=0.6, label=f"{label} Data Points")

            # Fit a linear regression model
            coefficients = np.polyfit(avg_time_per_id['id'], avg_time_per_id['time'], deg=1)
            poly_eq = np.poly1d(coefficients)

            # Generate regression line
            x_values = np.linspace(avg_time_per_id['id'].min(), avg_time_per_id['id'].max(), 100)
            y_values = poly_eq(x_values)

            # Plot regression line
            plt.plot(x_values, y_values, color=color, linestyle='-', linewidth=2, label=f"{label} Regression Line")

        plt.xlabel("Operation Index")
        plt.ylabel("Time (\u03BCs)")
        plt.title(f"Linear regression for Operation: {operation}")
        plt.legend(loc="upper left")
        
        plt.grid(True, linestyle="--", alpha=0.7)
        plt.tight_layout()
        #plt.show()
        plt.savefig(f"line_bench_{v}_{operation}.png")


version = int(sys.argv[1]) if len(sys.argv) > 1 else 2

dataset_sources = graph_utils.get_dataset_sources(version)

datasets = []
for source in dataset_sources:
    folder = source["folder"]
    files = graph_utils.list_files_in_folder(folder)
    dfs = [df for file in files if (df := graph_utils.load_csv(file)) is not None]
    datasets.append({"dataframes": dfs, "label": source["label"], "color": source["color"]})

plot_regression(datasets, version)
