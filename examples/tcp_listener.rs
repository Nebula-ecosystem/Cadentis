//! Example: TCP listener with Cadentis

use cadentis::net::TcpListener;

#[cadentis::main]
async fn main() {
    // Bind a TCP listener to localhost:8080
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening on 127.0.0.1:8080");
    loop {
        // Accept incoming connections asynchronously
        let (stream, addr) = listener.accept().await.unwrap();
        println!("Accepted connection from {}", addr);
        // You could spawn a task here to handle the stream
    }
}
