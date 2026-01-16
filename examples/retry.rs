//! Example: Using the retry utility


use cadentis::tools::retry;
use cadentis::task;

#[cadentis::main]
async fn main() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Shared counter for attempts
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();
    // Retry the async operation up to 5 times
    let result = retry(5, move || {
        let attempts = attempts_clone.clone();
        task::spawn(async move {
            let n = attempts.fetch_add(1, Ordering::SeqCst);
            println!("Attempt {}", n + 1);
            if n < 2 {
                Err::<&'static str, &str>("Failed")
            } else {
                Ok::<&'static str, &str>("Success!")
            }
        })
    }).await;
    println!("Result: {:?}", result);
}
