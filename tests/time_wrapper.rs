use cadentis::RuntimeBuilder;
use cadentis::time::instrumented;
use cadentis::time::sleep;
use std::time::Duration;

#[test]
fn test_time_wrapper_with_sleep() {
    let rt = RuntimeBuilder::new().build();

    let (_, elapsed) = rt.block_on(async { instrumented(sleep(Duration::from_millis(50))).await });

    assert!(
        elapsed >= Duration::from_millis(50),
        "Time wrapper should measure at least the sleep duration"
    );
}
