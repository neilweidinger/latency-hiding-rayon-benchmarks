use benchmarks::map_reduce::map_reduce_fib;
use benchmarks::map_reduce::players::{
    generate_random_ids, identity_classic, identity_latency_hiding, map_classic,
    map_latency_hiding, reduce_classic, reduce_latency_hiding,
};
use benchmarks::map_reduce::{map_constrain, map_reduce, map_reduce_latency_hiding};
use benchmarks::parse_latency_p;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "10")]
    n: usize,
    #[clap(short, long)]
    hide_latency: bool,
    #[clap(short = 'l', long)]
    latency_ms: u64,
    #[clap(short = 'p', long, parse(try_from_str = parse_latency_p), default_value = "1.0")]
    latency_p: f32,
    #[clap(long)]
    players: bool,
    #[clap(short, long)]
    map_reduce_fib: bool,
}

fn players(args: Args) {
    let mut ids = generate_random_ids(args.n);
    let max_id = *ids.iter().max().unwrap();

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

    println!("Max ID: {}", max_id);
    println!("Winner: {:?}", winner);
}

fn map_reduce_fib(args: Args) {
    fn constrain<F>(f: F) -> F
    where
        F: for<'a> Fn(&'a mut u32) -> u32,
    {
        f
    }

    let map = constrain(|&mut n| map_reduce_fib::map(n, args.hide_latency, args.latency_ms));
    let mut fib_n = vec![30; args.n];

    println!(
        "Final value: {}",
        map_reduce(
            &mut fib_n,
            &map,
            &map_reduce_fib::reduce,
            &map_reduce_fib::identity
        )
    );
}

fn main() {
    let args = Args::parse();

    if args.players {
        players(args);
    } else if args.map_reduce_fib {
        map_reduce_fib(args);
    } else {
        panic!("Choose correct flag (need to refactor Clap arg logic)");
    }
}
