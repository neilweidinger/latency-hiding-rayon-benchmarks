use benchmarks::fib::fib;
use benchmarks::{Parallel, ParallelLH, Serial, Work};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const FIB_N: u32 = 14;
const FIB_SERIAL_CUTOFF: u32 = 0; // needs to be 0 so we fully split our computational DAG all the way

const STACK_SIZE_MB: usize = 16; // set a large stack size to avoid overflow
const WORK_MS: [Option<u64>; 3] = [Some(1), Some(50), Some(100)]; // no need for 0 latency/compute time, since we always want to do at least some amount of work
const LATENCY_P: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

#[derive(Copy, Clone)]
struct Params(Work);

impl std::fmt::Display for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Params(Work::DoNothing) | Params(Work::PureLatency { .. }) => {
                panic!("Should not happen during param sweep benching")
            }
            Params(Work::LatencyOrCompute { work_ms, latency_p }) => {
                write!(f, "Work ms: {} | Latency p: {}", work_ms, latency_p)
            }
        }
    }
}

fn param_sweep(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("Fib Parameter Sweep");

    // Use all cores available
    rayon::ThreadPoolBuilder::new()
        .stack_size(STACK_SIZE_MB * 1024 * 1024)
        .build_global()
        .unwrap();

    for work_ms in WORK_MS {
        // hardcode Serial and Parallel to always do pure compute, as they don't support the
        // notion of hiding latency anyway
        let params = Params(Work::new(work_ms, Some(0.0)));

        // Serial benchmark
        bench_group.bench_with_input(BenchmarkId::new("Serial", params), &params, |b, p| {
            b.iter(|| {
                fib::<Serial>(
                    black_box(FIB_N),
                    black_box(&p.0),
                    black_box(FIB_SERIAL_CUTOFF),
                )
            })
        });

        // Parallel benchmarks
        bench_group.bench_with_input(BenchmarkId::new("Classic", params), &params, |b, p| {
            b.iter(|| {
                fib::<Parallel>(
                    black_box(FIB_N),
                    black_box(&p.0),
                    black_box(FIB_SERIAL_CUTOFF),
                )
            })
        });

        for latency_p in LATENCY_P {
            let params = Params(Work::new(work_ms, Some(latency_p)));

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
