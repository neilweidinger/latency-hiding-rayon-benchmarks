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
    use crate::{incur_latency, Joiner, Serial};

    pub fn map<J: Joiner>(n: u32, latency_ms: Option<u64>) -> u32 {
        if let Some(latency_ms) = latency_ms {
            incur_latency::<J>(latency_ms); // incur latency, if specified
        }

        fib::<Serial>(n, 0, 0.0).0
    }

    pub fn reduce(f1: u32, f2: u32) -> u32 {
        ((f1).wrapping_add(f2)) % 1_000_000_000
    }

    pub fn identity() -> u32 {
        0
    }
}