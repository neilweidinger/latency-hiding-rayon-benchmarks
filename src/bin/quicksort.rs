use benchmarks::quicksort::{generate_random_sequence, quicksort};
use benchmarks::{build_global_threadpool, ExecutionMode, Parallel, ParallelLH, Serial};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long, arg_enum)]
    mode: ExecutionMode,
    #[clap(short, long, default_value = "8000000")]
    n: usize,
    #[clap(short, long)]
    latency_ms: Option<u64>,
    /// Defaults to number of cores on machine
    #[clap(short, long)]
    cores: Option<usize>,
    /// In multiples of MB. Defaults to Rust stack size default, which is 2MB.
    #[clap(short, long)]
    stack_size: Option<usize>,
}

fn main() {
    let args = Args::parse();

    let mut v = generate_random_sequence(args.n);
    println!("Unsorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);

    build_global_threadpool(args.cores, args.stack_size);

    match args.mode {
        ExecutionMode::LatencyHiding => {
            quicksort::<ParallelLH, _>(&mut v, args.latency_ms);
        }
        ExecutionMode::Parallel => {
            quicksort::<Parallel, _>(&mut v, args.latency_ms);
        }
        ExecutionMode::Serial => {
            quicksort::<Serial, _>(&mut v, args.latency_ms);
        }
    }

    println!("Sorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);
}
