import pandas as pd
import os
import sys

from process_logs import merge_files 
from validation import validate_df
from macro_plot import plot_graphs, plot_comparisson

if __name__ == "__main__":
    if len(sys.argv) > 3 or len(sys.argv) <= 1:
        print(f"Usage: {sys.argv[0]} <name1> <name2?>")
        sys.exit(1)

    args = sys.argv[1:]
    
    df1 = []
    df2 = []
    if len(args) >= 1:
        name = args[0]
        input_path = "logs/macro/" +  name + "/"
        df1 = merge_files(input_path)
        print(df1)

        validation_errors = validate_df(df1)

        if validation_errors:
            for err in validation_errors[0:]:
                print(err)
            print(f"Finished with {len(validation_errors)} errors")
            sys.exit(1)
        else:
            print("CSV passed all validation checks.")

        path =f'{name}_final.csv'
        df1.to_csv(path, index=False)
        print(f"CSV saved to {path}.")


    if len(args) >= 2:
        name = args[1]
        input_path = "logs/macro/" +  name + "/"
        df2= merge_files(input_path)
        print(df2.head())

        validation_errors = validate_df(df2)

        if validation_errors:
            for err in validation_errors[0:]:
                print(err)
            print(f"Finished with {len(validation_errors)} errors")
            sys.exit(1)
        else:
            print("CSV passed all validation checks.")

        path = f'{name}_final.csv'
        df2.to_csv(path, index=False)
        print(f"CSV saved to {path}.")


    if len(args) == 1:
        plot_graphs(df1, args[0])
        print("Operation Plots saved in 'plots' directory.")
    else:
        names = sorted(args)
        path=f"{names[0]}x{names[1]}"
        plot_comparisson(df1,args[0],df2,args[1],path )
        print(f"Comparisson Plots saved in 'plots/{path}' directory.")
