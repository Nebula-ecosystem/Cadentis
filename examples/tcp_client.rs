//! Example: Asynchronous TCP client with Cadentis

use cadentis::net::TcpStream;

#[cadentis::main]
async fn main() {
    // Connect to a TCP server at localhost:8080
    let stream = TcpStream::connect("127.0.0.1:8080").await;
    match stream {
        Ok(mut s) => {
            // Send a message to the server
            let msg = b"Hello from client!";
            s.write_all(msg).await.unwrap();
            println!("Sent: {}", String::from_utf8_lossy(msg));
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
