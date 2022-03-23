use crate::{incurs_latency, Joiner};
use async_io::Timer;
use async_recursion::async_recursion;
use futures::join;
use rand::distributions::Distribution;
use rand::distributions::Standard;
use std::time::Duration;

const SERIAL_CUTOFF: usize = 5 * 1024;

fn partition<T: Ord>(input: &mut [T]) -> usize {
    let pivot_index = input.len() - 1;
    let mut swap = 0;

    for i in 0..pivot_index {
        if input[i] <= input[pivot_index] {
            if swap != i {
                input.swap(swap, i);
            }

            swap += 1;
        }
    }

    if swap != pivot_index {
        input.swap(swap, pivot_index);
    }

    swap
}

pub fn quicksort<T: Ord + Send, J: Joiner>(input: &mut [T], latency_ms: u64, latency_p: f32) {
    if input.len() <= SERIAL_CUTOFF {
        input.sort_unstable();
    } else {
        if incurs_latency(latency_p) {
            std::thread::sleep(Duration::from_millis(latency_ms));
        }

        let mid = partition(input);
        let (left, right) = input.split_at_mut(mid);

        J::join(
            || quicksort::<T, J>(left, latency_ms, latency_p),
            || quicksort::<T, J>(right, latency_ms, latency_p),
        );
    }
}

#[async_recursion]
pub async fn quicksort_latency_hiding<T: Ord + Send>(
    input: &mut [T],
    latency_ms: u64,
    latency_p: f32,
) {
    if input.len() <= SERIAL_CUTOFF {
        input.sort_unstable();
    } else {
        if incurs_latency(latency_p) {
            Timer::after(Duration::from_millis(latency_ms)).await;
        }

        let mid = partition(input);
        let (left, right) = input.split_at_mut(mid);

        join!(
            quicksort_latency_hiding(left, latency_ms, latency_p),
            quicksort_latency_hiding(right, latency_ms, latency_p),
        );
    }
}

pub fn generate_random_sequence(len: usize) -> Vec<i32> {
    let rng = rand::thread_rng();
    Standard.sample_iter(rng).take(len).collect()
}
