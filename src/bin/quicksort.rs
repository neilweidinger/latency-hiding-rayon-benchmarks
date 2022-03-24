use benchmarks::quicksort::{generate_random_sequence, quicksort};
use benchmarks::{parse_execution_mode, ExecutionMode, Parallel, ParallelLH, Serial};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long, parse(try_from_str = parse_execution_mode))]
    mode: ExecutionMode,
    #[clap(short, long, default_value = "10000000")]
    n: usize,
    #[clap(short, long)]
    latency_ms: Option<u64>,
}

fn main() {
    let args = Args::parse();

    let mut v = generate_random_sequence(args.n);
    println!("Unsorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);

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
