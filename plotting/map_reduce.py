from pathlib import Path

import json
import os
import numpy as np
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

target_root = os.path.join(os.getcwd(), 'target/criterion')
bench_group = os.path.join(target_root, 'MapReduce Fib')
schedulers = map(lambda s: Path(os.path.join(bench_group, s)), ['Serial', 'Classic', 'Latency Hiding'])

data = [] # list of observation dict rows to be put into a pandas df

for scheduler_path in filter(lambda dir_entry: dir_entry.name != 'report', os.scandir(bench_group)):
    for bench in filter(lambda dir_entry: dir_entry.name != 'report', os.scandir(scheduler_path)):
        observation = {} # row in pandas df
        observation['Scheduler'] = scheduler_path.name

        with open(os.path.join(bench, 'new/benchmark.json')) as f:
            bench_info = json.load(f)
            param_strings = bench_info['value_str'].split('|')

            for param_string in param_strings:
                param_name = param_string.split(':')[0].strip()
                param_value = param_string.split(':')[1].strip()

                observation[param_name] = int(param_value) # all param string values are specified as ints

        with open(os.path.join(bench, 'new/estimates.json')) as f:
            bench_results = json.load(f)
            observation['Wallclock'] = bench_results['mean']['point_estimate']

        if observation['Cores'] > 35:
            continue
        elif observation['Fib N'] == 35:
            continue

        data.append(observation)

df = pd.DataFrame(data)

# Append rows for Ideal scheduler
ideal = df.loc[((df['Scheduler'] == 'Classic') & (df['Latency ms'] == 0))]
ideal.loc[:, 'Scheduler'] = 'Ideal'
df = pd.concat([df, ideal], ignore_index=True)

print(df, end='\n---------------\n')

latencies = df.loc[(df['Latency ms'] != 0), 'Latency ms'].unique() # for each latency graph, don't include 0 ms

for latency in latencies:
    latency_view = df.loc[((df['Latency ms'] == latency) | (df['Scheduler'] == 'Ideal'))]

    serial_baseline = latency_view.loc[(latency_view['Scheduler'] == 'Serial'), 'Wallclock']
    assert serial_baseline.size == 1
    serial_baseline = serial_baseline.iloc[0]

    latency_view.loc[:, 'Speedup'] = serial_baseline / latency_view.loc[:, 'Wallclock']

    ideal = latency_view.loc[latency_view['Scheduler'] == 'Ideal', ['Cores', 'Speedup']].sort_values(by=['Cores'])
    classic = latency_view.loc[latency_view['Scheduler'] == 'Classic', ['Cores', 'Speedup']].sort_values(by=['Cores'])
    lh = latency_view.loc[latency_view['Scheduler'] == 'Latency Hiding', ['Cores', 'Speedup']].sort_values(by=['Cores'])

    with sns.axes_style(style="whitegrid"):
        plt.plot(classic['Cores'], classic['Speedup'], marker='D', label='Classic')
        plt.plot(lh['Cores'], lh['Speedup'], marker='^', label='ProWS-R')
        plt.plot(ideal['Cores'], ideal['Speedup'], marker='o', label='Ideal')

        plt.title(f'MapReduceFib with Latency: {latency}ms')
        plt.legend(loc='best')
        plt.xlabel('Worker Threads')
        plt.ylabel(r'Speedup $T_1 / T_P$')
        sns.despine()
        plt.savefig(f'plotting/plots/map_reduce_plot_latency_{latency}.png', dpi=200)
        plt.clf() # plt.show()
