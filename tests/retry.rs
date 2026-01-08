use cadentis::tools::retry;
use cadentis::{RuntimeBuilder, Task};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_retry_succeeds_before_limit() {
    let rt = RuntimeBuilder::new().enable_io().build();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let result = rt.block_on(async {
        retry(5, || {
            let attempts_clone = attempts_clone.clone();
            Task::spawn(async move {
                let n = attempts_clone.fetch_add(1, Ordering::SeqCst);
                if n < 2 { Err("fail") } else { Ok(42) }
            })
        })
        .await
    });

    assert!(
        matches!(result, Ok(42)),
        "Retry should succeed before limit"
    );
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        3,
        "Should have retried 3 times"
    );
}

#[test]
fn test_retry_fails_after_limit() {
    let rt = RuntimeBuilder::new().enable_io().build();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let result = rt.block_on(async {
        retry(3, || {
            let attempts_clone = attempts_clone.clone();
            Task::spawn(async move {
                attempts_clone.fetch_add(1, Ordering::SeqCst);
                Err::<usize, _>("fail")
            })
        })
        .await
    });

    assert!(result.is_err(), "Retry should fail after limit");
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        4,
        "Should have retried 4 times"
    );
}

#[test]
fn test_retry_with_interval() {
    use cadentis::time::sleep;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    let rt = RuntimeBuilder::new().enable_io().build();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();
    let last_time = Arc::new(Mutex::new(None));
    let last_time_clone = last_time.clone();
    let interval = Duration::from_millis(20);

    let result = rt.block_on(async {
        retry(3, move || {
            let attempts_clone = attempts_clone.clone();
            let last_time_clone = last_time_clone.clone();
            Task::spawn(async move {
                let now = Instant::now();
                let n = attempts_clone.fetch_add(1, Ordering::SeqCst);
                if n > 0 {
                    let mut last = last_time_clone.lock().unwrap();
                    if let Some(prev) = *last {
                        let elapsed = now.duration_since(prev);
                        assert!(
                            elapsed >= interval,
                            "Intervalle entre les tentatives trop court: {:?}",
                            elapsed
                        );
                    }
                    *last = Some(now);
                } else {
                    *last_time_clone.lock().unwrap() = Some(now);
                }
                if n < 2 {
                    sleep(interval).await;
                    Err("fail")
                } else {
                    Ok(77)
                }
            })
        })
        .await
    });

    assert!(
        matches!(result, Ok(77)),
        "Retry with interval should succeed"
    );
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        3,
        "Should have retried 3 times"
    );
}
#[test]
fn test_timeout_with_retry() {
    use cadentis::time::timeout;
    use std::time::Duration;

    let rt = RuntimeBuilder::new().enable_io().build();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let result = rt.block_on(async {
        retry(5, || {
            let attempts_clone = attempts_clone.clone();
            Task::spawn(async move {
                let n = attempts_clone.fetch_add(1, Ordering::SeqCst);
                timeout(Duration::from_millis(10), async move {
                    if n < 3 {
                        cadentis::time::sleep(Duration::from_millis(20)).await;
                        Ok::<_, &str>(0)
                    } else {
                        Ok::<_, &str>(123)
                    }
                })
                .await
                .map_err(|_| "timeout")?
            })
        })
        .await
    });

    assert!(
        matches!(result, Ok(123)),
        "Timeout+Retry doit finir par réussir"
    );
    assert!(
        attempts.load(Ordering::SeqCst) >= 4,
        "Doit avoir tenté au moins 4 fois"
    );
}
