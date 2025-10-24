import pandas as pd
import numpy as np
import argparse

OP_MAP = {
    "apply": "Update",
    "stateless_query": "Query",
    "stateful_query": "Query",
}

def make_box_summary(df, B=20, R=5, use_dag_ops=True, thread_id_col="front_id"):
    """
    Summarize back_time into B boxes based on thread IDs, pick R evenly spaced boxes,
    return a DataFrame where rows = boxes, columns = operations.
    Each cell contains a dict with keys Q1, Median, Q3.
    """
    target_column = "dag_operation" if use_dag_ops else "client_operation"
    df = df[df['front_id'] != -1]
    df_group = df.groupby(target_column)

    # Which boxes to pick (1-based indices)
    chosen_boxes = np.linspace(1, B, R, dtype=int)

    # Collect ranges to display
    ranges = {}

    # Initialize table
    data = {"Box": []}

    for b in chosen_boxes:
        data["Box"].append(f"Box {b}")  # placeholder, will add range later

    for group_name, dfg in df_group:
        sorted_threads = np.sort(dfg[thread_id_col].unique())
        box_edges = np.linspace(0, len(sorted_threads), B+1, dtype=int)

        cells = []
        for b in chosen_boxes:
            box_threads = sorted_threads[box_edges[b-1]:box_edges[b]]
            if len(box_threads) > 0:
                ranges[f"Box {b}"] = f"[{box_threads[0] + 1}–{box_threads[-1] + 1}]"
            else:
                ranges[f"Box {b}"] = "[empty]"

            box_rows = dfg[dfg[thread_id_col].isin(box_threads)]
            values = box_rows["back_time"].values

            if len(values) == 0:
                cells.append({"Q1": None, "Median": None, "Q3": None})
            else:
                cells.append({
                    "Q1": np.percentile(values, 25),
                    "Median": np.percentile(values, 50),
                    "Q3": np.percentile(values, 75),
                })
        data[group_name] = cells

    # Build dataframe
    df_summary = pd.DataFrame(data)

    # Add ranges to box labels
    df_summary["Box"] = df_summary["Box"].apply(lambda x: f"{x} {ranges[x]}")


    return df_summary

# ---------------------------
# Console Pretty Print
# ---------------------------
def print_box_summary(df_summary):
    col_headers = list(df_summary.columns)
    rows = df_summary.values.tolist()

    # Compute column widths
    col_widths = [max(len(str(cell)) for cell in [header] + [row[i] for row in rows]) 
                  for i, header in enumerate(col_headers)]

    def format_row(row):
        return "| " + " | ".join(f"{cell:<{col_widths[i]}}" for i, cell in enumerate(row)) + " |"

    # Print table
    print("-" * (sum(col_widths) + 3 * len(col_headers) + 1))
    print(format_row(col_headers))
    print("-" * (sum(col_widths) + 3 * len(col_headers) + 1))
    for row in rows:
        print(format_row(row))
    print("-" * (sum(col_widths) + 3 * len(col_headers) + 1))


def make_latex_box_table(df_summary):
    """
    Convert flipped box summary DataFrame into LaTeX tabular.
    Rows = boxes (with ranges), Columns = operations (with Q1, Median, Q3).
    """
    ops = [col for col in df_summary.columns if col != "Box"]

    # Header
    header = "\\begin{tabular}{l" + "ccc" * len(ops) + "}\n"
    header += "\\hline\n"

    # First header row: operation names
    header += "Box"
    for op in ops:
        header += f" & \\multicolumn{{3}}{{c}}{{{OP_MAP.get(op, op)}}}"
    header += " \\\\\n"

    # Second header row: Q1 Median Q3
    header += " "
    for _ in ops:
        header += " & Q1 & Median & Q3"
    header += " \\\\\n\\hline\n"

    # Rows
    body = ""
    for _, row in df_summary.iterrows():
        body += row["Box"]
        for op in ops:
            cell = row[op]
            if cell is None or cell["Median"] is None:
                body += " & NA & NA & NA"
            else:
                body += (
                    f" & {cell['Q1']:.0f} & {cell['Median']:.0f} & {cell['Q3']:.0f}"
                )
        body += " \\\\\n"

    footer = "\\hline\n\\end{tabular}"

    return header + body + footer

# ---------------------------
# Main Function
# ---------------------------
def make_box_table(df, B=20, R=5, latex=False, use_dag_ops=True):
    """
    Creates a summary table with R evenly spaced boxes from B,
    flipped orientation: rows = boxes, columns = operations.
    """
    df_summary = make_box_summary(df, B, R)

    if latex:
        latex_table = make_latex_box_table(df_summary)
        print(latex_table)
        return latex_table
    else:
        print_box_summary(df_summary)
        return None
