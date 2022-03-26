use benchmarks::quicksort::{generate_random_sequence, quicksort};
use benchmarks::{Parallel, Serial};
use criterion::BatchSize::SmallInput;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn inputs() -> Vec<Vec<i32>> {
    [1_000, 1_000_000, 10_000_000]
        .iter()
        .map(|&len| generate_random_sequence(len))
        .collect()
}

fn param_string(cores: usize, len: usize) -> String {
    format!("Cores - {} Len - {}", cores, len)
}

fn quicksort_bench(c: &mut Criterion) {
    let mut quicksort_group = c.benchmark_group("Quicksort");
    let step = if num_cpus::get() <= 10 { 2 } else { 5 };
    let num_cores = [1]
        .into_iter()
        .chain((step..=num_cpus::get()).step_by(step));
    let inputs = inputs();

    for input in inputs.iter() {
        // Bench serial version once for each len, no need to loop through cores since serial
        // version only runs on one core (doesn't hook into Rayon)
        quicksort_group.bench_function(
            BenchmarkId::new("Serial", param_string(1, input.len())),
            |b| {
                b.iter_batched_ref(
                    || input.clone(),
                    |i| quicksort::<_, Serial>(black_box(i), 0, 0.0), // no need for pool since serial version doesn't hook into Rayon
                    SmallInput,
                );
            },
        );
    }

    for cores in num_cores.clone() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .build()
            .unwrap();

        for input in inputs.iter() {
            quicksort_group.bench_function(
                BenchmarkId::new("Parallel", param_string(cores, input.len())),
                |b| {
                    b.iter_batched_ref(
                        || input.clone(),
                        |i| pool.install(|| quicksort::<_, Parallel>(black_box(i), 0, 0.0)),
                        SmallInput,
                    );
                },
            );
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
