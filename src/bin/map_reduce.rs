use benchmarks::map_reduce::{map_reduce, map_reduce_fib};
use benchmarks::{ExecutionMode, Parallel, ParallelLH, Serial};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long, arg_enum)]
    mode: ExecutionMode,
    #[clap(short, long, default_value = "10")]
    n: usize,
    #[clap(short, long)]
    latency_ms: Option<u64>,
    /// Defaults to number of cores on machine
    #[clap(short, long)]
    cores: Option<usize>,
}

fn main() {
    let args = Args::parse();

    let mut i = vec![30; args.n];

    if let Some(cores) = args.cores {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .build_global()
            .unwrap();
    }

    let r = match args.mode {
        ExecutionMode::LatencyHiding => {
            let map = |n: &mut u32| map_reduce_fib::map::<ParallelLH>(*n, args.latency_ms);

            map_reduce::<ParallelLH, _, _, _, _, _>(
                &mut i,
                &map,
                &map_reduce_fib::reduce,
                &map_reduce_fib::identity,
            )
        }
        ExecutionMode::Parallel => {
            let map = |n: &mut u32| map_reduce_fib::map::<Parallel>(*n, args.latency_ms);

            map_reduce::<Parallel, _, _, _, _, _>(
                &mut i,
                &map,
                &map_reduce_fib::reduce,
                &map_reduce_fib::identity,
            )
        }
        ExecutionMode::Serial => {
            let map = |n: &mut u32| map_reduce_fib::map::<Serial>(*n, args.latency_ms);

            map_reduce::<Serial, _, _, _, _, _>(
                &mut i,
                &map,
                &map_reduce_fib::reduce,
                &map_reduce_fib::identity,
            )
        }
    };

    println!("Final value: {}", r);
}
