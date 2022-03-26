use benchmarks::map_reduce::map_reduce;
use benchmarks::map_reduce::map_reduce_fib;
use benchmarks::{Joiner, Parallel, ParallelLH, Serial};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::iter::Iterator;

const LATENCY_MS: [u64; 5] = [0, 1, 50, 100, 500];

fn param_string(latency_ms: Option<u64>, cores: usize) -> String {
    if let Some(l) = latency_ms {
        format!("Latency ms - {} Cores - {}", l, cores)
    } else {
        format!("Latency ms - 0 Cores - {}", cores)
    }
}

fn map_reduce_fib<J: Joiner>(latency_ms: Option<u64>) -> u32 {
    fn constrain<F>(f: F) -> F
    where
        F: for<'a> Fn(&'a mut u32) -> u32,
    {
        f
    }

    let map = constrain(|&mut n| map_reduce_fib::map::<J>(n, latency_ms));
    let mut fib_n = [30; 5];

    map_reduce::<J, _, _, _, _, _>(
        &mut fib_n,
        &map,
        &map_reduce_fib::reduce,
        &map_reduce_fib::identity,
    )
}

fn map_reduce_fib_bench(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("MapReduce Fib Parallel");
    let step = if num_cpus::get() <= 10 { 2 } else { 5 };
    let num_cores = [1]
        .into_iter()
        .chain((step..=num_cpus::get()).step_by(step));

    bench_group.bench_function(BenchmarkId::new("Serial", param_string(None, 1)), |b| {
        b.iter(|| map_reduce_fib::<Serial>(black_box(None)))
    });

    for cores in num_cores {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .build()
            .unwrap();

        for latency_ms in LATENCY_MS.map(|l| if l == 0 { None } else { Some(1) }) {
            bench_group.bench_with_input(
                BenchmarkId::new("Classic", param_string(latency_ms, cores)),
                &latency_ms,
                |b, &l| pool.install(|| b.iter(|| map_reduce_fib::<Parallel>(black_box(l)))),
            );

            bench_group.bench_with_input(
                BenchmarkId::new("Latency Hiding", param_string(latency_ms, cores)),
                &latency_ms,
                |b, &l| pool.install(|| b.iter(|| map_reduce_fib::<ParallelLH>(black_box(l)))),
            );
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
