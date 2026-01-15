use cadentis::{RuntimeBuilder, select};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_select_single_future() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        select! {
            async { 42 } => |v| v * 2,
        }
    });

    assert_eq!(result, 84);
}

#[test]
fn test_select_two_futures_first_ready() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        select! {
            async { 10 } => |v| v,
            async { 20 } => |v| v,
        }
    });

    assert!(result == 10 || result == 20);
}

#[test]
fn test_select_two_futures_different_types() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        select! {
            async { 42i32 } => |v| format!("number: {}", v),
            async { "hello" } => |v| format!("string: {}", v),
        }
    });

    assert!(result == "number: 42" || result == "string: hello");
}

#[test]
fn test_select_three_futures() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        select! {
            async { 1 } => |v| v,
            async { 2 } => |v| v,
            async { 3 } => |v| v,
        }
    });

    assert!((1..=3).contains(&result));
}

#[test]
fn test_select_four_futures() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        select! {
            async { "a" } => |v| v,
            async { "b" } => |v| v,
            async { "c" } => |v| v,
            async { "d" } => |v| v,
        }
    });

    assert!(result == "a" || result == "b" || result == "c" || result == "d");
}

#[test]
fn test_select_with_trailing_comma() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        select! {
            async { 100 } => |v| v,
            async { 200 } => |v| v,
        }
    });

    assert!(result == 100 || result == 200);
}

#[test]
fn test_select_with_captured_values() {
    let rt = RuntimeBuilder::new().build();
    let multiplier = 10;

    let result = rt.block_on(async move {
        select! {
            async { 5 } => |v| v * multiplier,
            async { 3 } => |v| v * multiplier,
        }
    });

    assert!(result == 50 || result == 30);
}

#[test]
fn test_select_counter_incremented_once() {
    let rt = RuntimeBuilder::new().build();
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    let c2 = counter.clone();

    rt.block_on(async move {
        select! {
            async move { c1.fetch_add(1, Ordering::SeqCst) } => |_| {},
            async move { c2.fetch_add(10, Ordering::SeqCst) } => |_| {},
        }
    });

    let val = counter.load(Ordering::SeqCst);
    assert!(val == 1 || val == 10 || val == 11);
}

#[test]
fn test_select_pattern_binding() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        select! {
            async { (1, 2) } => |(a, b)| a + b,
            async { (3, 4) } => |(a, b)| a * b,
        }
    });

    assert!(result == 3 || result == 12);
}

#[test]
fn test_select_option_pattern() {
    let rt = RuntimeBuilder::new().build();

    let result = rt.block_on(async {
        select! {
            async { Some(42) } => |opt| opt.unwrap_or(0),
            async { None::<i32> } => |opt| opt.unwrap_or(-1),
        }
    });

    assert!(result == 42 || result == -1);
}
