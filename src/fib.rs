use crate::{Joiner, Serial, Work};
use async_io::Timer;
use async_recursion::async_recursion;
use futures::join;
use std::time::Duration;

#[must_use]
pub fn fib<J: Joiner>(n: u32, work: &Work, serial_cutoff: u32) -> (u32, u32) {
    if n <= 1 {
        // possibly do work, if specified, but only in root nodes of computation DAG
        work.do_work::<J>();

        return (n, 1);
    }

    if J::is_parallel() && n <= serial_cutoff {
        return fib::<Serial>(n, work, serial_cutoff);
    }

    let ((fib1, calls1), (fib2, calls2)) = J::join(
        || fib::<J>(n - 1, work, serial_cutoff),
        || fib::<J>(n - 2, work, serial_cutoff),
    );

    (fib1 + fib2, calls1 + calls2)
}

#[async_recursion]
pub async fn fib_single_future(n: u32, latency_ms: Option<u64>) -> (u32, u32) {
    if n <= 1 {
        return (n, 1);
    }

    if let Some(latency_ms) = latency_ms {
        // await future directly (don't spawn seperate FutureJob)
        Timer::after(Duration::from_millis(latency_ms)).await;
    }

    // Here it's fine to use the join! macro, because there is no compute bound work we want to run
    // in parallel. It is entirely I/O bound (no compute, pure latency), so running the recursive
    // computation concurrently as if it were on a single core is perfectly fine.
    //
    // To express parallelism we need a call to `join_async` or `FutureJob::spawn` instead: if we
    // just use the join! macro here, no other FutureJobs get created, meaning no other worker
    // threads can steal to help drive progress of work. A single future is basically serial in
    // nature, progress is made through the state machine serially (a future is basically a
    // cooperative greenthread. another way to think of it conceptually is that a future state
    // machine is like multiple cooperatively scheduled threads on a single core machine. await
    // points represent points where threads cooperatively yield, and nested futures are like other
    // threads that can be scheduled on this single core machine. polling a future is like spawning
    // a thread (hopefully this thread does useful work, otherwise if it blocks the thread state
    // changes to waiting and another thread is resumed), and awaiting a future is like blocking on
    // a future (at which point you can be swapped out by another thread) so calling join! is like
    // spawning multiple threads on this single core machine - they may run concurrently but never
    // in parallel). A call to the join! macro means that the futures can execute concurrently (but
    // serially, i.e not in parallel). That means in order for worker threads to perform additional
    // work in *parallel* in between suspension points of a given future, (await suspension points
    // mark points where threads can tend to other jobs, but if we only use the join! macro there
    // will be no other jobs to tend to) additional FutureJobs need to be created (same idea as
    // spawning Tasks with Tokio / other executors).
    //
    // Typically you'll want to just call `FutureJob::spawn`, since you'll likely be in a
    // non-async compute bound context and just want to indicate to the Rayon scheduler that you'll
    // be injecting latency into this primarily compute bound workload (essentially indicating that
    // the job can yield/suspend, so that Rayon worker thread can work on other things in the
    // meantime).
    let (ra, rb) = join!(
        fib_single_future(n - 1, latency_ms),
        fib_single_future(n - 2, latency_ms)
    );

    (ra.0 + rb.0, ra.1 + rb.1 + 1)
}
