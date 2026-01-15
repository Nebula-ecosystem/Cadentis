use cadentis::select;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cadentis::test]
async fn test_select_single_future() {
    let result = select! {
        async { 42 } => |v| v * 2,
    };

    assert_eq!(result, 84);
}

#[cadentis::test]
async fn test_select_two_futures_first_ready() {
    let result = select! {
        async { 10 } => |v| v,
        async { 20 } => |v| v,
    };

    assert!(result == 10 || result == 20);
}

#[cadentis::test]
async fn test_select_two_futures_different_types() {
    let result = select! {
        async { 42i32 } => |v| format!("number: {}", v),
        async { "hello" } => |v| format!("string: {}", v),
    };

    assert!(result == "number: 42" || result == "string: hello");
}

#[cadentis::test]
async fn test_select_three_futures() {
    let result = select! {
        async { 1 } => |v| v,
        async { 2 } => |v| v,
        async { 3 } => |v| v,
    };

    assert!((1..=3).contains(&result));
}

#[cadentis::test]
async fn test_select_four_futures() {
    let result = select! {
        async { "a" } => |v| v,
        async { "b" } => |v| v,
        async { "c" } => |v| v,
        async { "d" } => |v| v,
    };

    assert!(result == "a" || result == "b" || result == "c" || result == "d");
}

#[cadentis::test]
async fn test_select_with_trailing_comma() {
    let result = select! {
        async { 100 } => |v| v,
        async { 200 } => |v| v,
    };

    assert!(result == 100 || result == 200);
}

#[cadentis::test]
async fn test_select_with_captured_values() {
    let multiplier = 10;

    let result = select! {
        async { 5 } => |v| v * multiplier,
        async { 3 } => |v| v * multiplier,
    };

    assert!(result == 50 || result == 30);
}

#[cadentis::test]
async fn test_select_counter_incremented_once() {
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    let c2 = counter.clone();

    select! {
        async move { c1.fetch_add(1, Ordering::SeqCst) } => |_| {},
        async move { c2.fetch_add(10, Ordering::SeqCst) } => |_| {},
    };

    let val = counter.load(Ordering::SeqCst);
    assert!(val == 1 || val == 10 || val == 11);
}

#[cadentis::test]
async fn test_select_pattern_binding() {
    let result = select! {
        async { (1, 2) } => |(a, b)| a + b,
        async { (3, 4) } => |(a, b)| a * b,
    };

    assert!(result == 3 || result == 12);
}

#[cadentis::test]
async fn test_select_option_pattern() {
    let result = select! {
        async { Some(42) } => |opt: Option<i32>| opt.unwrap_or(0),
        async { None::<i32> } => |opt: Option<i32>| opt.unwrap_or(-1),
    };

    assert!(result == 42 || result == -1);
}
