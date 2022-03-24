use crate::{Joiner, Parallel};
use async_recursion::async_recursion;
use futures::join;
use std::future::Future;

pub mod map_reduce_fib;
pub mod players;

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
    if items.len() == 0 {
        return identity();
    } else if items.len() == 1 {
        return map(&mut items[0]);
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
    if items.len() == 0 {
        return identity().await.into();
    } else if items.len() == 1 {
        return map(&mut items[0]).await.into();
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
