use benchmarks::fib::{fib, fib_single_future};
use benchmarks::{
    build_global_threadpool, parse_latency_p, ExecutionMode, Parallel, ParallelLH, Serial, Work,
};
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
    #[clap(short = 'p', long, parse(try_from_str = parse_latency_p))]
    latency_p: Option<f32>,
    #[clap(short, long, default_value = "25")]
    serial_cutoff: u32,
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

    build_global_threadpool(args.cores, args.stack_size);

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
            ExecutionMode::LatencyHiding => fib::<ParallelLH>(args.n, &work, args.serial_cutoff),
            ExecutionMode::Parallel => fib::<Parallel>(args.n, &work, args.serial_cutoff),
            ExecutionMode::Serial => fib::<Serial>(args.n, &work, args.serial_cutoff),
        }
    };

    println!("result: {} calls: {}", fib, calls);
}
