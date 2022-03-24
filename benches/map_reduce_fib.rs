use benchmarks::map_reduce::map_reduce;
use benchmarks::map_reduce::map_reduce_fib;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

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

    for latency_ms in [1, 50, 100, 500] {
        map_reduce_fib_group.bench_with_input(
            BenchmarkId::new("Classic", format!("Latency ms - {}", latency_ms)),
            &latency_ms,
            |b, &l| b.iter(|| map_reduce_fib(false, black_box(l))),
        );
        map_reduce_fib_group.bench_with_input(
            BenchmarkId::new("Latency Hiding", format!("Latency ms - {}", latency_ms)),
            &latency_ms,
            |b, &l| b.iter(|| map_reduce_fib(true, black_box(l))),
        );
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
