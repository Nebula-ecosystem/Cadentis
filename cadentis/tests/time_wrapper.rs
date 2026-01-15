use cadentis::time::instrumented;
use cadentis::time::sleep;
use std::time::Duration;

#[cadentis::test]
async fn test_time_wrapper_with_sleep() {
    let (_, elapsed) = instrumented(sleep(Duration::from_millis(50))).await;

    assert!(
        elapsed >= Duration::from_millis(50),
        "Time wrapper should measure at least the sleep duration"
    );
}
