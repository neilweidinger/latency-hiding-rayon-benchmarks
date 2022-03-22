use crate::{incurs_latency, Joiner};
use async_io::Timer;
use async_recursion::async_recursion;
use std::time::Duration;

#[must_use]
pub fn fib<J: Joiner>(n: u32, latency_ms: u64, latency_p: f32) -> (u32, u32) {
    if n <= 1 {
        return (n, 1);
    }

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

#[async_recursion]
pub async fn fib_latency_hiding(n: u32, latency_ms: u64, latency_p: f32) -> (u32, u32) {
    if n <= 1 {
        return (n, 1);
    }

    if incurs_latency(latency_p) {
        Timer::after(Duration::from_millis(latency_ms)).await;
    }

    let (ra, rb) = rayon::join_async(
        fib_latency_hiding(n - 1, latency_ms, latency_p),
        fib_latency_hiding(n - 2, latency_ms, latency_p),
    );
    // let (ra, rb) = futures::join!(
    //     fib_latency_helper(n - 1, latency),
    //     fib_latency_helper(n - 2, latency)
    // );

    (ra.0 + rb.0, ra.1 + rb.1 + 1)
}
