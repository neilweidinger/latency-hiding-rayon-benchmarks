import json
import os
import numpy as np
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

target_root = os.path.join(os.getcwd(), 'target/criterion')
bench_group = os.path.join(target_root, 'MapReduce Fib')

data = [] # list of observation dict rows to be put into a pandas df

for scheduler in os.scandir(bench_group):
    for bench in os.scandir(scheduler):
        observation = {} # row in pandas df
        observation['Scheduler'] = scheduler.name

        with open(os.path.join(bench, 'new/benchmark.json')) as f:
            bench_info = json.load(f)
            param_strings = bench_info['value_str'].split('|')

            for param_string in param_strings:
                param_name = param_string.split('-')[0].strip()
                param_value = param_string.split('-')[1].strip()

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

sns.set_theme(style="darkgrid")

serial_baseline = df.loc[(df['Scheduler'] == 'Serial'), 'Wallclock']
assert serial_baseline.size == 1
serial_baseline = serial_baseline.iloc[0]

speedups = df.loc[(df['Scheduler'] != 'Serial'), ['Scheduler', 'Cores', 'Wallclock']]
speedups['Speedup'] = speedups['Wallclock'].map(lambda x: serial_baseline / x)

classic = speedups.loc[speedups['Scheduler'] == 'Classic', ['Cores', 'Speedup']].sort_values(by=['Cores'])
lh = speedups.loc[speedups['Scheduler'] == 'Latency Hiding', ['Cores', 'Speedup']].sort_values(by=['Cores'])

with sns.axes_style(style="darkgrid"):
    plt.plot(classic['Cores'], classic['Speedup'], marker='^', label='Classic')
    plt.plot(lh['Cores'], lh['Speedup'], marker='D', label='Latency Hiding')

    plt.title(f'Overhead')
    plt.legend(loc='best')
    plt.xlabel('Cores')
    plt.ylabel('Speedup')
    plt.show()
