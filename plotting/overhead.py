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

speedups = df.loc[(df['Scheduler'] != 'Serial'), ['Scheduler', 'Cores', 'Wallclock']]
speedups['Speedup'] = serial_baseline / speedups['Wallclock']

classic = speedups.loc[speedups['Scheduler'] == 'Classic', ['Cores', 'Speedup']].sort_values(by=['Cores'])
lh = speedups.loc[speedups['Scheduler'] == 'Latency Hiding', ['Cores', 'Speedup']].sort_values(by=['Cores'])

with sns.axes_style(style="whitegrid"):
    plt.plot(classic['Cores'], classic['Speedup'], marker='D', label='Classic')
    plt.plot(lh['Cores'], lh['Speedup'], marker='^', label='Latency Hiding')

    plt.title(f'Scheduler Overhead Compared to Classic Work Stealing')
    plt.legend(loc='best')
    plt.xlabel('Logical Cores')
    plt.ylabel(r'Speedup $T_1 / T_P$')
    sns.despine()
    plt.savefig('plotting/plots/overhead_plot.png', dpi=200)
    plt.clf() # plt.show()
