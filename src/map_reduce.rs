use crate::Joiner;

pub fn map_reduce<J, T, MAP, REDUCE, ID, R>(
    items: &mut [T],
    map: &MAP,
    reduce: &REDUCE,
    identity: &ID,
) -> R
where
    J: Joiner,
    T: Send,
    MAP: Fn(&mut T) -> R + Sync,
    REDUCE: Fn(R, R) -> R + Sync,
    ID: Fn() -> R + Sync,
    R: Send,
{
    if items.len() == 0 {
        return identity();
    } else if items.len() == 1 {
        return map(&mut items[0]);
    }

    let (s1, s2) = items.split_at_mut(items.len() / 2);
    let (ra, rb) = J::join(
        || map_reduce::<J, _, _, _, _, _>(s1, map, reduce, identity),
        || map_reduce::<J, _, _, _, _, _>(s2, map, reduce, identity),
    );

    reduce(ra, rb)
}

pub mod map_reduce_fib {
    use crate::fib::fib;
    use crate::{Joiner, Parallel, Work};

    pub fn map<J: Joiner>(n: u32, work: &Work, serial_cutoff: u32) -> u32 {
        // Possibly do work, if specified, but only in root nodes of computation DAG
        work.do_work::<J>();

        // Do only pure compute in fibonacci
        let fib_work = Work::new(None, None);

        fib::<Parallel>(n, &fib_work, serial_cutoff).0
    }

    pub fn reduce(f1: u32, f2: u32) -> u32 {
        ((f1).wrapping_add(f2)) % 1_000_000_000
    }

    pub fn identity() -> u32 {
        0
    }
}
