from pathlib import Path

import json
import os
import numpy as np
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

target_root = os.path.join(os.getcwd(), 'target/criterion')
bench_group = os.path.join(target_root, 'Old vs New Rayon')
schedulers = map(lambda s: Path(os.path.join(bench_group, s)), ['Serial', 'Old Rayon', 'New Rayon'])

data = [] # list of observation dict rows to be put into a pandas df

for scheduler_path in schedulers:
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

        if observation['Cores'] == 140:
            continue
        elif observation['Latency ms'] != 0:
            continue
        elif observation['Fib N'] == 35:
            continue

        data.append(observation)

df = pd.DataFrame(data)

print(df, end='\n---------------\n')

serial_baseline = df.loc[(df['Scheduler'] == 'Serial'), 'Wallclock']
assert serial_baseline.size == 1
serial_baseline = serial_baseline.iloc[0]

df.loc[:, 'Speedup'] = serial_baseline / df.loc[:, 'Wallclock']

old = df.loc[df['Scheduler'] == 'Old Rayon', ['Cores', 'Speedup']].sort_values(by=['Cores'])
new = df.loc[df['Scheduler'] == 'New Rayon', ['Cores', 'Speedup']].sort_values(by=['Cores'])

with sns.axes_style(style="whitegrid"):
    plt.plot(old['Cores'], old['Speedup'], marker='D', label='Classic Scheduler')
    plt.plot(new['Cores'], new['Speedup'], marker='^', label='ProWS Scheduler')

    plt.title(f'New ProWS Scheduler Overhead Compared to Old Classic WS Scheduler\n(MapReduceFib with 0ms Latency)')
    plt.legend(loc='best')
    plt.xlabel('Logical Cores')
    plt.ylabel(r'Speedup $T_1 / T_P$')
    sns.despine()
    plt.savefig('plotting/plots/overhead_plot.png', dpi=200)
    plt.clf() # plt.show()
