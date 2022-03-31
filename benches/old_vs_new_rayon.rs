use benchmarks::map_reduce::map_reduce;
use benchmarks::map_reduce::map_reduce_fib;
use benchmarks::{Joiner, Parallel, ParallelOldRayon, Serial, Work};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// (fib n, serial_cutoff)
type FibSettings = (u32, u32);

const STACK_SIZE_MB: usize = 24; // set a large stack size to avoid overflow
const LATENCY_MS: Option<u64> = None; // no latency, test pure compute
const LEN: usize = 200;
const FIB_SETTINGS: FibSettings = (35, 25);

fn param_string(
    length: usize,
    latency_ms: Option<u64>,
    cores: usize,
    fib_settings: FibSettings,
) -> String {
    format!(
        "Length: {} | Latency ms: {} | Cores: {} | Fib N: {} | Cutoff: {}",
        length,
        latency_ms.unwrap_or(0),
        cores,
        fib_settings.0,
        fib_settings.1
    )
}

fn map_reduce_fib<J: Joiner>(
    input: &mut [u32],
    latency_ms: Option<u64>,
    serial_cutoff: u32,
) -> u32 {
    fn constrain<F>(f: F) -> F
    where
        F: for<'a> Fn(&'a mut u32) -> u32,
    {
        f
    }

    let map = constrain(|&mut n| {
        map_reduce_fib::map::<J>(n, &Work::new(latency_ms, None), serial_cutoff)
    });

    map_reduce::<J, _, _, _, _, _>(
        input,
        &map,
        &map_reduce_fib::reduce,
        &map_reduce_fib::identity,
    )
}

fn map_reduce_fib_bench(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("Old vs New Rayon");

    let (fib_n, serial_cutoff) = FIB_SETTINGS;
    let mut input = vec![fib_n; LEN];

    let num_cores = {
        let step = if num_cpus::get() <= 10 { 2 } else { 5 };
        [1].into_iter()
            .chain((step..=num_cpus::get()).step_by(step))
    };

    // Serial benchmark
    bench_group.bench_with_input(
        BenchmarkId::new(
            "Serial",
            param_string(LEN, LATENCY_MS, 1, (fib_n, serial_cutoff)),
        ),
        &LATENCY_MS,
        |b, &l| {
            b.iter(|| {
                map_reduce_fib::<Serial>(
                    black_box(&mut input),
                    black_box(l),
                    black_box(serial_cutoff),
                )
            })
        },
    );

    for cores in num_cores.clone() {
        let old_pool = rayon_old::ThreadPoolBuilder::new()
            .num_threads(cores)
            .stack_size(STACK_SIZE_MB * 1024 * 1024)
            .build()
            .unwrap();

        // Old Rayon benchmark
        bench_group.bench_with_input(
            BenchmarkId::new(
                "Old Rayon",
                param_string(LEN, LATENCY_MS, cores, (fib_n, serial_cutoff)),
            ),
            &LATENCY_MS,
            |b, &l| {
                old_pool.install(|| {
                    b.iter(|| {
                        map_reduce_fib::<ParallelOldRayon>(
                            black_box(&mut input),
                            black_box(l),
                            black_box(serial_cutoff),
                        )
                    })
                })
            },
        );

        drop(old_pool);

        let new_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .stack_size(STACK_SIZE_MB * 1024 * 1024)
            .build()
            .unwrap();

        // New Rayon benchmark
        bench_group.bench_with_input(
            BenchmarkId::new(
                "New Rayon",
                param_string(LEN, LATENCY_MS, cores, (fib_n, serial_cutoff)),
            ),
            &LATENCY_MS,
            |b, &l| {
                new_pool.install(|| {
                    b.iter(|| {
                        map_reduce_fib::<Parallel>(
                            black_box(&mut input),
                            black_box(l),
                            black_box(serial_cutoff),
                        )
                    })
                })
            },
        );
    }

    bench_group.finish();
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = map_reduce_fib_bench
}
criterion_main!(benches);
