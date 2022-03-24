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

    (*id, ((*id).checked_mul(2).unwrap().try_into().unwrap())) // (id, predicted score)
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
