import pandas as pd
import matplotlib.pyplot as plt
import os

# Load the CSV
df = pd.read_csv('final.csv')

# Ensure the elapsed columns are numeric
df['elapsed_script'] = pd.to_numeric(df['elapsed_script'], errors='coerce')
df['elapsed_front'] = pd.to_numeric(df['elapsed_front'], errors='coerce')
df['time_back'] = pd.to_numeric(df['time_back'], errors='coerce')  # Assuming time_back = elapsed_back

# Fill NaNs with 0 for plotting
df = df.fillna(0)

# Create output folder for plots
os.makedirs("plots", exist_ok=True)

# Plot for each unique operation_name_script
for op_name in df['operation_name_script'].unique():
    subset = df[df['operation_name_script'] == op_name]

    # Calculate stacked bars: bottom = elapsed_back, middle = elapsed_front, top = script - front - back
    elapsed_back = subset['time_back']
    elapsed_front = subset['elapsed_front']
    elapsed_script = subset['elapsed_script']
    extra_elapsed = elapsed_script - elapsed_front - elapsed_back

    indices = range(len(subset))
    
    plt.figure(figsize=(10, 6))
    plt.bar(indices, extra_elapsed, bottom=elapsed_back + elapsed_front, label='Other Elapsed (Script)', color='green')
    plt.bar(indices, elapsed_front, bottom=elapsed_back, label='Elapsed Front', color='orange')
    plt.bar(indices, elapsed_back, label='Elapsed Back', color='skyblue')

    plt.title(f'Elapsed Time Breakdown - {op_name}')
    plt.xlabel('Record Index')
    plt.ylabel('Time')
    plt.legend()
    plt.tight_layout()
    
    # Save each plot
    #plt.savefig(f'plots/{op_name}_elapsed_breakdown.png')
    #plt.close()
    plt.show()

print("Plots saved in 'plots' directory.")
