use crate::fib::fib;
use crate::Parallel;
use async_io::Timer;
use std::time::Duration;

pub fn map(n: u32, hide_latency: bool, latency_ms: u64) -> u32 {
    if hide_latency {
        rayon::spawn_blocking_future(async {
            Timer::after(Duration::from_millis(latency_ms)).await;
        });
    } else {
        std::thread::sleep(Duration::from_millis(latency_ms));
    }

    fib::<Parallel>(n, 0, 0.0).0
}

pub fn reduce(f1: u32, f2: u32) -> u32 {
    ((f1).wrapping_add(f2)) % 1_000_000_000
}

pub fn identity() -> u32 {
    0
}
