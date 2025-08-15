import pandas as pd
import matplotlib.pyplot as plt
import sys
import os
import seaborn as sns

strategies = ["scatter", "box", "mean"]


def plot_graphs(df,strategy: str = "box", warmup=0,  include_front=True, use_dag_ops=False):
    """
    Plots benchmarking data using the specified visualization strategy.

    Args:
        df (pd.DataFrame): The benchmarking data.
        strategy (str): Visualization strategy to use. Options are:
                        "scatter", "mean", or "box".
        warmup (int): Number of initial entries to skip (for warm-up period).
        include_front (bool): Whether to include frontend times in plots.
        use_dag_ops (bool): If True, use 'dag_operation' column instead of 'client_operation'.

    Outputs:
        A dictionary from operations to matplotlib plots.
    """
    df['total_time'] = pd.to_numeric(df['total_time'], errors='coerce')
    df['front_time'] = pd.to_numeric(df['front_time'], errors='coerce')
    df['back_time'] = pd.to_numeric(df['back_time'], errors='coerce')  

    df = df.fillna(0)
    df['front_time'] = df['front_time'] + df['back_time']

    if warmup > 0 and len(df) > warmup:
        df = df.iloc[warmup:]

    # Plot for each unique operation_name_script
    final = {}
    op_field = 'dag_operation' if use_dag_ops else 'client_operation'
    for op_name_x in df[op_field].unique():
        op_name = op_name_x.strip()
        if op_name == "":
            continue

        subset = df[df[op_field].str.strip() == op_name]
        
        plt.figure(figsize=(10, 6))

        match strategy:
            case "scatter":
                plt.scatter(subset['front_id'], subset['back_time'], marker="o", 
                         label=f'Backend Time', 
                         alpha=0.7, color='#40456a')

                if  include_front:
                    plt.scatter(subset['front_id'], subset['front_time'], marker="o",
                             label=f'Frontend Time',
                             alpha=0.7, color='#f99d1b')
                plt.title(f"Scatter plot {op_name}")
                plt.xlabel('System Time')
                plt.ylabel('Request Time')

            case "mean":
                back= subset.groupby('front_id')['back_time'].mean().reset_index()
                front= subset.groupby('front_id')['front_time'].mean().reset_index()

                plt.plot(back['front_id'], back['back_time'],
                         marker='o', linestyle='-',
                         label=f'Backend Time', 
                         alpha=0.7, color='#40456a')
                if include_front:
                    plt.plot(front['front_id'],front['front_time'],
                             marker='o', linestyle='-',
                             label=f'Frontend Time',
                             alpha=0.7, color='#f99d1b')
                plt.title(f"Mean plot {op_name}")
                plt.xlabel('System Time')
                plt.ylabel('Request Time')

            case "box":
                box_df = subset.copy()
                box_df['id_qbin'] = pd.qcut(box_df['front_id'], q=19, duplicates='drop')
                box_df['id_qbin'] = box_df['id_qbin'].apply(
                        lambda x: f"{int(x.left)}–{int(x.right)}")

                value_vars = ['back_time']
                if include_front:
                    value_vars =['front_time', 'back_time']

                df_melted = pd.melt(
                    box_df,
                    id_vars='id_qbin',
                    value_vars= value_vars,
                    var_name='Stage',
                    value_name='Time'
                )
                sns.boxplot(x='id_qbin', y='Time', hue='Stage', data=df_melted)

                plt.title(f"Box Plot: {op_name}")
                plt.xticks(rotation=45)
                plt.xlabel('System Time')
                plt.ylabel('Request Time')

            case _:
                print("invalid strategy")
                return

        plt.legend()
        plt.tight_layout()
        final[op_name] = plt
    return final
        

def plot_comparison(df1, df1_name, df2, df2_name, include_front=True, use_dag_ops=False):
    """
    Compares two benchmarking runs by plotting their frontend and backend times.

    Args:
        df1 (pd.DataFrame): First benchmarking dataset.
        df1_name (str): Label for the first dataset.
        df2 (pd.DataFrame): Second benchmarking dataset.
        df2_name (str): Label for the second dataset.
        include_front (bool): Whether to include frontend times in comparison.
        use_dag_ops (bool): If True, use 'dag_operation' column instead of 'client_operation'.

    Outputs:
        A dictionary from operation to plot.
    """
    # Ensure the elapsed columns are numeric
    for df in [df1, df2]:
        df['total_time'] = pd.to_numeric(df['total_time'], errors='coerce')
        df['front_time'] = pd.to_numeric(df['front_time'], errors='coerce')
        df['back_time'] = pd.to_numeric(df['back_time'], errors='coerce')

    # Get all unique operations from both DataFrames
    final = {}
    op_field = 'dag_operation' if use_dag_ops else 'client_operation'
    all_operations = set(df1[op_field].unique()).union(df2[op_field].unique())

    for op_name in all_operations:
        op_name = op_name.strip()
        if op_name == "":
            continue

        subset1 = df1[df1[op_field].str.strip() == op_name]
        subset2 = df2[df2[op_field].str.strip() == op_name]

        # Pad the data to make them the same length for plotting
        back1 = subset1['back_time']
        back2 = subset2['back_time']
    
        #frontvalues
        front1 = subset1['front_time']
        front2 = subset2['front_time']

        front1 = [a + b for a, b in zip(front1, back1)]
        front2 = [a + b for a, b in zip(front2, back2)]

        plt.figure(figsize=(10, 6))
        #plot backend
        plt.plot(range(len(back1)), back1, marker="o", label=f'{df1_name} - Backend Time', alpha=0.7, color='skyblue')
        plt.plot(range(len(back2)), back2, marker="o", label=f'{df2_name} - Backend Time', alpha=0.7, color='salmon')

        #plot frontend
        if include_front:
            plt.plot(range(len(front1)), front1, marker="o", label=f'{df1_name} - Frontend Time', alpha=0.7, color='deepskyblue')
            plt.plot(range(len(front2)), front2, marker="o", label=f'{df2_name} - Frontend Time', alpha=0.7, color='orangered')

        plt.title(f'Backend Time Comparison - {op_name}')
        plt.xlabel('Record Index')
        plt.ylabel('Backend Time')
        plt.legend()
        plt.tight_layout()
        final[op_name] = plt
    return final

