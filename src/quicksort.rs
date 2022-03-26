use crate::{Joiner, Work};
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

pub fn quicksort<J: Joiner, T: Ord + Send>(input: &mut [T], work: &Work) {
    if input.len() <= SERIAL_CUTOFF {
        // possibly do work, if specified, but only in root nodes of computation DAG
        work.do_work::<J>();

        input.sort_unstable();
    } else {
        let mid = partition(input);
        let (left, right) = input.split_at_mut(mid);

        J::join(
            || quicksort::<J, T>(left, work),
            || quicksort::<J, T>(right, work),
        );
    }
}

pub fn generate_random_sequence(len: usize) -> Vec<i32> {
    let rng = rand::thread_rng();
    Standard.sample_iter(rng).take(len).collect()
}
