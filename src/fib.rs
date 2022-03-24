use crate::{incurs_latency, Joiner};
use async_io::Timer;
use async_recursion::async_recursion;
use futures::join;
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

    // Here it's fine to use the join! macro, because there is no compute bound work we want to run
    // in parallel. It is entirely I/O bound (no compute, pure latency), so running the recursive
    // computation concurrently as if it were on a single core is perfectly fine.
    //
    // To express parallelism we need a call to `join_async` instead: if we just use the join!
    // macro here, no other FutureJobs get created, meaning no other worker threads can steal to
    // help drive progress of work. A single future is basically serial in nature, progress is made
    // through the state machine serially (a future is basically a single cooperative greenthread.
    // i.e. conceptually a future state machine is like multiple cooperatively scheduled threads on
    // a single core machine. await points represent points where threads cooperatively yield, and
    // nested futures are like other threads that can be scheduled on this single core machine.
    // polling a future is like spawning a thread (hopefully this thread does useful work,
    // otherwise if it blocks the thread state changes to waiting and another thread is resumed),
    // and awaiting a future is like blocking on a future (at which point you can be swapped out by
    // another thread) so calling join! is like spawning multiple threads on this single core
    // machine - they may run concurrently but never in parallel). A call to the join! macro means
    // that the futures can execute concurrently (but serially, i.e not in parallel). That means in
    // order for worker threads to perform additional work in *parallel* in between suspension
    // points of a given future, (await suspension points mark points where threads can tend to
    // other jobs, but if we only use the join! macro there will be no other jobs to tend to)
    // additional FutureJobs need to be created (same idea as spawning Tasks with Tokio / other
    // executors).
    //
    // Although typically you'll want to just call `spawn_blocking_future`, since you'll likely be
    // in a non-async compute bound context and just want to indicate to the Rayon scheduler that
    // you'll be incurring latency (and can essentially yield, so that Rayon worker thread can work
    // on other things in the meantime).
    let (ra, rb) = join!(
        fib_latency_hiding(n - 1, latency_ms, latency_p),
        fib_latency_hiding(n - 2, latency_ms, latency_p)
    );

    (ra.0 + rb.0, ra.1 + rb.1 + 1)
}
