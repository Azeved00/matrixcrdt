import pandas as pd
import os
import sys
import argparse

from .process_logs import merge_files 
from .validation import validate_df
from .macro_plot import plot_graphs, plot_comparison

def merge(name, input_path, save=False):
    df1 = []

    df1 = merge_files(input_path)
    print(df1.head())

    validation_errors = validate_df(df1)
    if validation_errors:
        for err in validation_errors:
            print(err)
        raise ValueError(f"Final CSV is invalid( has errors {len(validation_errors)})")

    if save:
        path = f'{name}_final.csv'
        df1.to_csv(path, index=False)
        print(f"CSV saved to {path}.")

    return df1


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

    if len(args.names) == 1:
        df1 = merge(args.names[0], f"logs/macro/{args.names[0]}/", True)
        print("Plotting")
        plot_graphs(df1, args.names[0],
                strategy=args.strategy, show=args.display, warmup=args.warmup)
    else:
        df1 = merge(args.names[0],f"logs/macro/{args.names[0]}/", True)
        df2 = merge(args.name[0], f"logs/macro/{args.names[1]}/", True)

        print("Plotting comparison")
        path=f"{name1}x{name2}"
        plot_comparison(df1,args.names[0],df2,args.names[1],path)
