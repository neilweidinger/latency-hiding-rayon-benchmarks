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
    let mut fib_n = [30; 15];

    map_reduce(
        black_box(&mut fib_n),
        &map,
        &map_reduce_fib::reduce,
        &map_reduce_fib::identity,
    )
}

fn map_reduce_fib_bench(c: &mut Criterion) {
    for latency_ms in [1, 50, 100, 500] {
        c.bench_with_input(
            BenchmarkId::new("MapReduce Fib", format!("Latency - {}", latency_ms)),
            &latency_ms,
            |b, &l| b.iter(|| map_reduce_fib(false, l)),
        );
        c.bench_with_input(
            BenchmarkId::new("MapReduce Fib LH", format!("Latency - {}", latency_ms)),
            &latency_ms,
            |b, &l| b.iter(|| map_reduce_fib(false, l)),
        );
    }
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = map_reduce_fib_bench
}
criterion_main!(benches);
