import pandas as pd
import os
import sys

from process_logs import merge_files 
from validation import validate_df
from macro_plot import plot_graphs 

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <path>")
        sys.exit(1)

    input_path = sys.argv[1]
    merged_df = merge_files(input_path)
    print(merged_df.head())
    merged_df.to_csv('final.csv', index=False)

    validation_errors = validate_df(merged_df)

    if validation_errors:
        for err in validation_errors[0:]:
            print(err)
        print(f"Finished with {len(validation_errors)} errors")
        sys.exit(1)
    else:
        print("CSV passed all validation checks.")

    plot_graphs(merged_df)
