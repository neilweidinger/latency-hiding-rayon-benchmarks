use benchmarks::quicksort::{generate_random_sequence, quicksort};
use benchmarks::{
    build_global_threadpool, parse_latency_p, ExecutionMode, Parallel, ParallelLH, Serial, Work,
};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long, arg_enum)]
    mode: ExecutionMode,
    #[clap(short, long, default_value = "8000000")]
    n: usize,
    #[clap(short, long)]
    latency_ms: Option<u64>,
    #[clap(short = 'p', long, parse(try_from_str = parse_latency_p))]
    latency_p: Option<f32>,
    /// Defaults to number of cores on machine
    #[clap(short, long)]
    cores: Option<usize>,
    /// In multiples of MB. Defaults to Rust stack size default, which is 2MB.
    #[clap(short, long)]
    stack_size: Option<usize>,
}

fn main() {
    let args = Args::parse();
    let work = Work::new(args.latency_ms, args.latency_p);

    let mut v = generate_random_sequence(args.n);
    println!("Unsorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);

    build_global_threadpool(args.cores, args.stack_size);

    match args.mode {
        ExecutionMode::LatencyHiding => {
            quicksort::<ParallelLH, _>(&mut v, &work);
        }
        ExecutionMode::Parallel => {
            quicksort::<Parallel, _>(&mut v, &work);
        }
        ExecutionMode::Serial => {
            quicksort::<Serial, _>(&mut v, &work);
        }
    }

    println!("Sorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);
}
