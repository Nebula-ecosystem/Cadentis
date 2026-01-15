use cadentis::RuntimeBuilder;
use cadentis::task::spawn;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_single_worker_thread() {
    let rt = RuntimeBuilder::new().worker_threads(1).build();

    let result = rt.block_on(async { 42 });
    assert_eq!(result, 42);
}

#[test]
fn test_multiple_worker_threads() {
    let rt = RuntimeBuilder::new().worker_threads(4).build();

    let result = rt.block_on(async { 100 });
    assert_eq!(result, 100);
}

#[test]
fn test_worker_threads_parallel_execution() {
    let rt = RuntimeBuilder::new().worker_threads(4).build();

    let counter = Arc::new(Mutex::new(0));
    let results = Arc::new(Mutex::new(Vec::new()));

    let counter_clone = counter.clone();
    let results_clone = results.clone();

    rt.block_on(async move {
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let counter = counter_clone.clone();
                let results = results_clone.clone();

                spawn(async move {
                    let mut c = counter.lock().unwrap();
                    *c += 1;
                    drop(c);

                    results.lock().unwrap().push(i);
                    i * 2
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }
    });

    assert_eq!(*counter.lock().unwrap(), 10);
    assert_eq!(results.lock().unwrap().len(), 10);
}

#[test]
fn test_worker_threads_stress() {
    let rt = RuntimeBuilder::new().worker_threads(8).build();

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    rt.block_on(async move {
        let handles: Vec<_> = (0..100)
            .map(|_| {
                let counter = counter_clone.clone();
                spawn(async move {
                    let mut c = counter.lock().unwrap();
                    *c += 1;
                })
            })
            .collect();

        for handle in handles {
            handle.await;
        }
    });

    assert_eq!(*counter.lock().unwrap(), 100);
}

#[test]
fn test_worker_threads_max_parallelism() {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let rt = RuntimeBuilder::new().worker_threads(num_threads).build();

    let result = rt.block_on(async {
        let sum = Arc::new(Mutex::new(0));
        let handles: Vec<_> = (1..=10)
            .map(|i| {
                let sum = sum.clone();
                spawn(async move {
                    *sum.lock().unwrap() += i;
                })
            })
            .collect();

        for handle in handles {
            handle.await;
        }

        *sum.lock().unwrap()
    });

    assert_eq!(result, 55);
}

#[test]
fn test_worker_threads_chain_spawn() {
    let rt = RuntimeBuilder::new().worker_threads(4).build();

    let result = rt.block_on(async {
        let handle1 = spawn(async {
            let handle2 = spawn(async {
                let handle3 = spawn(async { 10 });
                handle3.await + 20
            });
            handle2.await + 30
        });
        handle1.await + 40
    });

    assert_eq!(result, 100);
}

#[test]
fn test_worker_threads_two_threads() {
    let rt = RuntimeBuilder::new().worker_threads(2).build();

    let completed = Arc::new(Mutex::new(HashSet::new()));
    let completed_clone = completed.clone();

    rt.block_on(async move {
        let handles: Vec<_> = (0..20)
            .map(|i| {
                let completed = completed_clone.clone();
                spawn(async move {
                    completed.lock().unwrap().insert(i);
                    i
                })
            })
            .collect();

        for handle in handles {
            handle.await;
        }
    });

    let set = completed.lock().unwrap();
    assert_eq!(set.len(), 20);
    for i in 0..20 {
        assert!(set.contains(&i), "Task {} should have completed", i);
    }
}

#[test]
#[should_panic(expected = "worker_threads must be > 0")]
fn test_worker_threads_zero_panics() {
    let _ = RuntimeBuilder::new().worker_threads(0).build();
}

#[test]
fn test_worker_threads_sequential_runtimes() {
    for n in 1..=4 {
        let rt = RuntimeBuilder::new().worker_threads(n).build();
        let result = rt.block_on(async move { n * 10 });
        assert_eq!(result, n * 10);
        drop(rt);
    }
}

#[test]
fn test_worker_threads_nested_spawns() {
    let rt = RuntimeBuilder::new().worker_threads(4).build();

    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    rt.block_on(async move {
        let outer_handles: Vec<_> = (0..4)
            .map(|i| {
                let results = results_clone.clone();
                spawn(async move {
                    let inner_handles: Vec<_> = (0..5)
                        .map(|j| {
                            let results = results.clone();
                            spawn(async move {
                                results.lock().unwrap().push(i * 10 + j);
                            })
                        })
                        .collect();

                    for handle in inner_handles {
                        handle.await;
                    }
                })
            })
            .collect();

        for handle in outer_handles {
            handle.await;
        }
    });

    let final_results = results.lock().unwrap();
    assert_eq!(final_results.len(), 20);
}
