use benchmarks::fib::{fib, fib_latency_hiding};
use benchmarks::{parse_latency_p, Parallel};
use clap::Parser;

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
        // rayon::scope(|s| {
        //     s.spawn_future(async {
        //         r = Some(fib_latency_hiding(args.n, args.latency_ms, args.latency_p).await)
        //     });
        // });
        rayon::spawn_blocking_future(async {
            r = Some(fib_latency_hiding(args.n, args.latency_ms, args.latency_p).await)
        });
        r.unwrap()
    } else {
        fib::<Parallel>(args.n, args.latency_ms, args.latency_p)
    };

    println!("result: {} calls: {}", fib, calls);
}
