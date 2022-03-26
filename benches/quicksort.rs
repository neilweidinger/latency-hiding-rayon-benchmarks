use benchmarks::quicksort::{generate_random_sequence, quicksort};
use benchmarks::{Parallel, ParallelLH, Serial};
use criterion::BatchSize::SmallInput;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const STACK_SIZE_MB: usize = 16; // set a large stack size to avoid overflow
const LATENCY_MS: [u64; 5] = [0, 1, 50, 100, 500];
const LEN: [usize; 4] = [100, 10_000, 1_000_000, 10_000_000];

fn inputs() -> Vec<Vec<i32>> {
    LEN.map(|len| generate_random_sequence(len))
        .into_iter()
        .collect()
}

fn param_string(length: usize, latency_ms: Option<u64>, cores: usize) -> String {
    if let Some(l) = latency_ms {
        format!("Length - {} Latency ms - {} Cores - {}", length, l, cores)
    } else {
        format!("Length - {} Latency ms - 0 Cores - {}", length, cores)
    }
}

fn quicksort_bench(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("Quicksort");
    let step = if num_cpus::get() <= 10 { 2 } else { 5 };
    let num_cores = [1]
        .into_iter()
        .chain((step..=num_cpus::get()).step_by(step));
    let mut all_inputs = inputs();

    // Serial Benchmarks
    for input in all_inputs.iter_mut() {
        bench_group.bench_with_input(
            BenchmarkId::new("Serial", param_string(input.len(), None, 1)),
            input,
            |b, ii| {
                b.iter_batched_ref(
                    || ii.clone(),
                    |i| quicksort::<Serial, _>(black_box(i), None), // no need for pool since serial version doesn't hook into Rayon
                    SmallInput,
                );
            },
        );
    }

    // Parallel Benchmarks
    for cores in num_cores.clone() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .stack_size(STACK_SIZE_MB * 1024 * 1024)
            .build()
            .unwrap();

        for input in all_inputs.iter_mut() {
            for latency_ms in LATENCY_MS.map(|l| if l == 0 { None } else { Some(l) }) {
                bench_group.bench_with_input(
                    BenchmarkId::new("Classic", param_string(input.len(), latency_ms, cores)),
                    input,
                    |b, ii| {
                        b.iter_batched_ref(
                            || ii.clone(),
                            |i| {
                                pool.install(|| {
                                    quicksort::<Parallel, _>(black_box(i), black_box(latency_ms))
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
                                    quicksort::<ParallelLH, _>(black_box(i), black_box(latency_ms))
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
