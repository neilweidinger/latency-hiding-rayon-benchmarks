use benchmarks::quicksort::{generate_random_sequence, quicksort};
use benchmarks::{Parallel, ParallelLH, Serial, Work};
use criterion::BatchSize::SmallInput;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const STACK_SIZE_MB: usize = 32; // set a large stack size to avoid overflow
const LATENCY_MS: [Option<u64>; 4] = [None, Some(1), Some(50), Some(100)];
const LEN: [usize; 1] = [10_000_000];

fn inputs() -> Vec<Vec<i32>> {
    LEN.map(|len| generate_random_sequence(len))
        .into_iter()
        .collect()
}

fn param_string(length: usize, latency_ms: Option<u64>, cores: usize) -> String {
    format!(
        "Length: {} | Latency ms: {} | Cores: {}",
        length,
        latency_ms.unwrap_or(0),
        cores
    )
}

fn quicksort_bench(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("Quicksort");
    let mut all_inputs = inputs();

    let num_cores = {
        let step = if num_cpus::get() <= 10 { 2 } else { 10 };
        [1].into_iter()
            .chain((step..=num_cpus::get()).step_by(step))
    };

    for input in all_inputs.iter_mut() {
        for latency_ms in LATENCY_MS {
            // Serial benchmark
            bench_group.bench_with_input(
                BenchmarkId::new("Serial", param_string(input.len(), latency_ms, 1)),
                input,
                |b, ii| {
                    b.iter_batched_ref(
                        || ii.clone(),
                        |i| quicksort::<Serial, _>(black_box(i), &Work::new(latency_ms, None)),
                        SmallInput,
                    );
                },
            );

            // Parallel Benchmarks
            // Setting up and tearing down threadpool in inner loop, but whatever
            for cores in num_cores.clone() {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(cores)
                    .stack_size(STACK_SIZE_MB * 1024 * 1024)
                    .build()
                    .unwrap();

                bench_group.bench_with_input(
                    BenchmarkId::new("Classic", param_string(input.len(), latency_ms, cores)),
                    input,
                    |b, ii| {
                        b.iter_batched_ref(
                            || ii.clone(),
                            |i| {
                                pool.install(|| {
                                    quicksort::<Parallel, _>(
                                        black_box(i),
                                        black_box(&Work::new(latency_ms, None)),
                                    )
                                })
                            },
                            SmallInput,
                        );
                    },
                );

                bench_group.bench_with_input(
                    BenchmarkId::new(
                        "Latency Hiding",
                        param_string(input.len(), latency_ms, cores),
                    ),
                    input,
                    |b, ii| {
                        b.iter_batched_ref(
                            || ii.clone(),
                            |i| {
                                pool.install(|| {
                                    quicksort::<ParallelLH, _>(
                                        black_box(i),
                                        black_box(&Work::new(latency_ms, None)),
                                    )
                                })
                            },
                            SmallInput,
                        );
                    },
                );
            }
        }
    }
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = quicksort_bench
}
criterion_main!(benches);
