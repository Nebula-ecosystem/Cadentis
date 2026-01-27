use cadentis::task::JoinSet;
use cadentis::time::sleep;
use std::time::Duration;

#[cadentis::test]
async fn joinset_abort_all() {
    let mut set = JoinSet::new();

    // Spawn a task that would take a long time
    set.spawn(async move {
        sleep(Duration::from_millis(500)).await;
        "should be cancelled"
    });

    // Immediately abort everything
    set.abort_all();

    assert!(set.is_empty(), "Set should be empty after abort_all");

    // join_next should return None immediately because handles were cleared
    assert!(set.join_next().await.is_none());
}

#[cadentis::test]
async fn joinset_race_condition() {
    let mut set = JoinSet::new();

    // Task 1: Fast
    set.spawn(async move {
        sleep(Duration::from_millis(10)).await;
        "winner"
    });

    // Task 2: Slow
    set.spawn(async move {
        sleep(Duration::from_millis(200)).await;
        "loser"
    });

    // Race should complete after the first one finishes
    let result = set.race().await;

    assert!(result.is_ok());
    assert!(set.is_empty(), "Remaining tasks should have been aborted");
}

#[cadentis::test]
async fn joinset_race_n_not_enough_tasks() {
    let mut set = JoinSet::new();

    set.spawn(async move { 1 });

    // We try to race 5 tasks but only 1 is spawned
    let result = set.race_n(5).await;

    assert!(result.is_err(), "Should return error if n > handles.len()");
}

#[cadentis::test]
async fn joinset_drop_cancels_tasks() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = flag.clone();

    {
        let mut set = JoinSet::new();
        set.spawn(async move {
            sleep(Duration::from_millis(100)).await;
            flag_clone.store(true, Ordering::SeqCst);
        });
        // Set is dropped here
    }

    sleep(Duration::from_millis(150)).await;

    assert!(
        !flag.load(Ordering::SeqCst),
        "Task should have been cancelled on drop"
    );
}

#[cadentis::test]
async fn joinset_is_empty_and_len() {
    let mut set = JoinSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);

    set.spawn(async move { sleep(Duration::from_millis(10)).await });
    set.spawn(async move { sleep(Duration::from_millis(10)).await });

    assert!(!set.is_empty());
    assert_eq!(set.len(), 2);

    set.join_next().await;
    assert_eq!(set.len(), 1);

    set.join_next().await;
    assert!(set.is_empty());
}
