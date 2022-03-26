use benchmarks::map_reduce::map_reduce;
use benchmarks::map_reduce::map_reduce_fib;
use benchmarks::{Joiner, Parallel, ParallelLH, Serial};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::iter::Iterator;

const LATENCY_MS: [u64; 5] = [0, 1, 50, 100, 500];
const LEN: [usize; 3] = [10, 500, 4000];
const FIB_SETTINGS: [(u32, u32); 2] = [(30, 25), (35, 15)]; // (fib n, serial_cutoff)

fn param_string(length: usize, latency_ms: Option<u64>, cores: usize) -> String {
    if let Some(l) = latency_ms {
        format!("Length - {} Latency ms - {} Cores - {}", length, l, cores)
    } else {
        format!("Length - {} Latency ms - 0 Cores - {}", length, cores)
    }
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

    let map = constrain(|&mut n| map_reduce_fib::map::<J>(n, latency_ms, serial_cutoff));

    map_reduce::<J, _, _, _, _, _>(
        input,
        &map,
        &map_reduce_fib::reduce,
        &map_reduce_fib::identity,
    )
}

fn map_reduce_fib_bench(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("MapReduce Fib");
    let step = if num_cpus::get() <= 10 { 2 } else { 5 };
    let num_cores = [1]
        .into_iter()
        .chain((step..=num_cpus::get()).step_by(step));

    // Serial benchmarks
    for len in LEN {
        for (fib_n, serial_cutoff) in FIB_SETTINGS {
            let mut input = vec![fib_n; len];

            bench_group.bench_function(
                BenchmarkId::new("Serial", param_string(len, None, 1)),
                |b| {
                    b.iter(|| {
                        map_reduce_fib::<Serial>(
                            black_box(&mut input),
                            black_box(None),
                            black_box(serial_cutoff),
                        )
                    })
                },
            );
        }
    }

    // Parallel benchmarks
    for cores in num_cores.clone() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .build()
            .unwrap();

        for len in LEN {
            for (fib_n, serial_cutoff) in FIB_SETTINGS {
                let mut input = vec![fib_n; len];

                for latency_ms in LATENCY_MS.map(|l| if l == 0 { None } else { Some(l) }) {
                    bench_group.bench_with_input(
                        BenchmarkId::new("Classic", param_string(len, latency_ms, cores)),
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
                        BenchmarkId::new("Latency Hiding", param_string(len, latency_ms, cores)),
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
