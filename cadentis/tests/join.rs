use cadentis::{RuntimeBuilder, join};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_join_single_future() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        let a = join!(async { 42 });
        a
    });

    assert_eq!(result, 42);
}

#[test]
fn test_join_two_futures() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        let (a, b) = join!(async { 10 }, async { 20 });
        (a, b)
    });

    assert_eq!(result, (10, 20));
}

#[test]
fn test_join_three_futures() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        let (a, b, c) = join!(async { "hello" }, async { 42 }, async { true });
        (a, b, c)
    });

    assert_eq!(result, ("hello", 42, true));
}

#[test]
fn test_join_concurrent_execution() {
    let rt = RuntimeBuilder::new().build();
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    let c2 = counter.clone();
    let c3 = counter.clone();

    rt.block_on(async move {
        join!(
            async move {
                c1.fetch_add(1, Ordering::SeqCst);
            },
            async move {
                c2.fetch_add(10, Ordering::SeqCst);
            },
            async move {
                c3.fetch_add(100, Ordering::SeqCst);
            }
        );
    });

    assert_eq!(counter.load(Ordering::SeqCst), 111);
}

#[test]
fn test_join_with_trailing_comma() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        let (a, b) = join!(async { 1 }, async { 2 },);
        a + b
    });

    assert_eq!(result, 3);
}

#[test]
fn test_join_different_types() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        let (num, text, v) = join!(async { 100i32 }, async { String::from("test") }, async {
            vec![1, 2, 3]
        });
        (num, text, v)
    });

    assert_eq!(result.0, 100);
    assert_eq!(result.1, "test");
    assert_eq!(result.2, vec![1, 2, 3]);
}

#[test]
fn test_join_with_captured_values() {
    let rt = RuntimeBuilder::new().build();
    let value = 50;
    let multiplier = 2;

    let result = rt.block_on(async move {
        let (a, b) = join!(async move { value * multiplier }, async move {
            value + multiplier
        });
        (a, b)
    });

    assert_eq!(result, (100, 52));
}

#[test]
fn test_join_nested_async() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        let outer = join!(async {
            let inner = async { 42 };
            inner.await * 2
        });
        outer
    });

    assert_eq!(result, 84);
}

#[test]
fn test_join_order_independence() {
    let rt = RuntimeBuilder::new().build();
    let order = Arc::new(std::sync::Mutex::new(Vec::new()));

    let o1 = order.clone();
    let o2 = order.clone();

    rt.block_on(async move {
        join!(
            async move {
                o1.lock().unwrap().push(1);
            },
            async move {
                o2.lock().unwrap().push(2);
            }
        );
    });

    let recorded = order.lock().unwrap();
    assert_eq!(recorded.len(), 2);
    assert!(recorded.contains(&1));
    assert!(recorded.contains(&2));
}

#[test]
fn test_join_with_option_results() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        let (a, b) = join!(async { Some(42) }, async { None::<i32> });
        (a, b)
    });

    assert_eq!(result.0, Some(42));
    assert_eq!(result.1, None);
}

#[test]
fn test_join_with_result_types() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        let (ok_result, err_result) = join!(async { Ok::<i32, &str>(100) }, async {
            Err::<i32, &str>("error")
        });
        (ok_result, err_result)
    });

    assert_eq!(result.0, Ok(100));
    assert_eq!(result.1, Err("error"));
}
