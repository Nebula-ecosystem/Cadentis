use cadentis::time::sleep;
use std::time::{Duration, Instant};

#[cadentis::test]
async fn test_sleep_basic() {
    let start = Instant::now();
    sleep(Duration::from_millis(50)).await;
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(50),
        "Sleep should wait at least the specified duration"
    );
}

#[cadentis::test]
async fn test_sleep_zero_duration() {
    let start = Instant::now();
    sleep(Duration::from_millis(0)).await;
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(10),
        "Zero duration sleep should be fast"
    );
}

#[cadentis::test]
async fn test_sleep_in_function() {
    let start = Instant::now();
    sleep_and_record(start).await;
}

async fn sleep_and_record(start: Instant) {
    let elapsed_before = start.elapsed();
    sleep(Duration::from_millis(30)).await;
    let elapsed_after = start.elapsed();

    assert!(elapsed_after - elapsed_before >= Duration::from_millis(30));
}
