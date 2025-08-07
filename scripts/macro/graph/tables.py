import pandas as pd
import numpy as np
import argparse
from scipy import stats

def make_back_summary(df, n_parts=4):
    from scipy import stats
    import numpy as np

    df = df[df['front_id'] != -1]
    df_group = df.groupby('client_operation')

    summary = []

    for group_name, dfg in df_group:
        parts = np.array_split(dfg, n_parts)
        row = {'client_operation': group_name}

        for i, part_df in enumerate(parts):
            data = part_df['back_time'].values
            mean = np.mean(data)
            sem = stats.sem(data)
            ci = stats.t.interval(0.95, len(data)-1, loc=mean, scale=sem) if len(data) > 1 else (mean, mean)

            row[f'Part{i+1}_Mean'] = mean
            row[f'Part{i+1}_CI'] = f"[{ci[0]:.2f}, {ci[1]:.2f}]"
            row[f'Part{i+1}_SEM'] = sem

        summary.append(row)

    return pd.DataFrame(summary)

def make_latex_table(df_summary, n_parts):
    header = "\\begin{tabular}{" + "l" + "|ccc" * n_parts + "}\n"
    header += "\\hline\n"

    # First row: merged column group headers
    header += "Client Operation"
    for i in range(n_parts):
        header += f" & \\multicolumn{{3}}{{c}}{{Part {i+1}}}"
    header += " \\\\\n"

    # Second row: sub-headers
    header += " "
    for i in range(n_parts):
        header += " & Mean & 95\\% CI & SEM"
    header += " \\\\\n\\hline\n"

    # Table rows
    body = ""
    for _, row in df_summary.iterrows():
        body += f"{row['client_operation'].replace("_"," ")}"
        for i in range(n_parts):
            mean = f"{row[f'Part{i+1}_Mean']:.2f}"
            ci = row[f'Part{i+1}_CI']
            sem = f"{row[f'Part{i+1}_SEM']:.2f}"
            body += f" & {mean} & {ci} & {sem}"
        body += " \\\\\n"

    footer = "\\hline\n\\end{tabular}"

    return header + body + footer

def print_summary(df_summary, n_parts):
    col_headers = ["Client Operation"]
    for i in range(n_parts):
        col_headers.extend([f"P{i+1} Mean", f"P{i+1} 95% CI", f"P{i+1} SEM"])

    # Build rows
    rows = []
    for _, row in df_summary.iterrows():
        row_data = [str(row['client_operation'])]
        for i in range(n_parts):
            row_data.append(f"{row[f'Part{i+1}_Mean']:.2f}")
            row_data.append(str(row[f'Part{i+1}_CI']))
            row_data.append(f"{row[f'Part{i+1}_SEM']:.2f}")
        rows.append(row_data)

    # Compute column widths
    col_widths = [max(len(str(cell)) for cell in [header] + [row[i] for row in rows]) for i, header in enumerate(col_headers)]

    # Helper: format a row
    def format_row(row):
        return "| " + " | ".join(f"{cell:<{col_widths[i]}}" for i, cell in enumerate(row)) + " |"

    # Print header
    print("-" * (sum(col_widths) + 3 * len(col_headers) + 1))
    print(format_row(col_headers))
    print("-" * (sum(col_widths) + 3 * len(col_headers) + 1))

    # Print rows
    for row in rows:
        print(format_row(row))
    print("-" * (sum(col_widths) + 3 * len(col_headers) + 1))


def make_table(df, parts=4, include_front=True, latex=False):
    """
    Creates a summary table with n parts, each part having the columns 
    mean, 95% interval and standard deviation. 
    The table has a row per operation.

    Inputs:
        - parts, the number of parts of the table, defaults to 4,
        - include_front, weather to include the frontend data, defaults to True,
        - latex, weather the table should be in latex

    Outputs: The table to standard output
    """
    summary = make_back_summary(df, parts)

    if latex:
        latex_table = make_latex_table(summary, parts)
        print(latex_table)
    else:
        print_summary(summary, parts)

def make_summary(df, include_front=True):
    summary = make_back_summary(df, parts)
    print(summary)

