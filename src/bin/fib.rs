use benchmarks::fib::{fib, fib_latency_hiding};
use benchmarks::{parse_latency_p, Parallel};
use clap::Parser;
use pin_utils::pin_mut;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    n: u32,
    #[clap(short, long)]
    hide_latency: bool,
    #[clap(short = 'l', long)]
    latency_ms: u64,
    #[clap(short = 'p', long, parse(try_from_str = parse_latency_p))]
    latency_p: f32,
}

fn main() {
    let args = Args::parse();

    let (fib, calls) = if args.hide_latency {
        let mut r: Option<(u32, u32)> = None;

        {
            let future_job = rayon::FutureJob::new(async {
                r = Some(fib_latency_hiding(args.n, args.latency_ms, args.latency_p).await)
            });
            pin_mut!(future_job);
            future_job.spawn().await_future_job();
        }

        r.unwrap()
    } else {
        fib::<Parallel>(args.n, args.latency_ms, args.latency_p)
    };

    println!("result: {} calls: {}", fib, calls);
}
