use cadentis::time::Time;
use cadentis::time::sleep;
use cadentis::{RuntimeBuilder, task};
use std::time::Duration;

#[test]
fn test_time_wrapper_with_sleep() {
    let rt = RuntimeBuilder::new().build();

    let (_, elapsed) = rt.block_on(async {
        let handle = task::spawn(async {
            sleep(Duration::from_millis(50)).await;
        });

        Time::new(handle).await
    });

    assert!(
        elapsed >= Duration::from_millis(50),
        "Time wrapper should measure at least the sleep duration"
    );
}
