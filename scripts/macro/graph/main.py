import pandas as pd
import os
import sys
import argparse

from .process_logs import merge_files 
from .validation import validate_df
from .macro_plot import plot_graphs, plot_comparisson

def graph_single(name, input_path, strategy, display, warmup ):
    df1 = []

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

    print("Plotting")
    plot_graphs(df1, name,
                strategy=strategy, show=display, warmup=warmup)

def graph_comparison(name1, input_path1, name2, input_path2,
                      strategy, display, warmup):
    df1 = []
    df2 = []
    
    df1 = merge_files(input_path1)
    print(df1.head())

    validation_errors = validate_df(df1)
    if validation_errors:
        for err in validation_errors:
            print(err)
        print(f"Finished with {len(validation_errors)} errors")
        sys.exit(1)
    else:
        print("CSV passed all validation checks.")

    path = f'{name1}_final.csv'
    df1.to_csv(path, index=False)
    print(f"CSV saved to {path}.")

    df2 = merge_files(input_path2)
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

    print("Plotting comparisson")
    path=f"{name1}x{name2}"
    plot_comparisson(df1,names1,df2,names2,path)

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
        graph_single(
                name= args.names[0],
                input_path= f"logs/macro/{args.names[0]}/",
                strategy=args.strategy,
                display=args.display,
                warmup=args.warmup)
    else:
        graph_comparisson(
                name1= args.names[0],
                input_path1= f"logs/macro/{args.names[0]}/",
                name2= args.names[1],
                input_path2= f"logs/macro/{args.names[1]}/",
                strategy=args.strategy,
                display=args.display,
                warmup=args.warmup)
