use crate::{Joiner, Parallel};
use async_recursion::async_recursion;
use futures::join;
use std::future::Future;

pub fn map_reduce<T, MAP, REDUCE, ID, R>(
    items: &mut [T],
    map: &MAP,
    reduce: &REDUCE,
    identity: &ID,
) -> R
where
    T: Send,
    MAP: Fn(&mut T) -> R + Sync,
    REDUCE: Fn(R, R) -> R + Sync,
    ID: Fn() -> R + Sync,
    R: Send,
{
    if items.len() == 1 {
        return reduce(map(&mut items[0]), identity());
    } else if items.len() == 2 {
        let (s1, s2) = items.split_at_mut(items.len() / 2);
        let (ra, rb) = Parallel::join(|| map(&mut s1[0]), || map(&mut s2[0]));
        return reduce(ra, rb);
    }

    let (s1, s2) = items.split_at_mut(items.len() / 2);

    let (ra, rb) = Parallel::join(
        || map_reduce(s1, map, reduce, identity),
        || map_reduce(s2, map, reduce, identity),
    );

    reduce(ra, rb)
}

#[async_recursion]
pub async fn map_reduce_latency_hiding<'a, T, MAP, REDUCE, ID, FUT1, FUT2, FUT3>(
    items: &'a mut [T],
    map: &MAP,
    reduce: &REDUCE,
    identity: &ID,
) -> <FUT2 as Future>::Output
where
    T: Send,
    MAP: Fn(&'a mut T) -> FUT1 + Sync + 'a,
    REDUCE: Fn(<FUT2 as Future>::Output, <FUT2 as Future>::Output) -> FUT2 + Sync,
    ID: Fn() -> FUT3 + Sync,
    FUT1: Future + Send,
    <FUT1 as Future>::Output: Send + Into<<FUT2 as Future>::Output>,
    FUT2: Future + Send,
    <FUT2 as Future>::Output: Send,
    FUT3: Future + Send,
    <FUT3 as Future>::Output: Send + Into<<FUT2 as Future>::Output>,
{
    if items.len() == 1 {
        let (ra, rb) = join!(map(&mut items[0]), identity());
        return reduce(ra.into(), rb.into()).await;
    } else if items.len() == 2 {
        let (s1, s2) = items.split_at_mut(items.len() / 2);
        let (ra, rb) = join!(map(&mut s1[0]), map(&mut s2[0]));
        return reduce(ra.into(), rb.into()).await;
    }

    let (s1, s2) = items.split_at_mut(items.len() / 2);

    let (ra, rb) = join!(
        map_reduce_latency_hiding(s1, map, reduce, identity),
        map_reduce_latency_hiding(s2, map, reduce, identity),
    );

    reduce(ra, rb).await
}

/// Just needed so that we can help the borrow checker with specifying lifetimes
pub fn map_constrain<'a, F, FUT, R>(f: F) -> F
where
    F: Fn(&'a mut usize) -> FUT + Sync + 'a,
    FUT: Future<Output = R> + 'a,
{
    f
}

pub mod example {
    use crate::incurs_latency;
    use async_io::Timer;
    use rand::distributions::{Distribution, Uniform};
    use std::time::Duration;

    type Player = (usize, i32); // ID, estimated score
    const IDENTITY: Player = (usize::MAX, i32::MAX);

    pub fn map_classic(id: &mut usize, latency_ms: u64, latency_p: f32) -> Player {
        // pretend this is a network call, e.g. to calculate a player IDs predicted score
        if incurs_latency(latency_p) {
            std::thread::sleep(Duration::from_millis(latency_ms));
        }

        (*id, ((*id).checked_mul(2).unwrap().try_into().unwrap())) // (id, predicted score)
    }

    pub fn reduce_classic(
        player_1: Player,
        player_2: Player,
        latency_ms: u64,
        latency_p: f32,
    ) -> Player {
        assert!(
            !(player_1 == IDENTITY && player_2 == IDENTITY),
            "Both players in reduction cannot be idendity"
        );

        // avoid latency incurring calculation if we receive idendity
        if player_1 == IDENTITY {
            return player_2;
        } else if player_2 == IDENTITY {
            return player_1;
        }

        // pretend this is a network call, e.g. to calculate a predicted winner between two players
        if incurs_latency(latency_p) {
            std::thread::sleep(Duration::from_millis(latency_ms));
        }

        if player_2.1 > player_1.1 {
            player_2
        } else {
            player_1
        }
    }

    pub fn identity_classic() -> Player {
        IDENTITY
    }

    pub async fn map_latency_hiding(id: &mut usize, latency_ms: u64, latency_p: f32) -> Player {
        // pretend this is a network call, e.g. to calculate a player IDs predicted score
        if incurs_latency(latency_p) {
            Timer::after(Duration::from_millis(latency_ms)).await;
        }

        // Do some CPU bound work (to get Rayon to create more jobs and all threads to have
        // something to do)
        // let mut v = generate_random_ids(100000);
        // crate::quicksort::quicksort::<_, crate::Parallel>(&mut v, 0, 0.0);

        (
            *id,
            ((*id + v[50_000])
                .checked_mul(2)
                .unwrap()
                .try_into()
                .unwrap()),
        ) // (id, predicted score)
    }

    pub async fn reduce_latency_hiding(
        player_1: Player,
        player_2: Player,
        latency_ms: u64,
        latency_p: f32,
    ) -> Player {
        assert!(
            !(player_1 == IDENTITY && player_2 == IDENTITY),
            "Both players in reduction cannot be idendity"
        );

        // avoid latency incurring calculation if we receive idendity
        if player_1 == IDENTITY {
            return player_2;
        } else if player_2 == IDENTITY {
            return player_1;
        }

        // pretend this is a network call, e.g. to calculate a predicted winner between two players
        if incurs_latency(latency_p) {
            Timer::after(Duration::from_millis(latency_ms)).await;
        }

        if player_2.1 > player_1.1 {
            player_2
        } else {
            player_1
        }
    }

    pub async fn identity_latency_hiding() -> Player {
        IDENTITY
    }

    pub fn generate_random_ids(len: usize) -> Vec<usize> {
        let rng = rand::thread_rng();
        let dist = Uniform::from(0..=100);
        dist.sample_iter(rng).take(len).collect()
    }
}
