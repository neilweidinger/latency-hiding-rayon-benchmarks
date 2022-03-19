use rand::prelude::*;
use std::cell::UnsafeCell;

pub mod fib;
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
