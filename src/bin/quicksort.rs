use benchmarks::quicksort::{generate_random_sequence, quicksort, quicksort_latency_hiding};
use benchmarks::{parse_latency_p, Parallel};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "1000000")]
    n: usize,
    #[clap(short, long)]
    hide_latency: bool,
    #[clap(short = 'l', long)]
    latency_ms: u64,
    #[clap(short = 'p', long, parse(try_from_str = parse_latency_p))]
    latency_p: f32,
}

fn main() {
    let args = Args::parse();

    let mut v = generate_random_sequence(args.n);
    println!("Unsorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);

    if args.hide_latency {
        rayon::spawn_blocking_future(quicksort_latency_hiding(
            &mut v,
            args.latency_ms,
            args.latency_p,
        ));
    } else {
        quicksort::<_, Parallel>(&mut v, args.latency_ms, args.latency_p);
    }

    println!("Sorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);
}
