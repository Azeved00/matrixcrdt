import pandas as pd
import os
import sys
import argparse

from process_logs import merge_files 
from validation import validate_df
from macro_plot import plot_graphs, plot_comparisson

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Process and plot macro benchmarks log CSVs.")
    parser.add_argument("names", nargs="+", 
                        help="One or two names for the log directories")
    parser.add_argument("-s", "--strategy", type=str, default="", 
                        help="Strategy string passed to plotting functions")
    parser.add_argument("-d", "--display", action="store_true", 
                        help="Display plots instead of saving them")
    parser.add_argument("-w", "--warmup", type=int, default=0, 
                        help="Ingnore N rows of the dataset")

    args = parser.parse_args()

    if len(args.names) > 2 or len(args.names) < 1:
        parser.error("Please provide one or two log directory names")

    df1 = []
    df2 = []

    if len(args.names) >= 1:
        name = args.names[0]
        input_path = f"logs/macro/{name}/"
        df1 = merge_files(input_path)
        print(df1.head())

        validation_errors = validate_df(df1)
        if validation_errors:
            for err in validation_errors:
                print(err)
            print(f"Finished with {len(validation_errors)} errors")
            sys.exit(1)
        else:
            print("CSV passed all validation checks.")

        path = f'{name}_final.csv'
        df1.to_csv(path, index=False)
        print(f"CSV saved to {path}.")

    if len(args.names) == 2:
        name = args.names[1]
        input_path = f"logs/macro/{name}/"
        df2 = merge_files(input_path)
        print(df2.head())

        validation_errors = validate_df(df2)
        if validation_errors:
            for err in validation_errors:
                print(err)
            print(f"Finished with {len(validation_errors)} errors")
            sys.exit(1)
        else:
            print("CSV passed all validation checks.")

        path = f'{name}_final.csv'
        df2.to_csv(path, index=False)
        print(f"CSV saved to {path}.")

    if len(args.names) == 1:
        print("Plotting")
        plot_graphs(df1, args.names[0],
                    strategy=args.strategy, show=args.display, warmup=args.warmup)
    else:
        print("Plotting comparisson")
        names = sorted(args)
        path=f"{names[0]}x{names[1]}"
        plot_comparisson(df1,args[0],df2,args[1],path )
