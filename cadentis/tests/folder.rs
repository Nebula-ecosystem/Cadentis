use cadentis::fs::Dir;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_base() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let base = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);

    base.join(format!("reactor_folder_test_{}_{}_{}", pid, nanos, seq))
}

#[cadentis::test]
fn folder_create_single() {
    let base = unique_temp_base();
    let base_str = base.to_string_lossy().into_owned();

    let dir = Dir::create(&base_str).await.expect("create single");
    assert_eq!(dir.path(), base);

    let meta = fs::metadata(&base_str).expect("metadata");
    assert!(meta.is_dir());

    fs::remove_dir(&base_str).expect("cleanup");
}

#[cadentis::test]
fn folder_create_all_nested_and_idempotent() {
    let base = unique_temp_base();
    let nested = base.join("a").join("b").join("c");
    let base_str = base.to_string_lossy().into_owned();
    let nested_str = nested.to_string_lossy().into_owned();

    let dir = Dir::create_all(&nested_str).await.expect("create_all");
    assert_eq!(dir.path(), nested);

    Dir::create_all(&nested_str)
        .await
        .expect("create_all idempotent");

    let meta = fs::metadata(&nested_str).expect("metadata nested");
    assert!(meta.is_dir());

    fs::remove_dir_all(&base_str).expect("cleanup nested");
}

#[cadentis::test]
fn folder_create_fails_when_exists() {
    let base = unique_temp_base();
    let base_str = base.to_string_lossy().into_owned();

    Dir::create(&base_str).await.expect("first create");

    let err = Dir::create(&base_str).await.err().expect("expected error");
    assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);

    fs::remove_dir(&base_str).expect("cleanup");
}

#[cadentis::test]
fn folder_exists_api() {
    let base = unique_temp_base();
    let base_str = base.to_string_lossy().into_owned();

    let dir = Dir::create(&base_str).await.expect("create");
    assert!(dir.exists());

    fs::remove_dir(&base_str).expect("cleanup");

    assert!(!dir.exists());
}
