use benchmarks::map_reduce::example::{
    identity_classic, identity_latency_hiding, map_classic, map_latency_hiding, reduce_classic,
    reduce_latency_hiding,
};
use benchmarks::map_reduce::{map_constrain, map_reduce, map_reduce_latency_hiding};
use benchmarks::parse_latency_p;
use clap::Parser;
use rand::distributions::{Distribution, Uniform};

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    n: usize,
    #[clap(short, long)]
    hide_latency: bool,
    #[clap(short = 'l', long)]
    latency_ms: u64,
    #[clap(short = 'p', long, parse(try_from_str = parse_latency_p))]
    latency_p: f32,
}

fn generate_random_ids(len: usize) -> Vec<usize> {
    let rng = rand::thread_rng();
    let dist = Uniform::from(0..=100);
    dist.sample_iter(rng).take(len).collect()
}

fn main() {
    let args = Args::parse();

    let mut ids = generate_random_ids(args.n);
    let max_id = *ids.iter().max().unwrap();
    println!("IDs: {:?}", ids);

    let winner = if args.hide_latency {
        let map =
            map_constrain(|id: &mut usize| map_latency_hiding(id, args.latency_ms, args.latency_p));
        let reduce = |player_1, player_2| {
            reduce_latency_hiding(player_1, player_2, args.latency_ms, args.latency_p)
        };

        rayon::spawn_blocking_future(map_reduce_latency_hiding(
            &mut ids,
            &map,
            &reduce,
            &identity_latency_hiding,
        ))
    } else {
        let map = |id: &mut usize| map_classic(id, args.latency_ms, args.latency_p);
        let reduce = |player_1, player_2| {
            reduce_classic(player_1, player_2, args.latency_ms, args.latency_p)
        };
        let identity = identity_classic;

        map_reduce(&mut ids, &map, &reduce, &identity)
    };

    println!("Max ID: {max_id}");
    println!("Winner: {winner:?}");
}
