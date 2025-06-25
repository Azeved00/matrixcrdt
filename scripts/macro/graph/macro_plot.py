import pandas as pd
import matplotlib.pyplot as plt
import sys
import os
import seaborn as sns


def plot_graphs(df, place=".", strategy: str = "box", show=False, warmup=0):
    df['total_time'] = pd.to_numeric(df['total_time'], errors='coerce')
    df['front_time'] = pd.to_numeric(df['front_time'], errors='coerce')
    df['back_time'] = pd.to_numeric(df['back_time'], errors='coerce')  

    df = df.fillna(0)
    df['front_time'] = df['front_time'] + df['back_time']

    if warmup > 0 and len(df) > warmup:
        df = df.iloc[warmup:]


    os.makedirs(f"plots/{place}", exist_ok=True)

    # Plot for each unique operation_name_script
    for op_name_x in df['client_operation'].unique():
        op_name = op_name_x.strip()
        if op_name == "":
            continue

        subset = df[df['client_operation'].str.strip() == op_name]
        
        plt.figure(figsize=(10, 6))

        match strategy:
            case "scatter":
                plt.scatter(subset['front_id'], subset['back_time'], marker="o", 
                         label=f'Backend Time', 
                         alpha=0.7, color='#40456a')

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

                df_melted = pd.melt(
                    box_df,
                    id_vars='id_qbin',
                    value_vars=['front_time', 'back_time'],
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
        
        if show:
            plt.show()
        else:
            plt.savefig(f'plots/{place}/{op_name}.png')
        plt.close()

def plot_comparison(df1, df1_name, df2, df2_name, place = ".", show=False):
    # Ensure the elapsed columns are numeric
    for df in [df1, df2]:
        df['total_time'] = pd.to_numeric(df['total_time'], errors='coerce')
        df['front_time'] = pd.to_numeric(df['front_time'], errors='coerce')
        df['back_time'] = pd.to_numeric(df['back_time'], errors='coerce')

    os.makedirs(f"plots/{place}", exist_ok=True)

    # Get all unique operations from both DataFrames
    all_operations = set(df1['client_operation'].unique()).union(df2['client_operation'].unique())

    for op_name in all_operations:
        op_name = op_name.strip()
        if op_name == "":
            continue

        subset1 = df1[df1['client_operation'].str.strip() == op_name]
        subset2 = df2[df2['client_operation'].str.strip() == op_name]

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
        plt.plot(range(len(front1)), front1, marker="o", label=f'{df1_name} - Frontend Time', alpha=0.7, color='deepskyblue')
        plt.plot(range(len(front2)), front2, marker="o", label=f'{df2_name} - Frontend Time', alpha=0.7, color='orangered')

        plt.title(f'Backend Time Comparison - {op_name}')
        plt.xlabel('Record Index')
        plt.ylabel('Backend Time')
        plt.legend()
        plt.tight_layout()

        if show:
            plt.show()
        else:
            plt.savefig(f'plots/{place}/{op_name}.png')
        plt.close()

if __name__=="__main__":
    args=sys.argv[1:]
    if len(args) < 1 or len(args) > 2:
        print("Usage: provide 1 or 2 parameters")
        sys.exit(1)

    df1 = []
    df2 = []
    if len(args) >= 1:
        path = f"{args[0]}_final.csv"
        df1 = pd.read_csv(path, na_filter=False)
        print(df1.head())

    if len(args) >= 2:
        path = f"{args[1]}_final.csv"
        df2 = pd.read_csv(path, na_filter=False)
        print(df2.head())

    if len(args) == 1:
        plot_graphs(df1, args[0])
        print("Plots saved in 'plots' directory.")
    else:
        plot_comparison(df1,args[0],df2,args[1], f"{args[0]}x{args[1]}")
        print("Plots saved in 'plots' directory.")

