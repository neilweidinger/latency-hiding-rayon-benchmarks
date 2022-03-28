from pathlib import Path

import json
import os
import numpy as np
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

target_root = os.path.join(os.getcwd(), 'target/criterion')
bench_group = os.path.join(target_root, 'Fib Parameter Sweep')
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

                observation[param_name] = float(param_value) # all param string values are specified as ints

        with open(os.path.join(bench, 'new/estimates.json')) as f:
            bench_results = json.load(f)
            observation['Wallclock'] = bench_results['mean']['point_estimate']

        data.append(observation)

df = pd.DataFrame(data)

print(df, end='\n---------------\n')

work_times = df.loc[:, 'Work ms'].unique() # get work ms times

for work_ms in work_times:
    serial_baseline = df.loc[((df['Scheduler'] == 'Serial') & (df['Work ms'] == work_ms)), 'Wallclock']
    assert serial_baseline.size == 1
    serial_baseline = serial_baseline.iloc[0]

    df.loc[(df['Work ms'] == work_ms), 'Speedup'] = serial_baseline / df.loc[(df['Work ms'] == work_ms), 'Wallclock']
    print(df)

speedup_matrix = df.loc[df['Scheduler'] == 'Latency Hiding'].pivot(columns='Work ms', index='Latency p',
                                                                   values='Speedup').sort_index(ascending=False)
print(speedup_matrix)

with sns.axes_style(style="ticks"):
    sns.relplot(data=speedup_matrix)
    plt.show()

    # TODO: come up with nice visualization (either plot or table)
