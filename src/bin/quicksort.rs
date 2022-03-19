use benchmarks::quicksort::{generate_random_sequence, quicksort, quicksort_latency};
use benchmarks::Parallel;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "1000")]
    n: usize,
    #[clap(short, long)]
    dont_hide_latency: bool,
    #[clap(short = 'l', long)]
    latency_ms: u64,
    #[clap(short = 'p', long)]
    latency_p: f32,
}

fn main() {
    let args = Args::parse();

    let mut v = generate_random_sequence(args.n);
    println!("Unsorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);

    if args.dont_hide_latency {
        quicksort::<_, Parallel>(&mut v, args.latency_ms, args.latency_p);
    } else {
        quicksort_latency(&mut v, args.latency_ms, args.latency_p);
    }

    println!("Sorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);
}
