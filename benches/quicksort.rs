use benchmarks::quicksort::{generate_random_sequence, quicksort};
use benchmarks::{Parallel, Serial};
use criterion::BatchSize::SmallInput;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn quicksort_bench(c: &mut Criterion) {
    let mut quicksort_group = c.benchmark_group("Quicksort");
    let step = if num_cpus::get() <= 10 { 2 } else { 5 };
    let num_cores = [1]
        .into_iter()
        .chain((step..=num_cpus::get()).step_by(step));

    for len in [1_000, 1_000_000, 10_000_000] {
        for cores in num_cores.clone() {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(cores)
                .build()
                .unwrap();

            let param_string = format!("Cores - {} Len - {}", cores, len);
            let input = generate_random_sequence(len);

            quicksort_group.bench_function(BenchmarkId::new("Serial", param_string.clone()), |b| {
                b.iter_batched_ref(
                    || input.clone(),
                    |i| quicksort::<_, Serial>(black_box(i), 0, 0.0), // no need for pool since serial version shouldn't hook into rayon
                    SmallInput,
                );
            });

            quicksort_group.bench_function(BenchmarkId::new("Parallel", param_string), |b| {
                b.iter_batched_ref(
                    || input.clone(),
                    |i| pool.install(|| quicksort::<_, Parallel>(black_box(i), 0, 0.0)),
                    SmallInput,
                );
            });
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
