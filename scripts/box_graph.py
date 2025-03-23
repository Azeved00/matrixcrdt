import os
import sys
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

def get_dataset_sources(version):
    """Returns dataset folder paths based on the selected version (1, 2, or 3)."""
    if version not in {1, 2, 3}:
        raise ValueError("Invalid version! Choose 1, 2, or 3.")

    return [
        {"folder": f"./base{version}/", "label": "Baseline", "color": "lightblue"},
        {"folder": f"./bench{version}/", "label": "Benchmark", "color": "lightcoral"},
    ]

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

def plot_boxplot(datasets, operation_group_sizes=None):
    """
    Plots multiple sets of aggregated DataFrames as box plots grouped by operation.
    
    Parameters:
    - datasets: List of dictionaries, each containing:
        - 'label': Name of the dataset
        - 'dataframes': List of Pandas DataFrames
        - 'color': Color for the boxplots
    - operation_group_sizes: Dictionary mapping each operation to its specific group size (optional).
      If not provided, defaults to a group size of 50 for all operations.
    """
    
    # Use a set to collect all unique operations
    all_operations = set()
    for dataset in datasets:
        for df in dataset["dataframes"]:
            if isinstance(df, pd.DataFrame) and 'operation' in df.columns:
                all_operations.update(df["operation"].unique())

    all_operations = sorted(all_operations)  # Convert back to a sorted list

    # Default group size if not provided for a specific operation
    if operation_group_sizes is None:
        operation_group_sizes = {operation: 50 for operation in all_operations}

    for operation in all_operations:
        group_size = operation_group_sizes.get(operation, 50)  # Use specific group size for the operation
        
        plt.figure(figsize=(20, 8))
        all_group_labels = []

        for idx, dataset in enumerate(datasets):
            label, dataframes, color = dataset["label"], dataset["dataframes"], dataset["color"]
            combined_df = pd.concat(dataframes, ignore_index=True)

            # Ensure required columns exist
            if 'id' not in combined_df.columns or 'time' not in combined_df.columns or 'operation' not in combined_df.columns:
                print(f"Error: DataFrames in '{label}' must contain 'id', 'time', and 'operation' columns.")
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
        plt.title(f"Comparison of Data Sets - Box Plot by Aggregated ID Groups for Operation: {operation}")
        
        # Create a custom legend using proxy patches
        legend_patches = [mpatches.Patch(color=dataset["color"], label=dataset["label"]) for dataset in datasets]
        plt.legend(handles=legend_patches, loc="upper left")
        
        plt.tight_layout()
        plt.show()

version = int(sys.argv[1]) if len(sys.argv) > 1 else 2

dataset_sources = get_dataset_sources(version)

datasets = []
for source in dataset_sources:
    folder = source["folder"]
    files = list_files_in_folder(folder)
    dfs = [df for file in files if (df := load_csv(file)) is not None]
    datasets.append({"dataframes": dfs, "label": source["label"], "color": source["color"]})

plot_boxplot(datasets, operation_group_sizes = {
    'apply': 20,
    'query': 5,
})
