from invoke import task
import os
import shutil
import pandas as pd
import matplotlib.pyplot as plt

from tasks.utils import notify_on_finish
from scripts.macro.workload import run_benchmark
from scripts.macro.graph import plot_graphs, merge_and_validate as merge, make_table, strategies

@task()
@notify_on_finish("Benchmark Finished", "Benchmark Finished")
def benchmark(c, name, clients, operations, 
              workload_seed=76, repetitions=1,
              benchmark_logs_dir="./logs/macro/temp",
              graph_strategy="all", graph_warmup=0, graph_use_dag_ops=True,
              table_latex=True, graph_output_dir="./plots", graph_show=True,
              graph_include_front=False):
    """Default function to run benchmarks"""

    os.makedirs(benchmark_logs_dir, exist_ok=True)
    os.makedirs(graph_output_dir, exist_ok=True)
    temp_dir="tempm12345"
    os.makedirs(temp_dir, exist_ok=True)
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
            case "matrix":
                run_benchmark(
                    clients=clients, operations=operations,
                    backend_bin="matrix", frontend_env= "",
                    logs_dir=temp_dir,
                    seed=workload_seed,
                )


        df1 = merge(temp_dir)
        df_list.append(df1)
        shutil.move(temp_dir, f"{benchmark_logs_dir}/run_{i}")
        os.makedirs(temp_dir, exist_ok=True)
        

    full_df = pd.concat(df_list, ignore_index=True)
    print("📈 Graphing results...")
    if graph_strategy == "all":
        for t in strategies:
            print(f"For strategy {t}...")
            plots = plot_graphs(full_df, strategy=t, warmup=graph_warmup, 
                        include_front=graph_include_front, use_dag_ops=graph_use_dag_ops)

            for op_name, fig in plots.items():
                if graph_show:
                    fig.show()
                fig.savefig(f'{graph_output_dir}/{name}-{t}-{op_name}.png')
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
    make_table(full_df, include_front=graph_include_front, latex=table_latex)

    shutil.rmtree(temp_dir)
