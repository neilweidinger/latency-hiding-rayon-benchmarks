use benchmarks::fib::{fib, fib_single_future};
use benchmarks::{ExecutionMode, Parallel, ParallelLH, Serial};
use clap::Parser;
use pin_utils::pin_mut;

#[derive(Parser)]
struct Args {
    #[clap(short, long, arg_enum)]
    mode: ExecutionMode,
    #[clap(short, long)]
    single_future_mode: bool,
    #[clap(short, long, default_value = "12")]
    n: u32,
    #[clap(short, long)]
    latency_ms: Option<u64>,
    #[clap(short, long)]
    serial_cutoff: Option<u32>,
    /// Defaults to number of cores on machine
    #[clap(short, long)]
    cores: Option<usize>,
}

fn main() {
    let args = Args::parse();

    if let Some(cores) = args.cores {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .build_global()
            .unwrap();
    }

    let (fib, calls) = if args.single_future_mode {
        let mut r: Option<(u32, u32)> = None;

        {
            let future_job = rayon::FutureJob::new(async {
                r = Some(fib_single_future(args.n, args.latency_ms).await)
            });
            pin_mut!(future_job);
            future_job.spawn().await_future_job();
        }

        r.unwrap()
    } else {
        match args.mode {
            ExecutionMode::LatencyHiding => {
                fib::<ParallelLH>(args.n, args.latency_ms, args.serial_cutoff)
            }
            ExecutionMode::Parallel => fib::<Parallel>(args.n, args.latency_ms, args.serial_cutoff),
            ExecutionMode::Serial => fib::<Serial>(args.n, args.latency_ms, args.serial_cutoff),
        }
    };

    println!("result: {} calls: {}", fib, calls);
}
