use cadentis::join;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cadentis::test]
async fn test_join_single_future() {
    let a = join!(async { 42 });
    assert_eq!(a, 42);
}

#[cadentis::test]
async fn test_join_two_futures() {
    let (a, b) = join!(async { 10 }, async { 20 });
    assert_eq!((a, b), (10, 20));
}

#[cadentis::test]
async fn test_join_three_futures() {
    let (a, b, c) = join!(async { "hello" }, async { 42 }, async { true });
    assert_eq!((a, b, c), ("hello", 42, true));
}

#[cadentis::test]
async fn test_join_concurrent_execution() {
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    let c2 = counter.clone();
    let c3 = counter.clone();

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

    assert_eq!(counter.load(Ordering::SeqCst), 111);
}

#[cadentis::test]
async fn test_join_with_trailing_comma() {
    let (a, b) = join!(async { 1 }, async { 2 },);
    assert_eq!(a + b, 3);
}

#[cadentis::test]
async fn test_join_different_types() {
    let (num, text, v) = join!(async { 100i32 }, async { String::from("test") }, async {
        vec![1, 2, 3]
    });

    assert_eq!(num, 100);
    assert_eq!(text, "test");
    assert_eq!(v, vec![1, 2, 3]);
}

#[cadentis::test]
async fn test_join_with_captured_values() {
    let value = 50;
    let multiplier = 2;

    let (a, b) = join!(async move { value * multiplier }, async move {
        value + multiplier
    });

    assert_eq!((a, b), (100, 52));
}

#[cadentis::test]
async fn test_join_nested_async() {
    let outer = join!(async {
        let inner = async { 42 };
        inner.await * 2
    });

    assert_eq!(outer, 84);
}

#[cadentis::test]
async fn test_join_order_independence() {
    let order = Arc::new(std::sync::Mutex::new(Vec::new()));

    let o1 = order.clone();
    let o2 = order.clone();

    join!(
        async move {
            o1.lock().unwrap().push(1);
        },
        async move {
            o2.lock().unwrap().push(2);
        }
    );

    let recorded = order.lock().unwrap();
    assert_eq!(recorded.len(), 2);
    assert!(recorded.contains(&1));
    assert!(recorded.contains(&2));
}

#[cadentis::test]
async fn test_join_with_option_results() {
    let (a, b) = join!(async { Some(42) }, async { None::<i32> });

    assert_eq!(a, Some(42));
    assert_eq!(b, None);
}

#[cadentis::test]
async fn test_join_with_result_types() {
    let (ok_result, err_result) = join!(async { Ok::<i32, &str>(100) }, async {
        Err::<i32, &str>("error")
    });

    assert_eq!(ok_result, Ok(100));
    assert_eq!(err_result, Err("error"));
}
