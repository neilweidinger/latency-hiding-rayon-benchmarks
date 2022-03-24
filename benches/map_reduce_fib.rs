use benchmarks::map_reduce::map_reduce;
use benchmarks::map_reduce::map_reduce_fib;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const LATENCY_MS: [u64; 5] = [0, 1, 50, 100, 500];

fn constrain<F>(f: F) -> F
where
    F: for<'a> Fn(&'a mut u32) -> u32,
{
    f
}

fn map_reduce_fib(hide_latency: bool, latency_ms: u64) -> u32 {
    let map = constrain(|&mut n| map_reduce_fib::map(n, hide_latency, latency_ms));
    let mut fib_n = [30; 5];

    map_reduce(
        &mut fib_n,
        &map,
        &map_reduce_fib::reduce,
        &map_reduce_fib::identity,
    )
}

fn map_reduce_fib_bench(c: &mut Criterion) {
    let mut map_reduce_fib_group = c.benchmark_group("MapReduce Fib");
    let step = if num_cpus::get() <= 10 { 2 } else { 5 };
    let num_cores = [1]
        .into_iter()
        .chain((step..=num_cpus::get()).step_by(step));

    for latency_ms in LATENCY_MS {
        for cores in num_cores.clone() {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(cores)
                .build()
                .unwrap();

            let param_string = format!("Latency ms - {} Cores - {}", latency_ms, cores);

            map_reduce_fib_group.bench_with_input(
                BenchmarkId::new("Classic", param_string.clone()),
                &latency_ms,
                |b, &l| {
                    pool.install(|| b.iter(|| map_reduce_fib(false, black_box(l))));
                },
            );
            map_reduce_fib_group.bench_with_input(
                BenchmarkId::new("Latency Hiding", param_string),
                &latency_ms,
                |b, &l| {
                    pool.install(|| b.iter(|| map_reduce_fib(true, black_box(l))));
                },
            );
        }
    }

    map_reduce_fib_group.finish();
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = map_reduce_fib_bench
}
criterion_main!(benches);
