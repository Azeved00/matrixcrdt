import pandas as pd
import os
import sys
import argparse

from .macro_plot import plot_graphs, plot_comparison
from . import merge_and_validate as merge

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
        df1 = merge(f"logs/macro/{args.names[0]}/")

        print("Plotting")
        plot_graphs(df1, strategy=args.strategy, warmup=args.warmup)

        for op,plt in plts.items():
            if args.display:
                plt.show()
            else:
                plt.savefig(f'plots/{place}-{op}.png')
            plt.close()
    else:
        df1 = merge(f"logs/macro/{args.names[0]}/")
        print(df1.head())
        df2 = merge(f"logs/macro/{args.names[1]}/")
        print(df2.head())

        print("Plotting comparison")
        path=f"{name1}x{name2}"

        plts = plot_comparison(df1,args.names[0],df2,args.names[1])
        for op,plt in plts.items():
            if args.display:
                plt.show()
            else:
                plt.savefig(f'plots/{place}-{op}.png')
            plt.close()

