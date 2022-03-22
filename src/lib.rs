use rand::prelude::*;
use std::cell::UnsafeCell;
use std::str::FromStr;

pub mod fib;
pub mod map_reduce;
pub mod quicksort;

thread_local! {
    static RNG: UnsafeCell<ThreadRng> = UnsafeCell::new(rand::thread_rng());
}

/// Returns true if latency is incurred according to given p (probability that latency is incurred)
pub fn incurs_latency(p: f32) -> bool {
    RNG.with(|rng| {
        let r: f32 = unsafe { &mut *rng.get() }.gen();
        r < p
    })
}

pub trait Joiner {
    #[must_use]
    fn is_parallel() -> bool;

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

impl std::error::Error for ParseLatencyPError {}

pub fn parse_latency_p(s: &str) -> Result<f32, ParseLatencyPError> {
    match f32::from_str(s) {
        Ok(f) if f >= 0.0 && f <= 1.0 => Ok(f),
        Ok(_) => Err(ParseLatencyPError::OutOfBounds),
        Err(_) => Err(ParseLatencyPError::ParseError),
    }
}
