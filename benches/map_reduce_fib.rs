use benchmarks::map_reduce::map_reduce;
use benchmarks::map_reduce::map_reduce_fib;
use benchmarks::{Joiner, Parallel, ParallelLH, Serial, Work};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::iter::Iterator;

// (fib n, serial_cutoff)
type FibSettings = (u32, u32);

const STACK_SIZE_MB: usize = 24; // set a large stack size to avoid overflow
const LATENCY_MS: [Option<u64>; 4] = [None, Some(1), Some(50), Some(100)];
const LEN: [usize; 1] = [5000];
const FIB_SETTINGS: [FibSettings; 2] = [(30, 25), (35, 15)];

fn param_string(
    length: usize,
    latency_ms: Option<u64>,
    cores: usize,
    fib_settings: FibSettings,
) -> String {
    format!(
        "Length: {} | Latency ms: {} | Cores: {} | Fib N: {} | Cutoff: {}",
        length,
        latency_ms.unwrap_or(0),
        cores,
        fib_settings.0,
        fib_settings.1
    )
}

fn map_reduce_fib<J: Joiner>(
    input: &mut [u32],
    latency_ms: Option<u64>,
    serial_cutoff: u32,
) -> u32 {
    fn constrain<F>(f: F) -> F
    where
        F: for<'a> Fn(&'a mut u32) -> u32,
    {
        f
    }

    let map = constrain(|&mut n| {
        map_reduce_fib::map::<J>(n, &Work::new(latency_ms, None), serial_cutoff)
    });

    map_reduce::<J, _, _, _, _, _>(
        input,
        &map,
        &map_reduce_fib::reduce,
        &map_reduce_fib::identity,
    )
}

fn map_reduce_fib_bench(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("MapReduce Fib");

    let num_cores = {
        let step = if num_cpus::get() <= 10 { 2 } else { 10 };
        [1].into_iter()
            .chain((step..=num_cpus::get()).step_by(step))
    };

    for len in LEN {
        for (fib_n, serial_cutoff) in FIB_SETTINGS {
            let mut input = vec![fib_n; len];

            for latency_ms in LATENCY_MS {
                // Serial benchmark
                bench_group.bench_with_input(
                    BenchmarkId::new(
                        "Serial",
                        param_string(len, latency_ms, 1, (fib_n, serial_cutoff)),
                    ),
                    &latency_ms,
                    |b, &l| {
                        b.iter(|| {
                            map_reduce_fib::<Serial>(
                                black_box(&mut input),
                                black_box(l),
                                black_box(serial_cutoff),
                            )
                        })
                    },
                );

                // Parallel benchmarks
                // Setting up and tearing down threadpool in inner loop, but whatever
                for cores in num_cores.clone() {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(cores)
                        .stack_size(STACK_SIZE_MB * 1024 * 1024)
                        .pin()
                        .build()
                        .unwrap();

                    bench_group.bench_with_input(
                        BenchmarkId::new(
                            "Classic",
                            param_string(len, latency_ms, cores, (fib_n, serial_cutoff)),
                        ),
                        &latency_ms,
                        |b, &l| {
                            pool.install(|| {
                                b.iter(|| {
                                    map_reduce_fib::<Parallel>(
                                        black_box(&mut input),
                                        black_box(l),
                                        black_box(serial_cutoff),
                                    )
                                })
                            })
                        },
                    );

                    bench_group.bench_with_input(
                        BenchmarkId::new(
                            "Latency Hiding",
                            param_string(len, latency_ms, cores, (fib_n, serial_cutoff)),
                        ),
                        &latency_ms,
                        |b, &l| {
                            pool.install(|| {
                                b.iter(|| {
                                    map_reduce_fib::<ParallelLH>(
                                        black_box(&mut input),
                                        black_box(l),
                                        black_box(serial_cutoff),
                                    )
                                })
                            })
                        },
                    );
                }
            }
        }
    }

    bench_group.finish();
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = map_reduce_fib_bench
}
criterion_main!(benches);
