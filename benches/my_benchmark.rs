use benchmarks::{fib, fib_latency, Parallel};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const FIB_N: u32 = 8;
const LATENCY_P: [f32; 2] = [0.5, 1.0];
// const LATENCY_MS: [u64; 8] = [0, 5, 10, 20, 50, 100, 250, 500];
const LATENCY_MS: [u64; 2] = [0, 20];

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

pub fn param_sweep(c: &mut Criterion) {
    let mut latency_ms_sweep = c.benchmark_group("Latency Time and P Sweep");

    for latency_ms in LATENCY_MS {
        for latency_p in LATENCY_P {
            latency_ms_sweep.bench_with_input(
                BenchmarkId::new(
                    "Non-latency Hiding",
                    Params {
                        latency_ms,
                        latency_p,
                    },
                ),
                &latency_ms,
                |b, &latency_ms| {
                    b.iter(|| {
                        fib::<Parallel>(
                            black_box(FIB_N),
                            black_box(latency_ms),
                            black_box(latency_p),
                        )
                    })
                },
            );
            latency_ms_sweep.bench_with_input(
                BenchmarkId::new(
                    "Latency Hiding",
                    Params {
                        latency_ms,
                        latency_p,
                    },
                ),
                &latency_ms,
                |b, &latency_ms| {
                    b.iter(|| {
                        fib_latency(
                            black_box(FIB_N),
                            black_box(latency_ms),
                            black_box(latency_p),
                        )
                    })
                },
            );
        }
    }

    latency_ms_sweep.finish();
}

criterion_group! {
  name = benches;
  config = Criterion::default().sample_size(25);
  targets = param_sweep
}
criterion_main!(benches);
