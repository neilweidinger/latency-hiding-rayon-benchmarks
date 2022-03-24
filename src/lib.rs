use async_io::Timer;
use pin_utils::pin_mut;
use rand::prelude::*;
use std::cell::UnsafeCell;
use std::str::FromStr;
use std::time::Duration;

pub mod fib;
pub mod map_reduce;
pub mod quicksort;

thread_local! {
    static RNG: UnsafeCell<ThreadRng> = UnsafeCell::new(rand::thread_rng());
}

pub enum ExecutionMode {
    LatencyHiding,
    Parallel,
    Serial,
}

pub fn parse_execution_mode(s: &str) -> Result<ExecutionMode, ParseExecutionModeError> {
    match s {
        "latency-hiding" | "l" => Ok(ExecutionMode::LatencyHiding),
        "parallel" | "p" => Ok(ExecutionMode::Parallel),
        "serial" | "s" => Ok(ExecutionMode::Serial),
        _ => Err(ParseExecutionModeError::ParseError),
    }
}

#[derive(Debug)]
pub enum ParseExecutionModeError {
    ParseError,
}

impl std::error::Error for ParseExecutionModeError {}

impl std::fmt::Display for ParseExecutionModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseExecutionModeError::ParseError => {
                write!(f, "Argument for execution mode could not be parsed")
            }
        }
    }
}

pub trait Joiner {
    #[must_use]
    fn is_parallel() -> bool;

    #[must_use]
    fn is_latency_hiding() -> bool;

    fn join<A, B, RA, RB>(oper_a: A, oper_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send;
}

pub struct Serial;

impl Joiner for Serial {
    #[must_use]
    fn is_parallel() -> bool {
        false
    }

    #[must_use]
    fn is_latency_hiding() -> bool {
        false
    }

    fn join<A, B, RA, RB>(oper_a: A, oper_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        let ra = oper_a();
        let rb = oper_b();

        (ra, rb)
    }
}

pub struct Parallel;

impl Joiner for Parallel {
    #[must_use]
    fn is_parallel() -> bool {
        true
    }

    #[must_use]
    fn is_latency_hiding() -> bool {
        false
    }

    fn join<A, B, RA, RB>(oper_a: A, oper_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        rayon::join(oper_a, oper_b)
    }
}

pub struct ParallelLH;

impl Joiner for ParallelLH {
    #[must_use]
    fn is_parallel() -> bool {
        true
    }

    #[must_use]
    fn is_latency_hiding() -> bool {
        true
    }

    fn join<A, B, RA, RB>(oper_a: A, oper_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        rayon::join(oper_a, oper_b)
    }
}

#[derive(Debug)]
pub enum ParseLatencyPError {
    OutOfBounds,
    ParseError,
}

impl std::error::Error for ParseLatencyPError {}

impl std::fmt::Display for ParseLatencyPError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseLatencyPError::OutOfBounds => {
                write!(f, "Latency p not in range [0.0, 1.0]")
            }
            ParseLatencyPError::ParseError => {
                write!(f, "Argument for latency p could not be parsed")
            }
        }
    }
}

pub fn parse_latency_p(s: &str) -> Result<f32, ParseLatencyPError> {
    match f32::from_str(s) {
        Ok(f) if f >= 0.0 && f <= 1.0 => Ok(f),
        Ok(_) => Err(ParseLatencyPError::OutOfBounds),
        Err(_) => Err(ParseLatencyPError::ParseError),
    }
}

/// Returns true if latency is incurred according to given p (probability that latency is incurred)
#[must_use]
fn incurs_latency(p: f32) -> bool {
    RNG.with(|rng| {
        let r: f32 = unsafe { &mut *rng.get() }.gen();
        r < p
    })
}

fn incur_latency<J: Joiner>(latency_ms: u64) {
    if J::is_latency_hiding() {
        let future_job = rayon::FutureJob::new(Timer::after(Duration::from_millis(latency_ms)));
        pin_mut!(future_job);
        future_job.spawn().await_future_job();
    } else {
        std::thread::sleep(Duration::from_millis(latency_ms));
    }
}

// This really should just be used in a synthetic param sweep bench
// /// if incurs_latency(latency_p) {
// ///     if latency_hiding {
// ///         spawn_blocking_future // incur latency, latency hiding
// ///     }
// ///     else {
// ///         sleep // incur latency, not latency hiding
// ///     }
// /// }
// /// else {
// ///     sleep // "compute" instead of incurring latency, e.g. we need to compute some required data
// /// }
// pub fn incur_latency_or_compute(latency_ms: u64, latency_p: f32, latency_hiding: bool) {
//     if incurs_latency(latency_p) && latency_hiding {
//         rayon::spawn_blocking_future(Timer::after(Duration::from_millis(latency_ms)));
//     } else {
//         std::thread::sleep(Duration::from_millis(latency_ms));
//     }
// }
