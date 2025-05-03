import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator
import math

csv_files = ["0_log.csv", "1_log.csv"] 

# Load and concatenate CSVs
df_list = []
for file in csv_files:
    temp_df = pd.read_csv(file)
    temp_df.columns = [col.strip().lower() for col in temp_df.columns]
    temp_df['client'] = os.path.basename(file)
    df_list.append(temp_df)

df = pd.concat(df_list, ignore_index=True)
df = df.sort_values(by='id').reset_index(drop=True)

operations = sorted(df['operation_name'].unique())

rows = cols = 3
fig, axes = plt.subplots(3, 3, figsize=(15, 12))

for idx, op in enumerate(operations):
    r, c = divmod(idx, cols)
    ax = axes[r][c]

    subset = df[df['operation_name'] == op]
    ax.plot(subset['id'], subset['elapsed'], marker='o')
    ax.set_title(f'Op: {op}')

    #ax.xaxis.set_major_locator(MaxNLocator(nbins=3))
    #ax.yaxis.set_major_locator(MaxNLocator(nbins=3))

    if r == rows - 1:
        ax.set_xlabel("ID")
    if c == 0:
        ax.set_ylabel("Time")

plt.tight_layout()
plt.suptitle("Operation Times by ID (Per Prescription)", fontsize=16, y=1.02)
plt.show()
