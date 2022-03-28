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

        data.append(observation)

df = pd.DataFrame(data)

# Append rows for Ideal scheduler
ideal = df.loc[((df['Scheduler'] == 'Classic') & (df['Latency ms'] == 0))]
ideal.loc[:, 'Scheduler'] = 'Ideal'
df = pd.concat([df, ideal], ignore_index=True)

print(df, end='\n---------------\n')

latencies = df.loc[(df['Latency ms'] != 0), 'Latency ms'].unique() # for each latency graph, don't include 0 ms

sns.set_theme(style="darkgrid")

for latency in latencies:
    latency_view = df.loc[(((df['Latency ms'] == latency) | (df['Scheduler'] == 'Ideal')) & (df['Fib N'] == 30) & (df['Cutoff'] == 25))]

    serial_baseline = latency_view.loc[((latency_view['Scheduler'] == 'Serial') &
                                       (latency_view['Latency ms'] == latency)), 'Wallclock']
    assert serial_baseline.size == 1
    serial_baseline = serial_baseline.iloc[0]

    speedups = latency_view.loc[(latency_view['Scheduler'] != 'Serial'), ['Scheduler', 'Cores', 'Wallclock']]
    speedups['Speedup'] = speedups['Wallclock'].map(lambda x: serial_baseline / x)
    print(speedups)

    # sns.relplot(data=speedups, x='Cores', y='Speedup', style='Scheduler', hue='Scheduler', kind='line', marker='o')
    # plt.title(f'Latency: {latency}')
    # plt.show()

    ideal = speedups.loc[speedups['Scheduler'] == 'Ideal', ['Cores', 'Speedup']].sort_values(by=['Cores'])
    classic = speedups.loc[speedups['Scheduler'] == 'Classic', ['Cores', 'Speedup']].sort_values(by=['Cores'])
    lh = speedups.loc[speedups['Scheduler'] == 'Latency Hiding', ['Cores', 'Speedup']].sort_values(by=['Cores'])

    with sns.axes_style(style="darkgrid"):
        plt.plot(ideal['Cores'], ideal['Speedup'], marker='o', label='Ideal')
        plt.plot(classic['Cores'], classic['Speedup'], marker='^', label='Classic')
        plt.plot(lh['Cores'], lh['Speedup'], marker='D', label='Latency Hiding')

        plt.title(f'Latency: {latency}')
        plt.legend(loc='best')
        plt.xlabel('Cores')
        plt.ylabel('Speedup')
        plt.show()
