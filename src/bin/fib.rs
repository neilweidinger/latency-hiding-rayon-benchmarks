use benchmarks::fib::{fib, fib_latency};
use benchmarks::Parallel;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    n: u32,
    #[clap(short, long)]
    dont_hide_latency: bool,
    #[clap(short = 'l', long)]
    latency_ms: u64,
    #[clap(short = 'p', long)]
    latency_p: f32,
}

fn main() {
    let args = Args::parse();

    let (fib, calls) = if args.dont_hide_latency {
        fib::<Parallel>(args.n, args.latency_ms, args.latency_p)
    } else {
        fib_latency(args.n, args.latency_ms, args.latency_p)
    };

    println!("result: {} calls: {}", fib, calls);
}
