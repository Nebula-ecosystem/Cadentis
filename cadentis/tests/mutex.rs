use cadentis::sync::Mutex;
use cadentis::task;
use std::sync::Arc;

#[cadentis::test]
async fn async_mutex_shared_counter() {
    let counter = Arc::new(Mutex::new(0usize));

    let mut handles = Vec::new();

    for _ in 0..10 {
        let counter = counter.clone();
        handles.push(task::spawn(async move {
            let mut guard = counter.lock().await;
            *guard += 1;
        }));
    }

    for handle in handles {
        handle.await;
    }

    let guard = counter.lock().await;
    assert_eq!(*guard, 10);
}
