import pandas as pd
import matplotlib.pyplot as plt
import math
import os

csv_files = ["0_log.csv", "1_log.csv"]

# Load and combine
df_list = []
for file in csv_files:
    temp_df = pd.read_csv(file)
    temp_df.columns = [col.strip().lower() for col in temp_df.columns]
    temp_df['client'] = os.path.basename(file)
    df_list.append(temp_df)

df = pd.concat(df_list, ignore_index=True)
df = df.sort_values(by='id').reset_index(drop=True)

operations = sorted(df['operation_name'].unique())

window_size = 50

# Grid size
cols = 3
rows = 3
fig, axes = plt.subplots(rows, cols, figsize=(cols * 5, rows * 4))
axes = axes.flatten()

# Plot each operation
for idx, op in enumerate(operations):
    ax = axes[idx]
    subset = df[df['operation_name'] == op].sort_values(by='id').copy()
    
    subset['mean'] = subset['elapsed'].rolling(window=window_size).mean()
    subset['median'] = subset['elapsed'].rolling(window=window_size).median()
    subset['p95'] = subset['elapsed'].rolling(window=window_size).quantile(0.95)
    
    ax.plot(subset['id'], subset['mean'], label='Mean', color='blue')
    ax.plot(subset['id'], subset['median'], label='Median', color='green')
    ax.plot(subset['id'], subset['p95'], label='95%ile', color='red')
    
    ax.set_title(f"Op: {op}")
    ax.set_xlabel("ID")
    ax.set_ylabel("Elapsed Time")
    ax.grid(True)

axes[0].legend(loc='upper right')

plt.tight_layout()
plt.suptitle("Rolling Stats (Mean, Median, 95%ile) by Operation", fontsize=16, y=1.03)
plt.subplots_adjust(top=0.92)
plt.show()

