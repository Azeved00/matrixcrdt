from invoke import task
import os
import shutil
import pandas as pd
import matplotlib.pyplot as plt
import time
from datetime import datetime

from tasks.utils import notify_on_finish
from scripts.macro.workload import run_benchmark, run_matrix
from scripts.macro.graph import plot_graphs, merge_and_validate as merge, make_table, strategies


def backup_and_reset_folder(folder_path, backup_root="backup"):
    if os.path.exists(folder_path):
        # Create backup folder with timestamp
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_path = os.path.join(backup_root, f"{os.path.basename(folder_path)}_{timestamp}")
        os.makedirs(backup_path, exist_ok=True)

        # Move all contents to backup folder
        for item in os.listdir(folder_path):
            s = os.path.join(folder_path, item)
            d = os.path.join(backup_path, item)
            shutil.move(s, d)

    # Recreate the folder
    os.makedirs(folder_path, exist_ok=True)

@task()
@notify_on_finish("Benchmark Finished", "Benchmark Finished")
def benchmark(c, name, clients, operations, 
              workload_seed=76, repetitions=1,
              benchmark_logs_dir="./logs/macro/temp",
              graph_strategy="all", graph_remove_outliers=0.5, graph_use_dag_ops=True,
              graph_num_groups=20,
              table_latex=True, graph_output_dir="./plots", graph_show=True,
              graph_include_front=False):
    """Default function to run benchmarks"""

    temp_dir = "temp1234"
    for folder in [benchmark_logs_dir, graph_output_dir, temp_dir]:
        backup_and_reset_folder(folder)

    graph_color="steelblue"
    graph_front_color="lightskyblue"
    df_list=[]

    for i in range(repetitions):
        print("🔧 Running benchmark...")
        match name:
            case "authdag":
                run_benchmark(
                    clients=clients, operations=operations,
                    backend_bin="socket", frontend_env= "",
                    logs_dir=temp_dir,
                    seed=workload_seed,
                )
                graph_color="yellow"
                graph_front_color="gold"
            case "authless":
                run_benchmark(
                    clients=clients, operations=operations,
                    backend_bin="baseline", frontend_env= "",
                    logs_dir=temp_dir,
                    seed=workload_seed,
                )
            case "stateless":
                run_benchmark(
                    clients=clients, operations=operations,
                    backend_bin="socket", frontend_env= "stateless",
                    logs_dir=temp_dir,
                    seed=workload_seed,
                )
                graph_color="salmon"
                graph_front_color="lightsalmon"
            case "baseline":
                run_benchmark(
                    clients=clients, operations=operations,
                    backend_bin="baseline", frontend_env= "stateless",
                    logs_dir=temp_dir,
                    seed=workload_seed,
                )
                graph_color="steelblue"
                graph_front_color="lightskyblue"
            case "matrix":
                run_matrix(
                    clients=clients, operations=operations,
                    frontend_env= "",
                    logs_dir=temp_dir,
                    seed=workload_seed,
                )
        time.sleep(3)


        df1 = merge(temp_dir)
        df_list.append(df1)
        shutil.move(temp_dir, f"{benchmark_logs_dir}/run_{i}")
        os.makedirs(temp_dir, exist_ok=True)
        

    full_df = pd.concat(df_list, ignore_index=True)
    print("📈 Graphing results...")
    for outliers in [0, graph_remove_outliers]:
        output_dir = f"{graph_output_dir}/removed-{outliers}"
        os.makedirs(output_dir, exist_ok=True)
        if graph_strategy == "all":
            for t in strategies:
                print(f"For strategy {t}...")
                plots = plot_graphs(full_df, 
                                    strategy=t, 
                                    color=graph_color,
                                    front_color=graph_front_color,
                                    remove_outliers=outliers, 
                                    num_groups=graph_num_groups,
                                    include_front=graph_include_front, 
                                    use_dag_ops=graph_use_dag_ops)

                for op_name, fig in plots.items():
                    if graph_show:
                        fig.show()
                    fig.savefig(f'{output_dir}/{name}-{t}-{op_name}.png')
                    plt.close(fig)
        else:
            plots = plot_graphs(df_list, strategy=graph_strategy, warmup=graph_warmup, 
                                include_front=graph_include_front, use_dag_ops=graph_use_dag_ops)

            for op_name, fig in plots.items():
                if graph_show:
                    fig.show()
                fig.savefig(f'{graph_output_dir}/{name}-{op_name}.png')
                plt.close(fig)

    print("Plots saved in 'plots' directory.")
    table = make_table(full_df, include_front=graph_include_front, latex=table_latex)
    with open(f"{graph_output_dir}/table.tex", "w") as f:
        f.write(table)

    shutil.rmtree(temp_dir)
