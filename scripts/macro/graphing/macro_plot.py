import pandas as pd
import matplotlib.pyplot as plt
import os

def plot_graphs(df, place="."):
    # Ensure the elapsed columns are numeric
    df['total_time'] = pd.to_numeric(df['total_time'], errors='coerce')
    df['front_time'] = pd.to_numeric(df['front_time'], errors='coerce')
    df['back_time'] = pd.to_numeric(df['back_time'], errors='coerce')  

    df = df.fillna(0)

    os.makedirs("plots", exist_ok=True)

    # Plot for each unique operation_name_script
    for op_name_x in df['client_operation'].unique():
        op_name = op_name_x.strip()
        subset = df[df['client_operation'] == op_name]

        backend_time = subset['back_time']
        frontend_time = subset['front_time']
        total_time = subset['total_time']
        extra_time =  total_time - frontend_time - backend_time

        indices = range(len(subset))
        
        plt.figure(figsize=(10, 6))
        #plt.bar(indices, extra_time, bottom=backend_time + frontend_time, label='Other Time', color='green')
        plt.bar(indices, backend_time, label='Backend Time', color='skyblue')

        plt.title(f'Elapsed Time Breakdown - {op_name}')
        plt.xlabel('Record Index')
        plt.ylabel('Time')
        plt.legend()
        plt.tight_layout()
        
        # Save each plot
        plt.savefig(f'plots/{place}/{op_name}.png')
        #plt.close()
        #plt.show()

def plot_comparisson(df1, df1_name, df2, df2_name, place = "."):
    # Ensure the elapsed columns are numeric
    for df in [df1, df2]:
        df['total_time'] = pd.to_numeric(df['total_time'], errors='coerce')
        df['front_time'] = pd.to_numeric(df['front_time'], errors='coerce')
        df['back_time'] = pd.to_numeric(df['back_time'], errors='coerce')
        df.fillna(0, inplace=True)

    os.makedirs(f"plots/{place}", exist_ok=True)

    # Get all unique operations from both DataFrames
    all_operations = set(df1['client_operation'].unique()).union(df2['client_operation'].unique())

    for op_name in all_operations:
        op_name = op_name.strip()

        subset1 = df1[df1['client_operation'].str.strip() == op_name]
        subset2 = df2[df2['client_operation'].str.strip() == op_name]

        # Determine the number of records to align the plots
        max_len = max(len(subset1), len(subset2))
        indices = range(max_len)

        # Pad the data to make them the same length for plotting
        back1 = subset1['back_time'].reset_index(drop=True)
        back2 = subset2['back_time'].reset_index(drop=True)

        back1 = back1.reindex(range(max_len), fill_value=0)
        back2 = back2.reindex(range(max_len), fill_value=0)

        plt.figure(figsize=(10, 6))
        plt.bar(indices, back1, label=f'{df1_name} - Backend Time', alpha=0.7, color='skyblue')
        plt.bar(indices, back2, label=f'{df2_name} - Backend Time', alpha=0.7, color='salmon')

        plt.title(f'Backend Time Comparison - {op_name}')
        plt.xlabel('Record Index')
        plt.ylabel('Backend Time')
        plt.legend()
        plt.tight_layout()

        plt.savefig(f'plots/{place}/{op_name}.png')
        plt.close()

if __name__=="main":
    input_path = sys.argv[1] if len(sys.argv) > 1 else 'input'
    df = pd.read_csv(triple[0])
    plot_graphs(df)
    print("Plots saved in 'plots' directory.")
