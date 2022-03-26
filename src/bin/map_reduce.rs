use benchmarks::map_reduce::{map_reduce, map_reduce_fib};
use benchmarks::{
    build_global_threadpool, parse_latency_p, ExecutionMode, Parallel, ParallelLH, Serial, Work,
};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long, arg_enum)]
    mode: ExecutionMode,
    #[clap(long, default_value = "10")]
    map_n: usize,
    #[clap(short, long)]
    latency_ms: Option<u64>,
    #[clap(short = 'p', long, parse(try_from_str = parse_latency_p))]
    latency_p: Option<f32>,
    #[clap(short, long, default_value = "30")]
    fib_n: u32,
    #[clap(short, long, default_value = "25")]
    serial_cutoff: u32,
    /// Defaults to number of cores on machine
    #[clap(short, long)]
    cores: Option<usize>,
    /// In multiples of MB. Defaults to Rust stack size default, which is 2MB.
    #[clap(long)]
    stack_size: Option<usize>,
}

fn main() {
    let args = Args::parse();
    let work = Work::new(args.latency_ms, args.latency_p);

    let mut i = vec![args.fib_n; args.map_n];

    build_global_threadpool(args.cores, args.stack_size);

    let r = match args.mode {
        ExecutionMode::LatencyHiding => {
            let map =
                |n: &mut u32| map_reduce_fib::map::<ParallelLH>(*n, &work, args.serial_cutoff);

            map_reduce::<ParallelLH, _, _, _, _, _>(
                &mut i,
                &map,
                &map_reduce_fib::reduce,
                &map_reduce_fib::identity,
            )
        }
        ExecutionMode::Parallel => {
            let map = |n: &mut u32| map_reduce_fib::map::<Parallel>(*n, &work, args.serial_cutoff);

            map_reduce::<Parallel, _, _, _, _, _>(
                &mut i,
                &map,
                &map_reduce_fib::reduce,
                &map_reduce_fib::identity,
            )
        }
        ExecutionMode::Serial => {
            let map = |n: &mut u32| map_reduce_fib::map::<Serial>(*n, &work, args.serial_cutoff);

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
