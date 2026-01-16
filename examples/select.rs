//! Example: select on multiple futures with Cadentis

use cadentis::select;
use cadentis::time::sleep;
use std::time::Duration;

#[cadentis::main]
async fn main() {
    // Create two sleep futures with different durations
    let fut1 = sleep(Duration::from_millis(500));
    let fut2 = sleep(Duration::from_millis(1000));
    // Wait for the first future to complete using Cadentis select!
    select! {
        fut1 => |v| { println!("fut1 finished first") },
        fut2 => |v| { println!("fut2 finished first") },
    }
}
