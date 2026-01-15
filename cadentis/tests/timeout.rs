use cadentis::time::sleep;
use cadentis::time::timeout;
use cadentis::task;
use std::time::Duration;

#[cadentis::test]
async fn test_timeout_completes_before_deadline() {
    let handle = task::spawn(async {
        sleep(Duration::from_millis(10)).await;
        123
    });

    let result = timeout(Duration::from_millis(50), handle).await;

    assert!(
        matches!(result, Ok(v) if v == 123),
        "Timeout should return Ok(123)"
    );
}

#[cadentis::test]
async fn test_timeout_expires() {
    let handle = task::spawn(async {
        sleep(Duration::from_millis(100)).await;
        456
    });
    let result = timeout(Duration::from_millis(20), handle).await;

    assert!(
        result.is_err(),
        "Timeout should return an error when deadline is exceeded"
    );
}
