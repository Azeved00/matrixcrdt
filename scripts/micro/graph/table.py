
import numpy as np

# OPERATION_TEXT constant mapping
OPERATION_TEXT = {
    "apply": "Update",
    "query": "Query",
    "stateful_query": "Query",
    "stateless_query": "Query"
}

def make_table(dataframe, group_percentage, sample_n, boxes):
    """
    Generates a summary table for LaTeX with Box rows and operations as multi-column headers.
    Returns a structure directly usable by table_to_latex.
    """
    if not all(col in dataframe.columns for col in ['id', 'time', 'operation']):
        raise ValueError("DataFrame must contain 'id', 'time', and 'operation' columns.")

    all_operations = dataframe["operation"].unique()
    display_names = [OPERATION_TEXT.get(op, op) for op in all_operations]

    box_labels = []
    grouped_quartiles = {op: [] for op in all_operations}

    for operation in all_operations:
        filtered_df = dataframe[dataframe['operation'] == operation].copy()
        total_ids = len(filtered_df['id'].unique())
        group_size = max(1, int(total_ids * group_percentage))
        filtered_df['group'] = (filtered_df['id'] - 1) // group_size
        grouped_df = [group for _, group in filtered_df.groupby('group')]

        n_groups = len(grouped_df)
        if n_groups == 0:
            continue

        selected_indices = np.linspace(0, n_groups - 1, boxes, dtype=int)

        for idx in selected_indices:
            group = grouped_df[idx]
            times = group['time']
            q1, median, q3 = np.percentile(times, [25, 50, 75])
            grouped_quartiles[operation].append([int(q1), int(median), int(q3)])

            if operation == all_operations[0]:
                start = idx * group_size * (sample_n if operation == "query" else 1) + 1
                end = (idx + 1) * group_size * (sample_n if operation == "query" else 1)
                box_labels.append(f"Box {idx+1} [{start}-{end}]")

    # Build a structure directly usable for LaTeX
    data_rows = []
    for i, box_label in enumerate(box_labels):
        row = [box_label]
        for op in all_operations:
            row.extend(grouped_quartiles[op][i])
        data_rows.append(row)

    return {
        'box_labels': box_labels,
        'operations': display_names,
        'data': data_rows
    }


def table_to_latex(table_data, label="tab:summary"):
    """
    Converts table_data from make_table into a LaTeX table with multi-column headers.

    table_data: dict with keys 'box_labels', 'operations', 'data'
    """
    ops = table_data['operations']
    n_ops = len(ops)
    col_format = 'l' + 'ccc'*n_ops

    latex_lines = []
    latex_lines.append("\\begin{table}[ht]")
    latex_lines.append("\\centering")
    latex_lines.append(f"\\begin{{tabular}}{{{col_format}}}")
    latex_lines.append("\\hline")

    # First header line: operation names with multicolumn
    header_line = ["Box"]
    for op in ops:
        header_line.append(f"\\multicolumn{{3}}{{c}}{{{op}}}")
    latex_lines.append(' & '.join(header_line) + " \\\\")

    # Second header line: Q1 Median Q3 for each operation
    sub_header = [""]
    for _ in ops:
        sub_header.extend(["Q1", "Median", "Q3"])
    latex_lines.append(' & '.join(sub_header) + " \\\\")
    latex_lines.append("\\hline")

    # Data rows
    for row in table_data['data']:
        latex_lines.append(' & '.join(map(str, row)) + " \\\\")

    latex_lines.append("\\hline")
    latex_lines.append("\\end{tabular}")
    latex_lines.append(f"\\caption{{\\{label}{{}} micro benchmark details table.}}")
    latex_lines.append(f"\\label{{tab.bench.micro.{label}}}")
    latex_lines.append("\\end{table}")

    return '\n'.join(latex_lines)

