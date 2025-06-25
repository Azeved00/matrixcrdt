import pandas as pd
import argparse

def summarize_operations(csv_path, include_front=True):
    """
    Summarizes operation statistics from a CSV file.

    Parameters:
        csv_path (str): Path to the CSV file.
        include_front (bool): If True, includes front_time statistics.

    Returns:
        pd.DataFrame: Summary table with average, 95th, and 99th percentiles.
    """
    df = pd.read_csv(csv_path)
    df = df[df['front_id'] != -1]

    # Validate that each operation maps to a single dag_operation
    op_to_dag = df[['client_operation', 'dag_operation']].drop_duplicates()
    if op_to_dag.duplicated('client_operation').any():
        print( op_to_dag.duplicated('client_operation'))
        raise ValueError("Each operation must map to a single dag_operation.")

    def compute_stats(group):
        stats = {
            'back_avg': group['back_time'].mean(),
            'back_p95': group['back_time'].quantile(0.95),
            'back_p99': group['back_time'].quantile(0.99),
        }
        if include_front:
            stats.update({
                'front_avg': group['front_time'].mean(),
                'front_p95': group['front_time'].quantile(0.95),
                'front_p99': group['front_time'].quantile(0.99),
            })
        return pd.Series(stats)

    summary = df.groupby('client_operation').apply(lambda g: compute_stats(g.drop(columns='client_operation'))).reset_index()
    summary = summary.merge(op_to_dag, on='client_operation')

    # Optional: reorder columns
    cols = ['client_operation', 'dag_operation'] + [c for c in summary.columns if c not in ('client_operation', 'dag_operation')]
    summary = summary[cols]

    return summary

def make_table(csv_path, include_front=True, latex=False):
    summary = summarize_operations(csv_path, include_front=include_front)

    if latex:
        latex_table = summary.to_latex(index=False, float_format="%.2f")
        print(latex_table)
    else:
        print(summary)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Summarize operation stats from a CSV file.")
    parser.add_argument("csv_file", help="Path to the input CSV file.")
    parser.add_argument(
        "-f", "--no-front",
        action="store_true",
        help="Exclude front_time statistics from the summary."
    )

    parser.add_argument(
        "--latex",
        action="store_true",
        help="Output table as latex."
    )

    args = parser.parse_args()
    make_table(args.csv_file, include_front= not args.no_front, latex=args.latex)
