use benchmarks::fib::{fib, fib_latency};
use benchmarks::quicksort::{generate_random_sequence, quicksort, quicksort_latency};
use benchmarks::Parallel;
use criterion::measurement::WallTime;
use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion,
};

const FIB_N: u32 = 8;
const LEN: usize = 1_000_000;

const LATENCY_P: [f32; 2] = [0.5, 1.0];
// const LATENCY_P: [f32; 4] = [0.1, 0.5, 0.9, 1.0];
const LATENCY_MS: [u64; 2] = [0, 20];
// const LATENCY_MS: [u64; 8] = [0, 5, 10, 20, 50, 100, 250, 500];

#[derive(Copy, Clone)]
struct Params {
    latency_ms: u64,
    latency_p: f32,
}

impl std::fmt::Display for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Latency ms: {} - Latency p: {}",
            self.latency_ms, self.latency_p
        )
    }
}

#[inline]
fn fib_classic(p: Params) {
    let _ = fib::<Parallel>(
        black_box(FIB_N),
        black_box(p.latency_ms),
        black_box(p.latency_p),
    );
}

#[inline]
fn fib_latency_hiding(p: Params) {
    let _ = fib_latency(
        black_box(FIB_N),
        black_box(p.latency_ms),
        black_box(p.latency_p),
    );
}

#[inline]
fn quicksort_classic(p: Params) {
    quicksort::<_, Parallel>(
        black_box(&mut generate_random_sequence(LEN)),
        black_box(p.latency_ms),
        black_box(p.latency_p),
    );
}

fn quicksort_latency_hiding(p: Params) {
    quicksort_latency(
        black_box(&mut generate_random_sequence(LEN)),
        black_box(p.latency_ms),
        black_box(p.latency_p),
    );
}

fn param_sweep(c: &mut Criterion) {
    fn run_param_sweep<F1: FnMut(Params), F2: FnMut(Params)>(
        latency_sweep_group: &mut BenchmarkGroup<WallTime>,
        mut classic: F1,
        mut latency_hiding: F2,
    ) {
        for latency_ms in LATENCY_MS {
            for latency_p in LATENCY_P {
                let params = Params {
                    latency_ms,
                    latency_p,
                };

                latency_sweep_group.bench_with_input(
                    BenchmarkId::new("Classic", params),
                    &params,
                    |b, p| b.iter(|| classic(*p)),
                );
                latency_sweep_group.bench_with_input(
                    BenchmarkId::new("Latency Hiding", params),
                    &params,
                    |b, p| b.iter(|| latency_hiding(*p)),
                );
            }
        }
    }

    let mut fib_group = c.benchmark_group("Fibonacci");
    run_param_sweep(&mut fib_group, fib_classic, fib_latency_hiding);
    fib_group.finish();

    let mut quicksort_group = c.benchmark_group("Quicksort");
    run_param_sweep(
        &mut quicksort_group,
        quicksort_classic,
        quicksort_latency_hiding,
    );
    quicksort_group.finish();
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = param_sweep
}
criterion_main!(benches);
