//! Example: Using Cadentis timer

use cadentis::time::sleep;
use std::time::Duration;

#[cadentis::main]
async fn main() {
    // Wait asynchronously for 1 second
    println!("Waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    println!("Done!");
}
