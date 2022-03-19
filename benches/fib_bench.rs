use benchmarks::fib::{fib, fib_latency};
use benchmarks::Parallel;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const FIB_N: u32 = 8;
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
            "Latency ms: {}/Latency p: {}",
            self.latency_ms, self.latency_p
        )
    }
}

fn param_sweep(c: &mut Criterion) {
    let mut latency_sweep_group = c.benchmark_group("Fibonacci");

    for latency_ms in LATENCY_MS {
        for latency_p in LATENCY_P {
            let params = Params {
                latency_ms,
                latency_p,
            };

            latency_sweep_group.bench_with_input(
                BenchmarkId::new("Non-latency Hiding", params),
                &params,
                |b, p| {
                    b.iter(|| {
                        fib::<Parallel>(
                            black_box(FIB_N),
                            black_box(p.latency_ms),
                            black_box(p.latency_p),
                        )
                    })
                },
            );
            latency_sweep_group.bench_with_input(
                BenchmarkId::new("Latency Hiding", params),
                &params,
                |b, p| {
                    b.iter(|| {
                        fib_latency(
                            black_box(FIB_N),
                            black_box(p.latency_ms),
                            black_box(p.latency_p),
                        )
                    })
                },
            );
        }
    }

    latency_sweep_group.finish();
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = param_sweep
}
criterion_main!(benches);
