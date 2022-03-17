use benchmarks::{fib, fib_latency, Parallel};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    n: u32,
    #[clap(short, long)]
    incur_latency: bool,
    #[clap(short, long)]
    latency_ms: u64,
    #[clap(short, long)]
    latency_p: f32,
}

fn main() {
    let args = Args::parse();

    let (fib, calls) = if args.incur_latency {
        fib::<Parallel>(args.n, args.latency_ms, args.latency_p)
    } else {
        fib_latency(args.n, args.latency_ms, args.latency_p)
    };

    println!("result: {} calls: {}", fib, calls);
}
