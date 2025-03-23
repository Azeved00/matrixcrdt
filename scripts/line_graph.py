import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

def list_files_in_folder(folder_path):
    """Returns a list of files in the given folder."""
    try:
        return [os.path.join(folder_path, f) for f in os.listdir(folder_path) if os.path.isfile(os.path.join(folder_path, f))]
    except FileNotFoundError:
        print(f"Error: The folder '{folder_path}' does not exist.")
        return []
    except PermissionError:
        print(f"Error: Permission denied for folder '{folder_path}'.")
        return []

def load_csv(file_path):
    """Loads a CSV file into a Pandas DataFrame. If empty or doesn't exist, returns None."""
    if not os.path.exists(file_path):
        print(f"Error: The file '{file_path}' does not exist.")
        return None
    
    df = pd.read_csv(file_path)
    
    if df.empty:
        print(f"Info: The file '{file_path}' is empty. Ignoring it.")
        return None
    
    return df

def plot_regression(sets_of_dataframes, labels=("Dataset 1", "Dataset 2")):
    """
    Plots a scatter plot of 'time' vs 'id' for two datasets with regression lines.
    
    Parameters:
    - sets_of_dataframes: A tuple of two lists, each containing DataFrames.
    - labels: Tuple containing labels for the two datasets.
    """
    colors = ['lightblue', 'lightcoral']

    plt.figure(figsize=(15, 6))

    for dataframes, color, label in zip(sets_of_dataframes, colors, labels):
        combined_df = pd.concat(dataframes, ignore_index=True)
        
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
    plt.legend(loc="upper left")
    
    plt.grid(True, linestyle="--", alpha=0.7)
    plt.tight_layout()
    plt.show()

# Load and process the baseline dataset
base_files = list_files_in_folder("./base2/")
base_dfs = [load_csv(file) for file in base_files if load_csv(file) is not None]

# Load and process the benchmark dataset
bench_files = list_files_in_folder("./bench2/")
bench_dfs = [load_csv(file) for file in bench_files if load_csv(file) is not None]

# Plot the averaged results
plot_regression((base_dfs, bench_dfs), labels=("Baseline", "Benchmark"))
