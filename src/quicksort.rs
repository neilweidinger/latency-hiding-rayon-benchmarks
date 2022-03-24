use crate::{incur_latency, Joiner};
use rand::distributions::Distribution;
use rand::distributions::Standard;

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

pub fn quicksort<J: Joiner, T: Ord + Send>(input: &mut [T], latency_ms: Option<u64>) {
    if input.len() <= SERIAL_CUTOFF {
        if let Some(latency_ms) = latency_ms {
            incur_latency::<J>(latency_ms); // incur latency, if specified
        }

        input.sort_unstable();
    } else {
        let mid = partition(input);
        let (left, right) = input.split_at_mut(mid);

        J::join(
            || quicksort::<J, T>(left, latency_ms),
            || quicksort::<J, T>(right, latency_ms),
        );
    }
}

pub fn generate_random_sequence(len: usize) -> Vec<i32> {
    let rng = rand::thread_rng();
    Standard.sample_iter(rng).take(len).collect()
}
