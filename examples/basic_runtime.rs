//! Example: Basic usage of Cadentis runtime


use cadentis::RuntimeBuilder;

#[cadentis::main]
async fn main() {
    // Create a runtime with 4 worker threads
    let runtime = RuntimeBuilder::new().worker_threads(4).build();

    // Spawn a simple async task
    runtime.spawn(async {
        println!("Hello from Cadentis runtime!");
    });

    // The runtime starts automatically with cadentis::main
}
