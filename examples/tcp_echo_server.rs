//! Example: TCP echo server with Cadentis

use cadentis::net::{TcpListener, TcpStream};
use cadentis::task;

#[cadentis::main]
async fn main() {
    // Bind a TCP echo server to localhost:9000
    let listener = TcpListener::bind("127.0.0.1:9000").unwrap();
    println!("Echo server listening on 127.0.0.1:9000");
    loop {
        // Accept incoming connections asynchronously
        let (stream, addr) = listener.accept().await.unwrap();
        println!("Accepted connection from {}", addr);
        // Spawn a task to handle each client
        task::spawn(handle_client(stream));
    }
}

// Echo handler: reads data and writes it back to the client
async fn handle_client(stream: TcpStream) {
    let mut buf = [0u8; 1024];
    loop {
        let n = match stream.read(&mut buf).await {
            Ok(n) => n,
            Err(_) => break,
        };
        if stream.write_all(&buf[..n]).await.is_err() {
            break;
        }
    }
}
