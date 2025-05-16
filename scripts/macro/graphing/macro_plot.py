import pandas as pd
import matplotlib.pyplot as plt
import os

def plot_graphs(df):
    # Ensure the elapsed columns are numeric
    df['total_time'] = pd.to_numeric(df['total_time'], errors='coerce')
    df['front_time'] = pd.to_numeric(df['front_time'], errors='coerce')
    df['back_time'] = pd.to_numeric(df['back_time'], errors='coerce')  

    # Fill NaNs with 0 for plotting
    df = df.fillna(0)

    # Create output folder for plots
    os.makedirs("plots", exist_ok=True)

    # Plot for each unique operation_name_script
    for op_name in df['client_operation'].unique():
        subset = df[df['client_operation'] == op_name]

        backend_time = subset['back_time']
        frontend_time = subset['front_time']
        total_time = subset['total_time']
        extra_time =  total_time - frontend_time - backend_time

        indices = range(len(subset))
        
        plt.figure(figsize=(10, 6))
        #plt.bar(indices, extra_time, bottom=backend_time + frontend_time, label='Other Time', color='green')
        plt.bar(indices, frontend_time, bottom=backend_time , label='Frontend Time', color='orange')
        plt.bar(indices, backend_time, label='Backend Time', color='skyblue')

        plt.title(f'Elapsed Time Breakdown - {op_name}')
        plt.xlabel('Record Index')
        plt.ylabel('Time')
        plt.legend()
        plt.tight_layout()
        
        # Save each plot
        #plt.savefig(f'plots/{op_name}_elapsed_breakdown.png')
        #plt.close()
        plt.show()

if __name__=="main":
    input_path = sys.argv[1] if len(sys.argv) > 1 else 'input'
    df = pd.read_csv(triple[0])
    plot_graphs(df)
    #print("Plots saved in 'plots' directory.")
