use async_io::Timer;
use async_recursion::async_recursion;
use rand::prelude::*;
use std::cell::UnsafeCell;
use std::time::Duration;

thread_local! {
    static RNG: UnsafeCell<ThreadRng> = UnsafeCell::new(rand::thread_rng());
}

/// Returns true if latency is incurred according to given p (probability that latency is incurred)
fn incurs_latency(p: f32) -> bool {
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

#[must_use]
pub fn fib<J: Joiner>(n: u32, latency_ms: u64, latency_p: f32) -> (u32, u32) {
    if n <= 1 {
        return (n, 1);
    }

    // if n < 10 && J::is_parallel() {
    //     return fib::<Serial>(n); // cross over to serial execution
    // }

    // incur latency here if probability satisfied
    // TODO: branch prediction overhead?
    if incurs_latency(latency_p) {
        std::thread::sleep(Duration::from_millis(latency_ms));
    }

    let (ra, rb) = J::join(
        || fib::<J>(n - 1, latency_ms, latency_p),
        || fib::<J>(n - 2, latency_ms, latency_p),
    );

    (ra.0 + rb.0, ra.1 + rb.1 + 1)
}

pub fn fib_latency(n: u32, latency_ms: u64, latency_p: f32) -> (u32, u32) {
    if n <= 1 {
        return (n, 1);
    }

    let (ra, rb) = rayon::join_async(
        fib_latency_helper(n - 1, latency_ms, latency_p),
        fib_latency_helper(n - 2, latency_ms, latency_p),
    );

    (ra.0 + rb.0, ra.1 + rb.1 + 1)
}

#[async_recursion]
pub async fn fib_latency_helper(n: u32, latency_ms: u64, latency_p: f32) -> (u32, u32) {
    if n <= 1 {
        return (n, 1);
    }

    // incur latency here if probability satisfied
    // TODO: branch prediction overhead?
    if incurs_latency(latency_p) {
        Timer::after(Duration::from_millis(latency_ms)).await;
    }

    let (ra, rb) = rayon::join_async(
        fib_latency_helper(n - 1, latency_ms, latency_p),
        fib_latency_helper(n - 2, latency_ms, latency_p),
    );
    // let (ra, rb) = futures::join!(
    //     fib_latency_helper(n - 1, latency),
    //     fib_latency_helper(n - 2, latency)
    // );

    (ra.0 + rb.0, ra.1 + rb.1 + 1)
}
