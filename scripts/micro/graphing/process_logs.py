import pandas as pd
import os
import sys
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator
import math

def get_files_number(folder_path):
    return len([
        name for name in os.listdir(folder_path)
        if os.path.isfile(os.path.join(folder_path, name))
    ])

def calculate_files(path, n):
    files=[]
    for i in range(1,n+1):
        files.append([f'{path}script_log_{i}.csv',f'{path}front_log_{3000+i}.csv',f'{path}back_log_{5000+i}.csv'])
    return files

def merge_triples(triple_list):
    df_list=[]
    for triple in triple_list:
        script_log = pd.read_csv(triple[0])
        script_log = script_log.add_suffix("_script")
        front_log = pd.read_csv(triple[1])
        front_log = front_log.add_suffix("_front")
        back_log = pd.read_csv(triple[2])
        back_log = back_log.add_suffix("_back")

        merged = pd.merge(script_log, front_log, left_on="id_script", right_on="req_id_front")
        final_df = pd.merge(merged, back_log, left_on="clock_front", right_on="id_back")

        df_list.append(final_df)

    return df_list

def plot_data(df):
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

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <path>")
        sys.exit(1)

    input_path = sys.argv[1]
    client_num = get_files_number(input_path)//3
    file_triples = calculate_files(input_path, client_num)
    client_df = merge_triples(file_triples)
    df = pd.concat(client_df, ignore_index=True)
    df = df.sort_values(by='id_script').reset_index(drop=True)


    df = df.drop(['clock_front', 'req_id_front', 'op_front'], axis=1)
    df = df.drop(['dag_size_back', 'space_back'], axis=1)
    df = df.rename(columns={
        'id_script': 'id', 
        'operation_name_script': 'client_operation', 
        'operation_back':'dag_operation',
        'id_back': 'back_id',
        'elapsed_script': 'total_time',
        'elapsed_front': 'front_time',
        'time_back': 'back_time'
    })
    df = df[['id', 'client_operation', 'back_id', 'dag_operation', 'total_time', 'front_time', 'back_time']]
    print(df.head())
    df.to_csv(f"final.csv", index=False)
