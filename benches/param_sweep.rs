use benchmarks::fib::fib;
use benchmarks::{ParallelLH, Work};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const FIB_N: u32 = 14;
const FIB_SERIAL_CUTOFF: u32 = 0; // needs to be 0 so we fully split our computational DAG all the way

const STACK_SIZE_MB: usize = 16; // set a large stack size to avoid overflow
const LATENCY_MS: [u64; 4] = [0, 1, 50, 100];
const LATENCY_P: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

#[derive(Copy, Clone)]
struct Params(Work);

impl std::fmt::Display for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Params(Work::Nothing) | Params(Work::PureLatency { .. }) => {
                panic!("Should not happen during benching")
            }
            Params(Work::LatencyOrCompute {
                latency_ms,
                latency_p,
            }) => {
                write!(f, "Latency ms: {} | Latency p: {}", latency_ms, latency_p)
            }
        }
    }
}

fn param_sweep(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("Fib Parameter Sweep");

    rayon::ThreadPoolBuilder::new()
        .stack_size(STACK_SIZE_MB * 1024 * 1024)
        .build_global()
        .unwrap();

    for latency_p in LATENCY_P {
        for latency_ms in LATENCY_MS.map(|l| if l == 0 { None } else { Some(l) }) {
            let params = Params(Work::new(latency_ms, Some(latency_p)));

            bench_group.bench_with_input(
                BenchmarkId::new("Latency Hiding", params),
                &params,
                |b, p| {
                    b.iter(|| {
                        fib::<ParallelLH>(
                            black_box(FIB_N),
                            black_box(&p.0),
                            black_box(FIB_SERIAL_CUTOFF),
                        )
                    })
                },
            );
        }
    }

    bench_group.finish();
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = param_sweep
}
criterion_main!(benches);
