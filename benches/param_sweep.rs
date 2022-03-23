use benchmarks::fib::{fib, fib_latency_hiding};
use benchmarks::map_reduce::example::{
    generate_random_ids, identity_classic, identity_latency_hiding, map_classic,
    map_latency_hiding, reduce_classic, reduce_latency_hiding,
};
use benchmarks::map_reduce::{map_constrain, map_reduce, map_reduce_latency_hiding};
use benchmarks::quicksort::{generate_random_sequence, quicksort, quicksort_latency_hiding};
use benchmarks::Parallel;
use criterion::measurement::WallTime;
use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion,
};

const FIB_N: u32 = 8;
const QSORT_LEN: usize = 1_000_000;
const MAP_REDUCE_LEN: usize = 100_000;

const LATENCY_MS: [u64; 2] = [0, 20];
// const LATENCY_MS: [u64; 1] = [20];
// const LATENCY_MS: [u64; 8] = [0, 5, 10, 20, 50, 100, 250, 500];
const LATENCY_P: [f32; 2] = [0.5, 1.0];
// const LATENCY_P: [f32; 1] = [1.0];
// const LATENCY_P: [f32; 4] = [0.1, 0.5, 0.9, 1.0];

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
fn fib_lh(p: Params) {
    let _ = rayon::spawn_blocking_future(fib_latency_hiding(
        black_box(FIB_N),
        black_box(p.latency_ms),
        black_box(p.latency_p),
    ));
}

#[inline]
fn quicksort_classic(p: Params) {
    quicksort::<_, Parallel>(
        black_box(&mut generate_random_sequence(QSORT_LEN)),
        black_box(p.latency_ms),
        black_box(p.latency_p),
    );
}

#[inline]
fn quicksort_lh(p: Params) {
    let _ = rayon::spawn_blocking_future(quicksort_latency_hiding(
        black_box(&mut generate_random_sequence(QSORT_LEN)),
        black_box(p.latency_ms),
        black_box(p.latency_p),
    ));
}

fn map_reduce_classic(p: Params) {
    let map = |id: &mut usize| map_classic(id, p.latency_ms, p.latency_p);
    let reduce = |player_1, player_2| reduce_classic(player_1, player_2, p.latency_ms, p.latency_p);
    let identity = identity_classic;

    let _ = map_reduce(
        black_box(&mut generate_random_ids(MAP_REDUCE_LEN)),
        &map,
        &reduce,
        &identity,
    );
}

fn map_reduce_lh(p: Params) {
    let map = map_constrain(|id: &mut usize| map_latency_hiding(id, p.latency_ms, p.latency_p));
    let reduce =
        |player_1, player_2| reduce_latency_hiding(player_1, player_2, p.latency_ms, p.latency_p);

    let _ = rayon::spawn_blocking_future(map_reduce_latency_hiding(
        black_box(&mut generate_random_ids(MAP_REDUCE_LEN)),
        &map,
        &reduce,
        &identity_latency_hiding,
    ));
}

fn param_sweep(c: &mut Criterion) {
    fn run_param_sweep<F1: FnMut(Params), F2: FnMut(Params)>(
        latency_sweep_group: &mut BenchmarkGroup<WallTime>,
        mut classic: F1,
        mut latency_hiding: F2,
        override_latency_ms: Option<&[u64]>,
        override_latency_p: Option<&[f32]>,
    ) {
        let latency_ms_list = override_latency_ms.unwrap_or(&LATENCY_MS);
        let latency_p_list = override_latency_p.unwrap_or(&LATENCY_P);

        for &latency_ms in latency_ms_list {
            for &latency_p in latency_p_list {
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
    run_param_sweep(&mut fib_group, fib_classic, fib_lh, None, None);
    fib_group.finish();

    let mut quicksort_group = c.benchmark_group("Quicksort");
    run_param_sweep(
        &mut quicksort_group,
        quicksort_classic,
        quicksort_lh,
        None,
        None,
    );
    quicksort_group.finish();

    // let mut map_reduce_group = c.benchmark_group("MapReduce");
    // run_param_sweep(
    //     &mut map_reduce_group,
    //     map_reduce_classic,
    //     map_reduce_lh,
    //     None,
    //     None,
    // );
    // map_reduce_group.finish()
}

criterion_group! {
  name = benches;
  // config = Criterion::default().sample_size(35);
  config = Criterion::default().sample_size(10);
  targets = param_sweep
}
criterion_main!(benches);
