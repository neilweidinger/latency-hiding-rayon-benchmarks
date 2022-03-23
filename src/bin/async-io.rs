use async_io::Timer;
use futures::join;
use std::time::Duration;

async fn future_1() {
    println!("Starting future 1...");
    Timer::after(Duration::from_secs(1)).await;
    println!("Finished future 1!");
}

async fn future_2() {
    println!("Starting future 2...");
    Timer::after(Duration::from_secs(1)).await;
    println!("Finished future 2!");
}

fn main() {
    println!("Starting...");
    rayon::spawn_blocking_future(async { join!(future_1(), future_2()) });
    println!("Finished!");
}
