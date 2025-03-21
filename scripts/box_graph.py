import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

def list_files_in_folder(folder_path):
    """Returns a list of files in the given folder."""
    try:
        files = [os.path.join(folder_path, f) for f in os.listdir(folder_path) if os.path.isfile(os.path.join(folder_path, f))]
        return files
    except FileNotFoundError:
        return f"Error: The folder '{folder_path}' does not exist."
    except PermissionError:
        return f"Error: Permission denied for folder '{folder_path}'."

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

def plot_boxplot(sets_of_dataframes, group_size=50, labels=("Dataset 1", "Dataset 2")):
    """
    Plots two sets of aggregated DataFrames as box plots grouped by operation.
    
    Parameters:
    - sets_of_dataframes: A tuple of two lists, each containing DataFrames.
    - group_size: Number of unique IDs to aggregate into one group.
    - labels: Tuple containing labels for the two data sets.
    """
    colors = ['lightblue', 'lightcoral']  # Colors for the two sets

    # Get all unique operations
    all_operations = set()
    for dataframes in sets_of_dataframes:
        for df in dataframes:
            if 'operation' in df.columns:
                all_operations.update(df['operation'].unique())

    all_operations = sorted(all_operations)

    for operation in all_operations:
        plt.figure(figsize=(20, 8))
        all_group_labels = []

        for idx, (dataframes, color, label) in enumerate(zip(sets_of_dataframes, colors, labels)):
            combined_df = pd.concat(dataframes, ignore_index=True)

            # Ensure required columns exist
            if 'id' not in combined_df.columns or 'time' not in combined_df.columns or 'operation' not in combined_df.columns:
                print("Error: DataFrames must contain 'id', 'time', and 'operation' columns.")
                return

            # Filter by the current operation
            operation_df = combined_df[combined_df['operation'] == operation]

            # Sort unique IDs for consistency
            unique_ids = sorted(operation_df['id'].unique())

            grouped_boxes = []
            group_labels = []

            for i in range(0, len(unique_ids), group_size):
                group_ids = unique_ids[i:i+group_size]
                group_data = operation_df[operation_df['id'].isin(group_ids)]['time'].values
                grouped_boxes.append(group_data)
                if idx == 0:  # Store labels only once
                    group_labels.append(f"{group_ids[0]} - {group_ids[-1]}")

            # Store labels from the first dataset
            if idx == 0:
                all_group_labels = group_labels

            # Create box plots with a slight x-offset to avoid overlap
            positions = np.arange(1, len(grouped_boxes) + 1) + (idx * 0.3)
            plt.boxplot(grouped_boxes, positions=positions, patch_artist=True,
                        boxprops=dict(facecolor=color), widths=0.3,
                        medianprops=dict(color='black'),
                        flierprops=dict(marker='o', markerfacecolor=color, markersize=6, linestyle='none'))

        # Set x-axis labels for the aggregated groups
        plt.xticks(np.arange(1, len(all_group_labels) + 1), all_group_labels, rotation=45, fontsize=10)
        plt.xlabel("Operation Index (groups of 5)")
        plt.ylabel("Time (\u03BCs)")
        plt.title(f"Comparison of Two Data Sets - Box Plot by Aggregated ID Groups for Operation: {operation}")
        
        # Create a custom legend using proxy patches
        blue_patch = mpatches.Patch(color=colors[0], label=labels[0])
        red_patch = mpatches.Patch(color=colors[1], label=labels[1])
        plt.legend(handles=[blue_patch, red_patch], loc="upper left")
        
        plt.tight_layout()
        plt.show()

base_files = list_files_in_folder("./base1/")
base_dfs = []
for file in base_files:
    df = load_csv(file)

    if df is not None:
        print("CSV loaded successfully!")
        print(df.head())  # Show the first few rows
        base_dfs.append(df)
    else:
        print("No data to process.")

bench_files = list_files_in_folder("./bench1/")
bench_dfs = []
for file in bench_files:
    df = load_csv(file)

    if df is not None:
        print("CSV loaded successfully!")
        print(df.head())  # Show the first few rows
        bench_dfs.append(df)
    else:
        print("No data to process.")

plot_boxplot((base_dfs, bench_dfs), 10, ("Baseline", "Benchmark"))
