use benchmarks::quicksort::{generate_random_sequence, quicksort, quicksort_latency_hiding};
use benchmarks::{parse_latency_p, Parallel, Serial};
use clap::Parser;

enum QuicksortMode {
    LatencyHiding,
    Parallel,
    Serial,
}

fn parse_quicksort_mode(s: &str) -> Result<QuicksortMode, ParseQuicksortModeError> {
    match s {
        "latency-hiding" | "l" => Ok(QuicksortMode::LatencyHiding),
        "parallel" | "p" => Ok(QuicksortMode::Parallel),
        "serial" | "s" => Ok(QuicksortMode::Serial),
        _ => Err(ParseQuicksortModeError::ParseError),
    }
}

#[derive(Debug)]
enum ParseQuicksortModeError {
    ParseError,
}

impl std::error::Error for ParseQuicksortModeError {}

impl std::fmt::Display for ParseQuicksortModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseQuicksortModeError::ParseError => {
                write!(f, "Argument for quicksort mode could not be parsed")
            }
        }
    }
}

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "1000000")]
    n: usize,
    #[clap(short, long, parse(try_from_str = parse_quicksort_mode))]
    mode: QuicksortMode,
    #[clap(short = 'l', long)]
    latency_ms: u64,
    #[clap(short = 'p', long, parse(try_from_str = parse_latency_p))]
    latency_p: f32,
}

fn main() {
    let args = Args::parse();

    let mut v = generate_random_sequence(args.n);
    println!("Unsorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);

    match args.mode {
        QuicksortMode::LatencyHiding => {
            rayon::spawn_blocking_future(quicksort_latency_hiding(
                &mut v,
                args.latency_ms,
                args.latency_p,
            ));
        }
        QuicksortMode::Parallel => {
            quicksort::<_, Parallel>(&mut v, args.latency_ms, args.latency_p);
        }
        QuicksortMode::Serial => {
            quicksort::<_, Serial>(&mut v, args.latency_ms, args.latency_p);
        }
    }

    println!("Sorted: {:?}...{:?}", &v[..3], &v[v.len() - 3..]);
}
