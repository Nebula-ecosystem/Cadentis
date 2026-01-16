//! Example: Retry with interval using Cadentis

use cadentis::tools::retry;
use cadentis::task;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[cadentis::main]
async fn main() {
    // Shared counter for attempts
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();
    // Retry the async operation up to 3 times with interval
    let result = retry(3, move || {
        let attempts = attempts_clone.clone();
        task::spawn(async move {
            let n = attempts.fetch_add(1, Ordering::SeqCst);
            println!("Attempt {}", n + 1);
            Err::<(), &str>("fail")
        })
    })
    .set_interval(Duration::from_millis(100))
    .await;
    println!("Result: {:?}", result);
}
