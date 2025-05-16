import os
import pandas as pd

def get_dataset_sources(version):
    """Returns dataset folder paths based on the selected version (1, 2, or 3)."""
    if version not in {1, 2, 3}:
        raise ValueError("Invalid version! Choose 1, 2, or 3.")

    return [
            {"folder": f"./base{version}/", "label": "Baseline", "color": "lightblue", "secondary":"deepskyblue"},
            {"folder": f"./bench{version}/", "label": "Benchmark", "color": "lightcoral", "secondary":"firebrick"},
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

